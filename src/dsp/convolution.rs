use std::collections::VecDeque;
use std::f32::consts::PI;
use rustfft::num_complex::Complex;

/*
Typically the best option here is a Segmented Overlap Add or Block Convolver. This works roughly like this

- Break your 8k impulse response into 16 chunks of 512 each. Zero pad to 1024 and FFT. Now you have 16 frequency domain transfer function vectors 𝐻𝑘(𝑧),𝑘=0,1,2...15
- At every frame n, you get a new signal block 𝑥𝑛(𝑡) of length 512. Zero pad to 1024 and FFT. Now have the frequency domain signal vector 𝑋𝑛(𝑧). Make sure you keep the last 15 frames around as well, so you have 𝑋𝑛(𝑧),𝑋𝑛−1(𝑧)...𝑋𝑛−15(𝑧)
- Now multiply the signal vectors with the transfer function vectors and sum up all the results. 𝑌𝑛(𝑧)=𝑋𝑛(𝑧)⋅𝐻0(𝑧)+𝑋𝑛−1(𝑧)⋅𝐻1(𝑧)+...+𝑋𝑛−15(𝑧)⋅𝐻15(𝑧)
- Inverse FFT, you get 1024 time domain samples, 𝑦𝑛(𝑡)
- Manage the overlap. Create the output as the first 512 samples of 𝑦𝑛(𝑡) of plus the last 512 from the previous frame 𝑦𝑛−1(𝑡). Keep the last 512 samples from the current frame, 𝑦𝑛(𝑡), as overlap for the next frame.
*/

struct Convolver {
  segment_size: usize,
  input_signal: &[f32],
  ir_signal: &[f32],
  segment_size: usize,
  ir_segments: Vec<Vec<Complex<f32>>>,
  previous_frames: VecDeque<Complex<f32>>> // previous freq domain input signals
}

//const FFT_SIZE = 1024
/*
Setup IR
  - segment len is 1/2 fft_size
  - segment IR buffer (pad with 0s to be fft_size)
  - FFT and hold onto each IR segment
Setup frame history Queue
  - queue for previous input frame buffers
  - len is same as # of IR segments
  - start with 0.s
Process Input
  - segment len is 1/2 fft_size
  - segment Input buffer (pad with 0s to be fft_size)
  - keep track of # of input segments (N)
  - FFT each input segment
  - convolve each input segment with the IR and the History (frequency domain)
    - ???
    - push/pop history queue
  - take N output segments
  - IFFT output segments
  - Concat into time domain output vec
  - overlap add output with output from previous frame
  - hold on to output for overlap with next frame

  - return output vec
Convolution
*/
impl Convolver {

  // set up saved segmented IR
  pub fn new(ir_signal: &[f32], fft_size: usize) -> Self {
    segment_size = fft_size / 2;

    Self {
      segment_size: usize,
      input_signal: &[f32],
      ir_signal: &[f32],
      segment_size: usize,
      previous_frames: VecDeque<Complex<f32>>>,
      segmented_ir: Vec<Vec<Complex<f32>>>,
    }
  }
}

// - segment IR buffer (pad with 0s to be fft_size)
// - FFT and hold onto each IR segment
pub fn setup_ir(ir_signal: &[f32], fft_size: usize) -> Vec<Vec<Complex<f32>>> {
}

segmentize(list: &[f32], segment_size: usize) {
  
}
