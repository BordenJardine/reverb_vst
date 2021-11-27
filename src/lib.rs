use std::sync::{mpsc::channel, Arc};

use vst::{
    api::Supported,
    buffer::AudioBuffer,
    plugin::{CanDo, HostCallback, Info, Plugin, PluginParameters},
};

mod dsp;
use dsp::PluginDsp;

mod plugin_state;
use plugin_state::PluginState;

/// Top level wrapper that exposes a full `vst::Plugin` implementation.
struct ReverbVst {
    /// The `PluginDsp` handles all of the plugin's audio processing, and is only accessed from the
    /// audio processing thread.
    dsp: PluginDsp,

    /// The `PluginState` holds the long-term state of the plugin and distributes raw parameter
    /// updates as they occur to other parts of the plugin. It is shared on both the audio
    /// processing thread and the UI thread, and updated using thread-safe interior mutability.
    state_handle: Arc<PluginState>,
}

impl ReverbVst {
    /// Initializes the VST plugin, along with an optional `HostCallback` handle.
    fn new_maybe_host(maybe_host: Option<HostCallback>) -> Self {
        let host = maybe_host.unwrap_or_default();

        let (to_dsp, dsp_recv) = channel();

        let state_handle = Arc::new(PluginState::new(host, to_dsp));

        let dsp = PluginDsp::new(dsp_recv);

        Self {
            dsp,
            state_handle,
        }
    }
}

/// `vst::plugin_main` requires a `Default` implementation.
impl Default for ReverbVst {
    fn default() -> Self {
        Self::new_maybe_host(None)
    }
}

/// Main `vst` plugin implementation.
impl Plugin for ReverbVst {
    fn new(host: HostCallback) -> Self {
        Self::new_maybe_host(Some(host))
    }

    fn get_info(&self) -> Info {
        /// Use a hash of a string describing this plugin to avoid unique ID conflicts.
        const UNIQUE_ID_SEED: &str = "Reverb VST2 Plugin";
        static UNIQUE_ID: once_cell::sync::Lazy<i32> = once_cell::sync::Lazy::new(|| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut s = DefaultHasher::new();
            UNIQUE_ID_SEED.hash(&mut s);
            s.finish() as i32
        });

        Info {
            name: "Reverb".to_string(),
            vendor: "Borden".to_string(),
            unique_id: *UNIQUE_ID,
            inputs: 2,
            outputs: 2,
            parameters: 0,
            initial_delay: 0,
            preset_chunks: true,
            ..Info::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        self.dsp.process(buffer);
    }

    fn can_do(&self, _can_do: CanDo) -> Supported {
        Supported::Maybe
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.state_handle) as Arc<dyn PluginParameters>
    }
}

vst::plugin_main!(ReverbVst);
