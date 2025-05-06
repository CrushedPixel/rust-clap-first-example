# Rust CLAP-First Plugin Example

> Write a CLAP plugin in Rust, get self-contained AU and VST3 plugins for free!

This repository demonstrates a Rust-based approach to audio plugin development that starts with
the [CLAP](https://cleveraudio.org/) plugin format and extends to VST3 and AU formats using
the [clap-wrapper](https://github.com/free-audio/clap-wrapper/) project.
The resulting plugins are self-contained thanks to a static linking approach.

## Example Plugin

This example exposes two variations of a simple gain plugin:

- **Gain Halver**: Reduces input signal by 50%
- **Gain Doubler**: Doubles the input signal

The plugin demonstrates:

- Basic CLAP plugin structure in Rust
- Multi-plugin export from CLAP
- Support for multi-plugin AU with AUv2 factory
- Format-specific extension support

## Requirements

- Rust toolchain (2021 edition or later)
- CMake (3.15 or later)
- C++ compiler with C++17 support

## Building the Plugins

The project uses the [xtask](https://github.com/matklad/cargo-xtask) pattern to simplify building:

```bash
# Build debug version
cargo xtask build gain-example

# Build release version
cargo xtask build gain-example --release

# Build and install release version to user's plugin directories (macOS/Linux only)
cargo xtask build gain-example --release --install
```

See the [xtask README](./xtask/README.md) for more detailed commands and options.

## How It Works

1. The Rust code is compiled into a static library that exports a non-standard `rust_clap_entry` symbol
2. A small C++ shim re-exports this symbol as the standard CLAP entry point
3. clap-wrapper builds self-contained plugins for CLAP, VST3, and AU formats

## Customizing

To adapt this example for your own plugin:

1. Rename/duplicate the `gain-example` plugin directory
2. Modify the implementations in `audio_thread.rs` and `main_thread.rs`
3. Update the plugin descriptors in `lib.rs`
4. Update bundle IDs and other metadata in build commands

## Acknowledgements

- [@Prokopyl](https://github.com/prokopyl) for providing Rust bindings for the clap-wrapper's extensions
- [rust-clap-wrapper-thick](https://github.com/SG60/rust-clap-wrapper-thick) for their pioneering work

## License

MIT OR Apache-2.0