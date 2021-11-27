pub fn load_wav(file_name: &str) -> Vec<i16> {
  println!("loading: {}", file_name);
  let mut reader = hound::WavReader::open(file_name).unwrap();
  let samples: Vec<i16> = reader.samples().map(|s| s.unwrap()).collect();
  samples
}

fn write_wav(buffer: &[i16], file_name: &str) {
  let spec = hound::WavSpec {
    channels: 1,
    sample_rate: 44100,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int
  };
  let mut writer = hound::WavWriter::create(file_name, spec).unwrap();

  for sample in buffer {
    writer.write_sample(*sample).unwrap();
  }
}
