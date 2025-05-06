//! This module handles all CLAP callbacks that run on the main thread.

use clack_extensions::audio_ports::{AudioPortFlags, AudioPortInfo, AudioPortInfoWriter, AudioPortType, PluginAudioPortsImpl};
use clack_plugin::prelude::*;

pub struct GainPluginMainThread<'a> {
    #[allow(dead_code)] // unused in example
    host: HostMainThreadHandle<'a>,

    /// The constant factor to multiply incoming samples with.
    pub factor: f32,
}

impl<'a> GainPluginMainThread<'a> {
    /// Creates an instance of the plugin's main thread.
    /// This plugin will multiply the incoming signal with gain_factor.
    pub fn create(host: HostMainThreadHandle<'a>, gain_factor: f32) -> Result<Self, PluginError> {
        // this example main thread doesn't
        // do anything or hold any data
        Ok(Self { host, factor: gain_factor })
    }
}

impl<'a> PluginMainThread<'a, ()> for GainPluginMainThread<'a> {
    fn on_main_thread(&mut self) {
        // in a real plugin, you might exchange information
        // with your GUI or audio thread in this callback.
    }
}

/// This example plugin has a single input and output audio port.
/// additional ports, e.g. for sidechain inputs, would be configured here.
impl<'a> PluginAudioPortsImpl for GainPluginMainThread<'a> {
    fn count(&mut self, is_input: bool) -> u32 {
        match is_input {
            true => { 1 }
            false => { 1 }
        }
    }

    fn get(&mut self, index: u32, is_input: bool, writer: &mut AudioPortInfoWriter) {
        if index != 0 {
            return;
        }

        // input and output ports are both stereo (2 channels)
        // and 32-bit only.
        writer.set(&AudioPortInfo {
            id: ClapId::new(if is_input { 0 } else { 1 }),
            name: b"Audio port",
            channel_count: 2,
            flags: AudioPortFlags::IS_MAIN,
            port_type: Some(AudioPortType::STEREO),
            in_place_pair: None,
        });
    }
}