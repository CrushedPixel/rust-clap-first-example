# CLAP Plugin Build System (xtask)

This module follows the [xtask pattern](https://github.com/matklad/cargo-xtask) to provide a custom build system for
CLAP-first Rust plugins.
It handles the integration between Rust code and the C++ clap-wrapper project to build plugins in multiple formats.

## How It Works

1. The Rust code is compiled into a static library
2. The `xtask` tool creates a CMake build environment with clap-wrapper and builds the final plugin formats (CLAP, VST3,
   AU).

## Command Reference

The build system is invoked via the cargo alias defined in `.cargo/config.toml`:

```bash
cargo xtask build <CRATE_NAME> [OPTIONS]
```

### Options

| Option                | Description                                                                     |
|-----------------------|---------------------------------------------------------------------------------|
| `--release`           | Build using the release profile. Default is debug.                              |
| `--bundle-id <ID>`    | Set bundle identifier (default: "org.free-audio.rust-gain-example")             |
| `--formats <FORMATS>` | Comma-separated list of formats to build (default: "CLAP,VST3,AUV2")            |
| `--clean`             | Clean build directories before building                                         |
| `--install`           | Install plugins to system directories after building (not supported on Windows) |

### Examples

```bash
# Basic build
cargo xtask build gain-example

# Release build with custom bundle ID
cargo xtask build gain-example --release --bundle-id "com.mycompany.myplugin"

# Build only CLAP and VST3 formats
cargo xtask build gain-example --formats "CLAP,VST3"

# Clean build with installation
cargo xtask build gain-example --clean --install
```

## Adding New Plugins

To add a new plugin:

1. Create a new crate in the `plugins/` directory
2. Ensure it has a `staticlib` crate type in `Cargo.toml`
3. Implement the necessary CLAP interfaces
4. Run `cargo xtask build your-new-plugin`
