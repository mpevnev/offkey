mod input;
mod mic;

use rustfft::FFTplanner;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

use input::Input;
use mic::{open_microphone, MicSettings};

fn main() -> Result<(), String> {
    let set = MicSettings {
        access: Some(alsa::pcm::Access::RWInterleaved),
        ..MicSettings::default()
    };
    let mic = open_microphone("default", set)
        .map_err(|e| format!("Failed to open the microphone: {}", e))?;
    let mut input: Input<'_, f64> = Input::from_pcm(&mic, 1024)
        .map_err(|e| format!("Failed to initialize input: {}", e))?;
    input.read()
        .map_err(|e| format!("Failed to read samples: {}", e))?;
    let mut inbuf = input.get_frames();
    dbg!(inbuf.len());
    let mut outbuf = vec![Complex::zero(); inbuf.len()];
    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(inbuf.len());
    fft.process(&mut inbuf, &mut outbuf);

    for item in outbuf.iter() {
        println!("{}", item);
    }

    Ok(())
}
