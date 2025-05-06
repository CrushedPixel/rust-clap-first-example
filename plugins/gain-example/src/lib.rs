//! This module declares a plugin factory
//! that is exposed behind the CLAP entry points.

mod audio_thread;
mod main_thread;

use crate::audio_thread::GainPluginProcessor;
use crate::main_thread::GainPluginMainThread;
use clack_extensions::audio_ports::PluginAudioPorts;
use clack_plugin::clack_entry;
use clack_plugin::entry::prelude::*;
use clack_plugin::entry::prelude::*;
use clack_plugin::plugin::features::AUDIO_EFFECT;
use clack_plugin::prelude::*;
use clap_wrapper_extensions::auv2::{
    PluginFactoryAsAUv2, PluginFactoryAsAUv2Wrapper, PluginInfoAsAUv2,
};
use clap_wrapper_extensions::vst3::{PluginFactoryAsVST3, PluginInfoAsVST3};
use std::ffi::CStr;

pub struct GainPlugin;

impl Plugin for GainPlugin {
    type AudioProcessor<'a> = GainPluginProcessor<'a>;
    type MainThread<'a> = GainPluginMainThread<'a>;

    /// We don't use any shared state in this example.
    ///
    /// Generally, it is preferred in Rust to communicate data between threads
    /// by passing messages through queues instead of sharing state.
    /// You can use the ringbuf crate or any other lock-free realtime-safe
    /// queue to achieve this in practice.
    type Shared<'a> = ();

    fn declare_extensions(
        builder: &mut PluginExtensions<Self>,
        _shared: Option<&Self::Shared<'_>>,
    ) {
        builder.register::<PluginAudioPorts>();
    }
}

/// Contains the CLAP, VST3 and AUv2 descriptors for a single plugin.
struct PluginInfo(
    PluginDescriptor,
    PluginInfoAsVST3<'static>,
    PluginInfoAsAUv2,
);

/// The factory exposes the plugins that can be instantiated from this binary.
pub struct GainPluginFactory {
    info_halver: PluginInfo,
    info_doubler: PluginInfo,
}

const VST3_VENDOR: &CStr = c"free-audio";
const AU_MANUFACTURER_CODE: &CStr = c"Frau";
const AU_MANUFACTURER_NAME: &CStr = c"free-audio";

// 4-char IDs for the AU descriptors
const AU_ID_HALVER: &str = "Ghlv";
const AU_ID_DOUBLER: &str = "Gdbl";

impl GainPluginFactory {
    fn new() -> Self {
        Self {
            info_halver: PluginInfo(
                PluginDescriptor::new("free-audio.clap.rust-gain-example.halver", "Gain Halver")
                    .with_features([AUDIO_EFFECT]),
                PluginInfoAsVST3::new(Some(&VST3_VENDOR), None, None),
                PluginInfoAsAUv2::new("aufx", AU_ID_HALVER),
            ),
            info_doubler: PluginInfo(
                PluginDescriptor::new("free-audio.clap.rust-gain-example.doubler", "Gain Doubler")
                    .with_features([AUDIO_EFFECT]),
                PluginInfoAsVST3::new(Some(&VST3_VENDOR), None, None),
                PluginInfoAsAUv2::new("aufx", AU_ID_DOUBLER),
            ),
        }
    }
}

/// Implements a plugin factory that exposes 2 plugins.
/// For this gain example, one plugin halves the incoming audio,
/// and the other doubles incoming audio.
impl PluginFactory for GainPluginFactory {
    fn plugin_count(&self) -> u32 {
        2
    }

    fn plugin_descriptor(&self, index: u32) -> Option<&PluginDescriptor> {
        match index {
            0 => Some(&self.info_halver.0),
            1 => Some(&self.info_doubler.0),
            _ => None,
        }
    }

    fn create_plugin<'b>(
        &'b self,
        host_info: HostInfo<'b>,
        plugin_id: &CStr,
    ) -> Option<PluginInstance<'b>> {
        // the only way in which the two exposed plugins differ
        // is the gain factor that is passed to the main thread upon creation.

        if plugin_id == self.info_halver.0.id() {
            Some(PluginInstance::new::<GainPlugin>(
                host_info,
                &self.info_halver.0,
                |_host| Ok(()),
                |host, _| GainPluginMainThread::create(host, 0.5),
            ))
        } else if plugin_id == self.info_doubler.0.id() {
            Some(PluginInstance::new::<GainPlugin>(
                host_info,
                &self.info_doubler.0,
                |_host| Ok(()),
                |host, _| GainPluginMainThread::create(host, 2.0),
            ))
        } else {
            None
        }
    }
}

impl PluginFactoryAsVST3 for GainPluginFactory {
    fn get_vst3_info(&self, index: u32) -> Option<&PluginInfoAsVST3> {
        match index {
            0 => Some(&self.info_halver.1),
            1 => Some(&self.info_doubler.1),
            _ => None,
        }
    }
}

impl PluginFactoryAsAUv2 for GainPluginFactory {
    fn get_auv2_info(&self, index: u32) -> Option<PluginInfoAsAUv2> {
        match index {
            0 => Some(self.info_halver.2),
            1 => Some(self.info_doubler.2),
            _ => None,
        }
    }
}

/// Provides the CLAP entry points by deferring to our factory.
pub struct GainPluginEntry {
    factory: PluginFactoryWrapper<GainPluginFactory>,
    factory_auv2: PluginFactoryAsAUv2Wrapper<GainPluginFactory>,
}

impl Entry for GainPluginEntry {
    fn new(_bundle_path: &CStr) -> Result<Self, EntryLoadError> {
        Ok(Self {
            factory: PluginFactoryWrapper::new(GainPluginFactory::new()),
            factory_auv2: PluginFactoryAsAUv2Wrapper::new(
                AU_MANUFACTURER_CODE,
                AU_MANUFACTURER_NAME,
                GainPluginFactory::new(),
            ),
        })
    }

    fn declare_factories<'a>(&'a self, builder: &mut EntryFactories<'a>) {
        builder
            .register_factory(&self.factory)
            .register_factory(&self.factory_auv2);
    }
}

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
pub static rust_clap_entry: EntryDescriptor = clack_entry!(GainPluginEntry);
