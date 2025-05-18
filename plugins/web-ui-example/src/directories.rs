use std::path::PathBuf;

/// Returns a directory to store plugin data in.
pub fn global_data_dir() -> PathBuf {
    dirs::data_dir().unwrap().join("rust-web-ui-example")
}
