//! The plugin's digital signal processing is fully implemented within this module.
//!
//! All updates to input parameters are received through message passing to avoid thread locking
//! during audio processing. In particular, note that parameter smoothing is considered within the
//! scope of audio processing rather than state management.

use std::sync::mpsc::Receiver;
use vst::buffer::AudioBuffer;
use crate::plugin_state::StateUpdate;

pub mod convolution;
use convolution::Convolver;

/// Handles all audio processing algorithms for the plugin.
pub(super) struct PluginDsp {
  // convolver: Convolver,
  messages_from_params: Receiver<StateUpdate>,
}

impl PluginDsp {
  pub fn new(incoming_messages: Receiver<StateUpdate>) -> Self {
    Self {
      //convolver: Convolver::new(),
      messages_from_params: incoming_messages
    }
  }

  /// Applies any incoming state update events to the audio generation algorithm, and then writes
  /// processed audio into the output buffer.
  pub fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
    //outputbuffer = self.convolver.process(inputbuffer)
  }
}

