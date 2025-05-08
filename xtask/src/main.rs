use clap::{Parser, Subcommand};
use std::fs;
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
            clean,
            install,
        } => build_plugin(crate_name, release, bundle_id, clean, install)?,
    }

    Ok(())
}

/// Build a plugin from a Rust crate
fn build_plugin(
    crate_name: String,
    release: bool,
    bundle_id: String,
    clean: bool,
    install: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the project root directory
    let project_root = project_root();

    // Clean if requested
    if clean {
        println!("Cleaning build directories...");
        let _ = fs::remove_dir_all(project_root.join("target/cmake-build"));
        let _ = fs::remove_dir_all(project_root.join("target/cmake-assets"));
        let _ = fs::remove_dir_all(project_root.join("target/plugins"));
    }

    // Normalize crate name for file naming
    let normalized_crate_name = crate_name.replace('-', "_");

    // Determine the output directory based on build profile
    let profile = if release { "release" } else { "debug" };

    let static_lib_file = if cfg!(target_os = "macos") {
        // on macOS, build for both architectures
        // and create a universal binary using lipo
        build_universal_macos_binary(&project_root, &crate_name, &normalized_crate_name, release)?
    } else {
        // Regular build for the current architecture
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

        let target_dir = project_root.join("target").join(profile);

        // Determine the static library name based on the platform
        if cfg!(windows) {
            // On Windows, the static library is named: crate_name.lib
            target_dir.join(format!("{}.lib", normalized_crate_name))
        } else {
            // On Unix-like systems (Linux, macOS), the static library is named: libcrate_name.a
            target_dir.join(format!("lib{}.a", normalized_crate_name))
        }
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
    fs::create_dir_all(&cmake_build_dir)?;

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
    fs::copy(&clap_entry_cpp, cmake_build_dir.join("clap_entry.cpp"))?;
    fs::copy(&clap_entry_h, cmake_build_dir.join("clap_entry.h"))?;
    fs::copy(&build_cmake, cmake_build_dir.join("CMakeLists.txt"))?;

    // Create a temporary assets directory for CMake output
    let cmake_assets_dir = project_root.join("target/cmake-assets");
    fs::create_dir_all(&cmake_assets_dir)?;

    // Final plugin output directory
    let plugin_output_dir = project_root.join("target").join(profile).join("plugins");
    fs::create_dir_all(&plugin_output_dir)?;

    // Run CMake to configure the build
    println!("Configuring CMake build...");

    let mut cmake_args = vec![
        "-S".to_string(),
        cmake_dir.display().to_string(),
        "-B".to_string(),
        cmake_build_dir.display().to_string(),
        format!("-DPROJECT_NAME={}", crate_name),
        format!("-DSTATIC_LIB_FILE={}", static_lib_file.display()),
        format!("-DBUNDLE_ID={}", bundle_id),
        format!("-DPLUGIN_OUTPUT_DIR={}", cmake_assets_dir.display()),
        format!(
            "-DINSTALL_PLUGINS_AFTER_BUILD={}",
            if install { "ON" } else { "OFF" }
        ),
    ];

    let status = Command::new("cmake")
        .current_dir(&cmake_build_dir)
        .args(&cmake_args)
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

    // Copy the plugin files from the CMake output directory to the final plugin directory
    println!("Copying plugin files to final destination...");
    copy_plugin_files(&cmake_assets_dir, &plugin_output_dir, &profile)?;

    println!("Build completed successfully!");
    println!("Plugins are available in: {}", plugin_output_dir.display());

    Ok(())
}

/// Build a universal binary for macOS by building for both architectures and combining with lipo
fn build_universal_macos_binary(
    project_root: &Path,
    crate_name: &str,
    normalized_crate_name: &str,
    release: bool,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Ensure both targets are available
    let status = Command::new("rustup")
        .args(&[
            "target",
            "add",
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
        ])
        .status()?;

    if !status.success() {
        return Err("Failed to add required targets".into());
    }

    // Build profile
    let profile = if release { "release" } else { "debug" };

    // Build for x86_64 (Intel)
    println!("Building for x86_64-apple-darwin...");
    let mut cargo_args = vec!["build"];

    if release {
        cargo_args.push("--release");
    }

    cargo_args.extend(&["--target", "x86_64-apple-darwin", "-p", crate_name]);

    let status = Command::new("cargo")
        .args(&cargo_args)
        .current_dir(project_root)
        .status()?;

    if !status.success() {
        return Err("Failed to build for x86_64-apple-darwin".into());
    }

    // Build for arm64 (Apple Silicon)
    println!("Building for aarch64-apple-darwin...");
    let mut cargo_args = vec!["build"];

    if release {
        cargo_args.push("--release");
    }

    cargo_args.extend(&["--target", "aarch64-apple-darwin", "-p", crate_name]);

    let status = Command::new("cargo")
        .args(&cargo_args)
        .current_dir(project_root)
        .status()?;

    if !status.success() {
        return Err("Failed to build for aarch64-apple-darwin".into());
    }

    // Path to the x86_64 and arm64 libraries
    let x86_64_lib = project_root
        .join("target")
        .join("x86_64-apple-darwin")
        .join(profile)
        .join(format!("lib{}.a", normalized_crate_name));

    let arm64_lib = project_root
        .join("target")
        .join("aarch64-apple-darwin")
        .join(profile)
        .join(format!("lib{}.a", normalized_crate_name));

    // Create output directory for universal binary
    let universal_dir = project_root.join("target").join("universal");
    fs::create_dir_all(&universal_dir)?;

    // Path for the universal library
    let universal_lib = universal_dir.join(format!("lib{}.a", normalized_crate_name));

    // Use lipo to create universal binary
    println!(
        "Creating universal binary with lipo: {}",
        universal_lib.display()
    );
    let status = Command::new("lipo")
        .args(&[
            "-create",
            &x86_64_lib.to_string_lossy(),
            &arm64_lib.to_string_lossy(),
            "-output",
            &universal_lib.to_string_lossy(),
        ])
        .status()?;

    if !status.success() {
        return Err("Failed to create universal binary with lipo".into());
    }

    // Verify the universal binary
    let output = Command::new("lipo")
        .args(&["-info", &universal_lib.to_string_lossy()])
        .output()?;

    if output.status.success() {
        let info = String::from_utf8_lossy(&output.stdout);
        println!("Universal binary info: {}", info.trim());
    }

    Ok(universal_lib)
}

/// Copy plugin files from CMake output to final destination
fn copy_plugin_files(
    source_dir: &Path,
    dest_dir: &Path,
    profile: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create destination directory if it doesn't exist
    fs::create_dir_all(dest_dir)?;

    // Handle platform-specific differences
    if cfg!(target_os = "windows") {
        // On Windows, we need to handle the nested file structure
        for format in ["VST3", "CLAP"] {
            let format_source_dir = source_dir.join(format).join(profile);
            if format_source_dir.exists() {
                for entry in fs::read_dir(&format_source_dir)? {
                    let entry = entry?;
                    let source_path = entry.path();
                    if source_path.is_file() {
                        let dest_path = dest_dir.join(source_path.file_name().unwrap());
                        fs::copy(&source_path, &dest_path)?;
                    } else if source_path.is_dir() {
                        let dest_subdir = dest_dir.join(source_path.file_name().unwrap());
                        copy_dir_recursive(&source_path, &dest_subdir)?;
                    }
                }
            }
        }
    } else {
        // On macOS, files are output directly in the asset output directory.
        // it's a sensible default for Linux as well
        copy_dir_recursive(source_dir, dest_dir)?;
    }

    Ok(())
}

/// Copy all files and directories recursively
fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest.join(path.file_name().unwrap());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(path, dest_path)?;
        }
    }

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
