use rustfft::{FftPlanner, num_complex::Complex};

// convolution segment
fn find_segment_size(len_a: usize, len_b: usize) -> usize {
  return (next_power_of_two(min(len_a, len_b)) * 2) as usize;
}

//find next power of two
fn next_power_of_two(n: i16) -> i16 {
  let mut pow = 1;
  while pow < n {
    pow *= 2;
  }
  return pow
}

fn fft(signal: &[f32]) -> Vec<Complex<f32>> {
  let mut planner = FftPlanner::<f32>::new();
  let fft = planner.plan_fft_forward(FFT_SIZE);

  let mut buffer: Vec<Complex<f32>> = Vec::new();
  for sample in signal {
    buffer.push(Complex{ re: sample, im: 0.0 })
  }

  fft.process(&mut buffer);
  buffer
}

fn ifft(fft_buffer: &[Complex<f32>]) -> Vec<f32> {
  let mut planner = FftPlanner::<f32>::new();
  let ifft = planner.plan_fft_inverse(FFT_SIZE);

  fft.process(&mut fft_buffer);

  let mut buffer: Vec<f32> = Vec::new();
  for sample in fft_buffer {
    buffer.push(sample)
  }
}

