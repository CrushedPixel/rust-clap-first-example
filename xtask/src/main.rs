use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Parser)]
#[command(
    name = "xtask",
    about = "Build CLAP-first audio plugins from a Rust crate"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Build a crate as a CLAP plugin
    Build {
        /// The crate to build as a static library
        crate_name: String,

        /// Release mode (default is debug)
        #[arg(long)]
        release: bool,

        /// Plugin bundle identifier
        #[arg(long, default_value = "org.free-audio.rust-gain-example")]
        bundle_id: String,

        /// Plugin formats to build (comma-separated: CLAP,VST3,AUV2)
        #[arg(long, default_value = "CLAP,VST3,AUV2")]
        formats: String,

        /// Clean build directories first
        #[arg(long)]
        clean: bool,

        /// Install the resulting plugins to the local drive.
        /// Not supported on Windows.
        #[arg(long)]
        install: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            crate_name,
            release,
            bundle_id,
            formats,
            clean,
            install,
        } => build_plugin(crate_name, release, bundle_id, formats, clean, install)?,
    }

    Ok(())
}

/// Build a plugin from a Rust crate
fn build_plugin(
    crate_name: String,
    release: bool,
    bundle_id: String,
    formats: String,
    clean: bool,
    install: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the project root directory
    let project_root = project_root();

    // Clean if requested
    if clean {
        println!("Cleaning build directories...");
        let _ = std::fs::remove_dir_all(project_root.join("target/plugins"));
        let _ = std::fs::remove_dir_all(project_root.join("target/cmake-build"));
    }

    // Build the static library
    println!("Building static library for crate '{}'...", crate_name);

    let mut cargo_args = vec!["build"];

    // Configure build profile
    if release {
        cargo_args.push("--release");
    }

    // Add the crate to build
    cargo_args.push("-p");
    cargo_args.push(&crate_name);

    let status = Command::new("cargo")
        .args(&cargo_args)
        .current_dir(&project_root)
        .status()?;

    if !status.success() {
        return Err("Failed to build static library".into());
    }

    // Determine the output directory based on build profile
    let profile = if release { "release" } else { "debug" };
    let target_dir = project_root.join("target").join(profile);

    // Generate the library name based on the platform
    let normalized_crate_name = crate_name.replace('-', "_");

    // Determine the static library name based on the platform
    let static_lib_file = if cfg!(windows) {
        // On Windows, the static library is named: crate_name.lib
        target_dir.join(format!("{}.lib", normalized_crate_name))
    } else {
        // On Unix-like systems (Linux, macOS), the static library is named: libcrate_name.a
        target_dir.join(format!("lib{}.a", normalized_crate_name))
    };

    if !static_lib_file.exists() {
        return Err(format!(
            "Static library file not found: {}",
            static_lib_file.display()
        )
        .into());
    }

    println!("Found static library: {}", static_lib_file.display());

    // Create the CMake build directory
    let cmake_build_dir = project_root.join("target/cmake-build");
    std::fs::create_dir_all(&cmake_build_dir)?;

    // Path to the CMake script and source files
    let cmake_dir = project_root.join("xtask/cmake");
    let build_cmake = cmake_dir.join("CMakeLists.txt");
    let clap_entry_cpp = cmake_dir.join("clap_entry.cpp");
    let clap_entry_h = cmake_dir.join("clap_entry.h");

    // Check if the required files exist
    if !build_cmake.exists() || !clap_entry_cpp.exists() || !clap_entry_h.exists() {
        return Err("Required CMake files not found in xtask/cmake directory".into());
    }

    // Copy files required for CMake build to the build directory
    std::fs::copy(&clap_entry_cpp, cmake_build_dir.join("clap_entry.cpp"))?;
    std::fs::copy(&clap_entry_h, cmake_build_dir.join("clap_entry.h"))?;
    std::fs::copy(&build_cmake, cmake_build_dir.join("CMakeLists.txt"))?;

    // Plugin output directory
    let plugin_output_dir = project_root.join(target_dir.join("plugins"));
    std::fs::create_dir_all(&plugin_output_dir)?;

    // Run CMake to configure the build
    println!("Configuring CMake build...");

    let status = Command::new("cmake")
        .current_dir(&cmake_build_dir)
        .arg("-S")
        .arg(cmake_dir)
        .arg("-B")
        .arg(&cmake_build_dir)
        .arg(format!("-DPROJECT_NAME={}", crate_name))
        .arg(format!("-DSTATIC_LIB_FILE={}", static_lib_file.display()))
        .arg(format!("-DBUNDLE_ID={}", bundle_id))
        .arg(format!(
            "-DPLUGIN_OUTPUT_DIR={}",
            plugin_output_dir.display()
        ))
        .arg(format!(
            "-DINSTALL_PLUGINS_AFTER_BUILD={}",
            if install { "ON" } else { "OFF" }
        ))
        .arg(format!("-DPLUGIN_FORMATS={}", formats))
        .status()?;

    if !status.success() {
        return Err("CMake configuration failed".into());
    }

    // Build the plugins
    println!("Building plugins...");
    let status = Command::new("cmake")
        .current_dir(&cmake_build_dir)
        .arg("--build")
        .arg(".")
        .arg("--config")
        .arg(if release { "Release" } else { "Debug" })
        .status()?;

    if !status.success() {
        return Err("Plugin build failed".into());
    }

    println!("Build completed successfully!");
    println!("Plugins are available in: {}", plugin_output_dir.display());

    Ok(())
}

/// Get the project root directory
fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
