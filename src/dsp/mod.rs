use std::sync::mpsc::Receiver;
use vst::buffer::AudioBuffer;
use crate::plugin_state::StateUpdate;

pub mod convolution;
use convolution::Convolver;

pub mod spring_impulse_response;
use spring_impulse_response::SPRING_IMPULSE_RESPONSE;

/// Entry point for audio processing algorithms for the plugin.
pub(super) struct PluginDsp {
  convolver_l: Convolver,
  convolver_r: Convolver,
  messages_from_params: Receiver<StateUpdate>,
}

const FFT_SIZE: usize = 1024;

impl PluginDsp {
  pub fn new(incoming_messages: Receiver<StateUpdate>) -> Self {
    let impulse_response = &SPRING_IMPULSE_RESPONSE;
    Self {
      convolver_l: Convolver::new(&impulse_response, FFT_SIZE),
      convolver_r: Convolver::new(&impulse_response, FFT_SIZE),
      messages_from_params: incoming_messages
    }
  }

  /// Applies any incoming state update events to the audio generation algorithm, and then writes
  /// processed audio into the output buffer.
  pub fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
    let (inputs, mut outputs) = buffer.split();

    //TODO: DRY
    let devisor = 5000.;
    for (output_sample, result_sample) in outputs[0].iter_mut().zip(self.convolver_l.process(&inputs[0]).iter()) {
      *output_sample = *result_sample / devisor;
    }
    for (output_sample, result_sample) in outputs[1].iter_mut().zip(self.convolver_r.process(&inputs[1]).iter()) {
      *output_sample = *result_sample / devisor;
    }
  }
}

