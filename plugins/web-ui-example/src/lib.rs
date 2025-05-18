mod audio_thread;
mod directories;
mod gui;
mod main_thread;

use crate::audio_thread::WebUiPluginProcessor;
use crate::main_thread::WebUiPluginMainThread;
use clack_extensions::gui::PluginGui;
use clack_plugin::clack_entry;
use clack_plugin::plugin::features::AUDIO_EFFECT;
use clack_plugin::prelude::*;

pub struct WebUiPlugin;

impl Plugin for WebUiPlugin {
    type AudioProcessor<'a> = WebUiPluginProcessor<'a>;
    type Shared<'a> = ();
    type MainThread<'a> = WebUiPluginMainThread<'a>;

    fn declare_extensions(
        builder: &mut PluginExtensions<Self>,
        _shared: Option<&Self::Shared<'_>>,
    ) {
        builder.register::<PluginGui>();
    }
}

impl DefaultPluginFactory for WebUiPlugin {
    fn get_descriptor() -> PluginDescriptor {
        PluginDescriptor::new("free-audio.clap.rust-web-ui-example", "Web UI Example")
            .with_features([AUDIO_EFFECT])
    }

    fn new_shared(_host: HostSharedHandle) -> Result<Self::Shared<'_>, PluginError> {
        Ok(())
    }

    fn new_main_thread<'a>(
        host: HostMainThreadHandle<'a>,
        _shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        WebUiPluginMainThread::create(host)
    }
}

// TODO: AUv2 factory

/// Expose the CLAP entry point,
/// but notably under a non-standard symbol name,
/// i.e. "rust_clap_entry" instead of "clap_entry"!
///
/// When building the final plug-ins with clap-wrapper,
/// the C++ rust_clap_entry.cpp file links against the static library built from this crate.
/// and re-exports this entry under the expected "clap_entry" symbol name.
#[allow(non_upper_case_globals, missing_docs)]
#[allow(unsafe_code)]
#[allow(warnings, unused)]
#[unsafe(no_mangle)]
pub static rust_clap_entry: EntryDescriptor = clack_entry!(SinglePluginEntry<WebUiPlugin>);
