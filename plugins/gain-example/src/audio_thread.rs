//! This module handles all CLAP callbacks that run on the audio thread.

use crate::main_thread::GainPluginMainThread;
use clack_plugin::prelude::*;

pub struct GainPluginProcessor<'a> {
    #[allow(dead_code)] // unused in example
    host: HostAudioProcessorHandle<'a>,

    /// The constant factor to multiply incoming samples with.
    factor: f32,
}

impl<'a> PluginAudioProcessor<'a, (), GainPluginMainThread<'a>> for GainPluginProcessor<'a> {
    fn activate(
        host: HostAudioProcessorHandle<'a>,
        main_thread: &mut GainPluginMainThread<'a>,
        _shared: &'a (),
        _audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        // in a real plugin, you might set up
        // communication lines with the main thread here.
        Ok(Self {
            host,
            factor: main_thread.factor,
        })
    }

    fn deactivate(self, _main_thread: &mut GainPluginMainThread<'a>) {
        // here's where you tear down communications with the main thread.
    }

    /// This is where the DSP happens!
    /// This example plugin simply multiplies
    /// the amplitude of the incoming signal with a constant factor.
    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        _events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        for mut port_pair in &mut audio {
            let Some(channel_pairs) = port_pair.channels()?.into_f32() else {
                continue;
            };

            for pair in channel_pairs {
                if let ChannelPair::InputOutput(input, output) = pair {
                    for i in 0..input.len() {
                        output[i] = input[i] * self.factor;
                    }
                }
            }
        }

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }
}
