use crate::gui::DEFAULT_GUI_SIZE;
use clack_plugin::prelude::*;
use wry::dpi::LogicalSize;
use wry::WebView;

pub struct WebUiPluginMainThread<'a> {
    #[allow(dead_code)] // unused in example
    host: HostMainThreadHandle<'a>,

    // --- GUI fields ---
    /// The web view displaying the GUI.
    pub(crate) web_view: Option<WebView>,

    /// The scale factor of the window hosting the GUI.
    /// Only used when dealing in physical pixels.
    pub(crate) scale_factor: f64,

    /// The GUI size. We store this in logical pixels
    /// instead of [GuiSize] to be independent
    /// of the platform's interpretation of [GuiSize]
    /// in our own calculations.
    pub(crate) gui_size: LogicalSize<f64>,
}

impl<'a> WebUiPluginMainThread<'a> {
    pub fn create(host: HostMainThreadHandle<'a>) -> Result<Self, PluginError> {
        Ok(Self {
            host,

            web_view: None,
            scale_factor: 1.0,
            // set the initial GUI size here.
            // you can also assign to this value when loading state.
            gui_size: DEFAULT_GUI_SIZE,
        })
    }
}

impl<'a> PluginMainThread<'a, ()> for WebUiPluginMainThread<'a> {
    fn on_main_thread(&mut self) {
        // in a real plugin, you might exchange information
        // with your GUI or audio thread in this callback.
    }
}
