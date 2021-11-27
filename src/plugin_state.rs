//! Parameters are kept as the single "source of truth" for the long-term state of the plugin. As
//! used by the VST API, the parameter bank is accessible by both the audio processing thread and
//! the UI thread, and updated using thread-safe interior mutability. However, to avoid costly
//! synchronization overhead, and to reduce recalculation of derived parameters, the audio
//! processing and UI threads subscribe to parameter updates through cross-thread message passing.
//!
//! This plugin's long-term state only consists of a single floating-point value (the value of the
//! cutoff knob), but it should be simple to extend this scheme to work with multiple knobs,
//! toggles, node locations, waveforms, user-defined labels, and so on.

use std::sync::{
    mpsc::Sender,
    Mutex,
};

use vst::{
    plugin::{HostCallback, PluginParameters},
};

/// Describes a discrete operation that can update this plugin's long-term state.
#[derive(Clone)]
pub enum StateUpdate {
    SetKnob(f32),
}

#[allow(dead_code)]
pub struct PluginState {
    host: HostCallback,
    to_dsp: Mutex<Sender<StateUpdate>>,
    state_record: Mutex<Vec<f32>>,
}

/// VST-accessible long-term plugin state storage. This is accessed through the audio processing
/// thread and the UI thread, so all fields are protected by thread-safe interior mutable
/// constructs.
impl PluginState {
    pub fn new(
        host: HostCallback,
        to_dsp: Sender<StateUpdate>,
    ) -> Self {
        Self {
            host,
            to_dsp: Mutex::new(to_dsp),
            state_record: Mutex::new(vec![0.5, 0., 0., 0.]),
        }
    }
}

/// The DAW directly accesses the plugin state through the VST API to get reports on knob states.
impl PluginParameters for PluginState {
    fn set_parameter(&self, index: i32, value: f32) {
        let state_update = StateUpdate::SetKnob(value);
        self.to_dsp.lock().unwrap().send(state_update).unwrap();
        self.state_record.lock().unwrap()[index as usize] = value;
    }

    fn get_parameter(&self, index: i32) -> f32 {
        self.state_record.lock().unwrap()[index as usize]
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 => "x".to_string(),
            _ => unreachable!(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!(
                "{:.2}",
                self.state_record.lock().unwrap()[index as usize]
            ),
            _ => unreachable!(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Cutoff",
            _ => unreachable!(),
        }
        .to_string()
    }

    fn string_to_parameter(&self, index: i32, text: String) -> bool {
        dbg!("Set string to parameter for {}, {}", index, &text);
        match index {
            0 => match text.parse::<f32>() {
                Ok(value) if value <= 1. && value >= 0. => {
                    self.set_parameter(index, value);
                    true
                }
                _ => false,
            },
            _ => unreachable!(),
        }
    }
}
