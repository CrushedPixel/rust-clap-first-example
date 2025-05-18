use clack_extensions::gui::{GuiApiType, GuiSize};
use wry::dpi::{LogicalSize, Size};

/// Whether the current platform uses logical sizes for GUIs.
/// If true, [GuiSize] values provided by and sent to the CLAP API
/// are in logical pixels, otherwise in physical pixels.
pub(super) fn host_uses_logical_size() -> bool {
    GuiApiType::default_for_current_platform()
        .unwrap()
        .uses_logical_size()
}

/// Trait for converting LogicalSize to GuiSize
pub(super) trait LogicalSizeExtensions {
    fn to_host_size(&self, scale_factor: f64) -> GuiSize;
    fn to_webview_size(&self, scale_factor: f64) -> Size;
}

/// Trait for converting GuiSize to LogicalSize
pub(super) trait GuiSizeExtensions {
    fn to_logical(&self, scale_factor: f64) -> LogicalSize<f64>;
}

/// Implement conversions for LogicalSize
impl LogicalSizeExtensions for LogicalSize<f64> {
    /// Converts a [LogicalSize] value into a [GuiSize] value
    /// that is appropriate for the host's pixel format.
    fn to_host_size(&self, scale_factor: f64) -> GuiSize {
        if host_uses_logical_size() {
            GuiSize {
                width: self.width as u32,
                height: self.height as u32,
            }
        } else {
            GuiSize {
                width: (self.width * scale_factor) as u32,
                height: (self.height * scale_factor) as u32,
            }
        }
    }

    /// Converts a [LogicalSize] into a [Size] value to provide to the WebView Rect.
    fn to_webview_size(&self, scale_factor: f64) -> Size {
        if host_uses_logical_size() {
            Size::Logical(*self)
        } else {
            Size::Physical(self.to_physical(scale_factor))
        }
    }
}

/// Implement conversion for GuiSize
impl GuiSizeExtensions for GuiSize {
    /// Converts a [GuiSize] value from the host's format
    /// into a standardized [LogicalSize] value.
    fn to_logical(&self, scale_factor: f64) -> LogicalSize<f64> {
        if host_uses_logical_size() {
            LogicalSize {
                width: self.width as f64,
                height: self.height as f64,
            }
        } else {
            LogicalSize {
                width: self.width as f64 / scale_factor,
                height: self.height as f64 / scale_factor,
            }
        }
    }
}
