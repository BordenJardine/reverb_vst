mod wav;
use wav::load_wav;

fn main() {
  let impulse_response = load_wav("src/assets/spring_impulse.wav");
  print_impulse(&impulse_response);
}

fn print_impulse(impulse_response: &[f32]) {
  print!("pub const SPRING_IMPULSE_RESPONSE: &[f32] = &[");
  for sample in impulse_response {
    print!("{:.7},", sample);
  }
  println!("];");
}
