use crate::directories::global_data_dir;
use crate::gui::dpi::{GuiSizeExtensions, LogicalSizeExtensions};
use crate::main_thread::WebUiPluginMainThread;
use clack_extensions::gui::{
    AspectRatioStrategy, GuiApiType, GuiConfiguration, GuiResizeHints, GuiSize, PluginGuiImpl,
    Window,
};
use clack_plugin::prelude::*;
use std::env;
use std::num::{NonZeroIsize, NonZeroU32};
use std::ptr::NonNull;
use wry::dpi::{LogicalSize, PhysicalPosition, Position};
use wry::raw_window_handle::{
    AppKitWindowHandle, RawWindowHandle, Win32WindowHandle, WindowHandle, XcbWindowHandle,
};
use wry::{Rect, WebViewBuilder};

mod dpi;

pub const DEFAULT_GUI_SIZE: LogicalSize<f64> = LogicalSize::new(400.0, 300.0);
pub const MIN_GUI_SIZE: LogicalSize<f64> = LogicalSize::new(200.0, 100.0);
pub const MAX_GUI_SIZE: LogicalSize<f64> = LogicalSize::new(600.0, 600.0);

/// Implements the CLAP GUI extension.
///
/// This implementation allows for resizing
/// between [MIN_GUI_SIZE] and [MAX_GUI_SIZE].
impl<'a> PluginGuiImpl for WebUiPluginMainThread<'a> {
    fn is_api_supported(&mut self, configuration: GuiConfiguration) -> bool {
        // only support our preferred API -
        // our requirements in get_preferred_api
        // are sensible and pretty much guaranteed
        // to be met by all DAWs.
        configuration == self.get_preferred_api().unwrap()
    }

    fn get_preferred_api(&mut self) -> Option<GuiConfiguration> {
        Some(GuiConfiguration {
            api_type: GuiApiType::default_for_current_platform()?,
            // no known host supports floating mode at this time
            is_floating: false,
        })
    }

    fn create(&mut self, configuration: GuiConfiguration) -> Result<(), PluginError> {
        if !self.is_api_supported(configuration) {
            return Err(PluginError::Message("Unsupported GUI configuration"));
        }

        // this function is intended for the floating mode flow.
        // since we only support non-floating windows,
        // set_parent will be called at a later point,
        // and we will instantiate the GUI there,
        // as we need to know the parent window to pass to Wry.
        Ok(())
    }

    fn destroy(&mut self) {
        self.web_view.take();
    }

    fn set_scale(&mut self, scale: f64) -> Result<(), PluginError> {
        self.scale_factor = scale;
        Ok(())
    }

    fn get_size(&mut self) -> Option<GuiSize> {
        Some(self.gui_size.to_host_size(self.scale_factor))
    }

    fn can_resize(&mut self) -> bool {
        true
    }

    fn get_resize_hints(&mut self) -> Option<GuiResizeHints> {
        Some(GuiResizeHints {
            can_resize_horizontally: true,
            can_resize_vertically: true,
            // change this to Preserve if you want to keep
            // the aspect ratio when resizing.
            strategy: AspectRatioStrategy::Disregard,
        })
    }

    fn adjust_size(&mut self, size: GuiSize) -> Option<GuiSize> {
        // convert from host to logical size
        let mut size = size.to_logical(self.scale_factor);

        // constrain the size
        size.width = size.width.clamp(MIN_GUI_SIZE.width, MAX_GUI_SIZE.width);
        size.height = size.height.clamp(MIN_GUI_SIZE.height, MAX_GUI_SIZE.height);

        // convert back to host format
        Some(size.to_host_size(self.scale_factor))
    }

    fn set_size(&mut self, size: GuiSize) -> Result<(), PluginError> {
        self.gui_size = size.to_logical(self.scale_factor);
        if let Some(web_view) = &mut self.web_view {
            web_view.set_bounds(Rect {
                position: Position::Physical(PhysicalPosition::new(0, 0)),
                size: self.gui_size.to_webview_size(self.scale_factor),
            })?;
        }

        Ok(())
    }

    fn set_parent(&mut self, parent: Window) -> Result<(), PluginError> {
        // here's where we create the editor.

        // convert clap window to WindowHandle expected by wry
        let parent = unsafe {
            WindowHandle::borrow_raw(if cfg!(target_os = "macos") {
                RawWindowHandle::AppKit(AppKitWindowHandle::new(
                    NonNull::new(parent.as_cocoa_nsview().unwrap()).unwrap(),
                ))
            } else if cfg!(target_os = "windows") {
                RawWindowHandle::Win32(Win32WindowHandle::new(
                    NonZeroIsize::new(parent.as_win32_hwnd().unwrap() as isize).unwrap(),
                ))
            } else {
                RawWindowHandle::Xcb(XcbWindowHandle::new(
                    NonZeroU32::new(parent.as_x11_handle().unwrap() as u32).unwrap(),
                ))
            })
        };

        if cfg!(target_os = "windows") {
            // WebView2 crashes on Windows if no valid data directory is set.
            // Therefore, we provide a directory we control.
            // The environment variable only changes for this process,
            // so it doesn't affect any other programs on the system.
            let web_view_dir = global_data_dir().join("webview2");
            env::set_var("WEBVIEW2_USER_DATA_FOLDER", web_view_dir);
        }

        // now we can create the web view!

        self.web_view = Some(
            WebViewBuilder::new()
                // load HTML from our local file.
                // by using include_str, the file contents
                // are statically embedded into the plugin binary.
                .with_html(include_str!("index.html"))
                // enable dev tools in debug builds
                .with_devtools(cfg!(debug_assertions))
                // set initial size
                .with_bounds(Rect {
                    position: Position::Physical(PhysicalPosition::new(0, 0)),
                    size: self.gui_size.to_webview_size(self.scale_factor),
                })
                // open any website links in the browser instead of the UI webview
                .with_navigation_handler(|url| {
                    if url.starts_with("http") {
                        if let Err(_e) = open::that(url) {
                            // log this if you want
                        }
                        false
                    } else {
                        true
                    }
                })
                .build_as_child(&parent)?,
        );

        Ok(())
    }

    fn set_transient(&mut self, _window: Window) -> Result<(), PluginError> {
        // does not apply to parented windows
        Ok(())
    }

    fn show(&mut self) -> Result<(), PluginError> {
        // does not apply to parented windows
        Ok(())
    }

    fn hide(&mut self) -> Result<(), PluginError> {
        // does not apply to parented windows
        Ok(())
    }
}
