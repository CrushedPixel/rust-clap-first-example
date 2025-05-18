use crate::main_thread::WebUiPluginMainThread;
use clack_plugin::prelude::*;

pub struct WebUiPluginProcessor<'a> {
    #[allow(dead_code)] // unused in example
    host: HostAudioProcessorHandle<'a>,
}

impl<'a> PluginAudioProcessor<'a, (), WebUiPluginMainThread<'a>> for WebUiPluginProcessor<'a> {
    fn activate(
        host: HostAudioProcessorHandle<'a>,
        _main_thread: &mut WebUiPluginMainThread<'a>,
        _shared: &'a (),
        _audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self { host })
    }

    fn process(
        &mut self,
        _process: Process,
        _audio: Audio,
        _events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        // TODO: gain example with parameter connected to web UI
        Ok(ProcessStatus::Continue)
    }
}
