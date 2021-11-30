use std::collections::VecDeque;
use std::sync::Arc;
use rustfft::{Fft, FftPlanner, num_complex::Complex};

/*
Typically the best option here is a Segmented Overlap Add or Block Convolver. This works roughly like this

- Break your 8k impulse response into 16 chunks of 512 each. Zero pad to 1024 and FFT. Now you have 16 frequency domain transfer function vectors 𝐻𝑘(𝑧),𝑘=0,1,2...15
- At every frame n, you get a new signal block 𝑥𝑛(𝑡) of length 512. Zero pad to 1024 and FFT. Now have the frequency domain signal vector 𝑋𝑛(𝑧). Make sure you keep the last 15 frames around as well, so you have 𝑋𝑛(𝑧),𝑋𝑛−1(𝑧)...𝑋𝑛−15(𝑧)
- Now multiply the signal vectors with the transfer function vectors and sum up all the results. 𝑌𝑛(𝑧)=𝑋𝑛(𝑧)⋅𝐻0(𝑧)+𝑋𝑛−1(𝑧)⋅𝐻1(𝑧)+...+𝑋𝑛−15(𝑧)⋅𝐻15(𝑧)
- Inverse FFT, you get 1024 time domain samples, 𝑦𝑛(𝑡)
- Manage the overlap. Create the output as the first 512 samples of 𝑦𝑛(𝑡) of plus the last 512 from the previous frame 𝑦𝑛−1(𝑡). Keep the last 512 samples from the current frame, 𝑦𝑛(𝑡), as overlap for the next frame.
*/

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

pub struct Convolver {
  fft_size: usize,
  ir_segments: Vec<Vec<Complex<f32>>>, // freq domain impulse response segments
  previous_frame_q: VecDeque<Vec<Complex<f32>>>, // previous freq domain input signals
  previous_tail: Vec<f32>, // previous output frame (time domain) for overlap add
  fft_processor: Arc<dyn Fft<f32>>,
  ifft_processor: Arc<dyn Fft<f32>>, //inverse ff
}

impl Convolver {
  // set up saved segmented IR
  pub fn new(ir_signal: &[f32], fft_size: usize) -> Self {
    let mut planner = FftPlanner::<f32>::new();
    let fft_processor = planner.plan_fft_forward(fft_size);
    let ifft_processor = planner.plan_fft_inverse(fft_size);

    let ir_segments = segment_buffer(ir_signal, fft_size, &fft_processor);
    let segment_count = ir_segments.len();
    Self {
      fft_size,
      ir_segments,
      fft_processor,
      ifft_processor,
      previous_frame_q: init_previous_frame_q(segment_count, fft_size),
      previous_tail: init_previous_tail(fft_size/2),
    }
  }

  pub fn process(&mut self, input_buffer: &[f32]) -> Vec<f32> {
    let io_len = input_buffer.len();
    // segment and convert to freq domain
    let input_segments = segment_buffer(input_buffer, self.fft_size, &self.fft_processor);


    let mut output_segments: Vec<Vec<Complex<f32>>> = Vec::new();
    // push front/ pop back
    for segment in input_segments {
      self.previous_frame_q.push_front(segment);
      self.previous_frame_q.pop_back();
      // multiply
      output_segments.push(self.convolve_frame());
    }

    // go back to time domain
    let mut time_domain: Vec<f32> = Vec::new();
    for mut segment in output_segments {
      self.ifft_processor.process(&mut segment);
      for sample in segment {
        time_domain.push(sample.re);
      }
    }

    // overlap add
    for (i, sample) in self.previous_tail.iter().enumerate() {
      match time_domain.get_mut(i) {
        Some(out_sample) => *out_sample += sample,
        None => break
      }
    }

    // everything outside of the buffer length is the tail for the next run
    self.previous_tail = time_domain[io_len..time_domain.len()].to_vec();

    // return a buffers worth of signal
    return time_domain[0..io_len].to_vec();
  }
 
  // in freq domain
  // 𝑌𝑛(𝑧)=𝑋𝑛(𝑧)⋅𝐻0(𝑧)+𝑋𝑛−1(𝑧)⋅𝐻1(𝑧)+...+𝑋𝑛−15(𝑧)⋅𝐻15(𝑧)
  fn convolve_frame(&mut self) -> Vec<Complex<f32>> {
    //init output to accumulate onto
    let mut convolved: Vec<Complex<f32>> = Vec::new();
    for _ in 0..self.fft_size {
      convolved.push(Complex {re: 0. , im: 0. });
    }

    for i in 0..self.ir_segments.len() {
      add_frames(&mut convolved, mult_frames(
        &self.previous_frame_q[i],
        &self.ir_segments[i]
      ));
    }
    convolved
  }
}

// mutates the first frame!
pub fn add_frames(f1: &mut [Complex<f32>], f2: Vec<Complex<f32>>) {
  for (mut sample1, sample2) in f1.iter_mut().zip(f2) {
    sample1.re = sample1.re + sample2.re;
    sample1.im = sample1.im + sample2.im;
  }
}

//freq domain multiplication
//ReY[f] = ReX[f]ReH[f]-ImX[f]ImH[f]
//ImY[f] = ImX[f]ReH[f] + ReX[f]ImH[f]
//
// returns new vec
pub fn mult_frames(f1: &[Complex<f32>], f2: &[Complex<f32>]) -> Vec<Complex<f32>> {
  let mut out: Vec<Complex<f32>> = Vec::new();
  for (sample1, sample2) in f1.iter().zip(f2) {
    out.push(Complex {
     re: (sample1.re * sample2.re) - (sample1.im * sample2.im),
     im: (sample1.im * sample2.re) - (sample1.re * sample2.im)
    });
  }
  out
}

pub fn init_previous_tail(size: usize) -> Vec<f32> {
  let mut tail = Vec::new();
  for _ in 0..size {
    tail.push(0.);
  }
  tail
}

// - segment buffer (pad with 0s to be fft_size)
// - FFT and hold onto each segment
pub fn segment_buffer(buffer: &[f32], fft_size: usize, fft_processor: &Arc<dyn Fft<f32>>) -> Vec<Vec<Complex<f32>>> {
  let mut segments = Vec::new();
  let segment_size = fft_size / 2;

  let mut index = 0;
  while index < buffer.len() {
    let mut new_segment: Vec<Complex<f32>> = Vec::new();
    for i in index..index+segment_size {
      match buffer.get(i) {
        Some(sample) => new_segment.push(Complex { re: *sample, im: 0. }),
        None => continue
      }
    }
    while new_segment.len() < fft_size {
      new_segment.push(Complex { re: 0., im: 0. });
    }
    fft_processor.process(&mut new_segment);
    segments.push(new_segment);
    index += segment_size;
  }

  segments
}

// queue of previous input segments in the frequency domain (polar notation)
// init to 0s
pub fn init_previous_frame_q(segment_count: usize, fft_size: usize) -> VecDeque<Vec<Complex<f32>>> {
  let mut q = VecDeque::new();
  for _ in 0..segment_count {
    let mut empty = Vec::new();
    for _ in 0..fft_size {
      empty.push(Complex{ re: 0., im: 0. });
    }
    q.push_back(empty);
  }
  q
}
