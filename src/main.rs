#![warn(clippy::all)]

mod input;
mod mic;
mod normalize;

use std::env::args;
use std::sync::Arc;

use advanced_collections::circular_buffer::CircularBuffer;
use alsa::pcm::PCM;
use nix::Errno;
use ordered_float::NotNan;
use rustfft::num_complex::Complex;
use rustfft::num_traits::{Float, Num, Zero};
use rustfft::FFTnum;
use rustfft::{FFTplanner, FFT};

use input::Input;
use mic::{open_microphone, MicSettings};
use normalize::OmniNormal;

fn main() -> Result<(), String> {
    let device_name = args().nth(1).ok_or("usage: offkey <input device>")?;
    let set = MicSettings {
        access: Some(alsa::pcm::Access::RWInterleaved),
        ..MicSettings::default()
    };
    let mic = open_microphone(&device_name, set)
        .map_err(|e| format!("Failed to open the microphone: {}", e))?;
    let mut input: Input<'_, f64> = Input::from_pcm(&mic, 300)
        .map_err(|e| format!("Failed to initialize input: {}", e))?;
    let fft = make_fft(&input);
    let mut fft_output = vec![Complex::zero(); input.buf_len()];
    let mut old_buckets = CircularBuffer::new(10);
    loop {
        let mut inbuf = make_frame_buffer(&mic, &mut input)?;
        fft.process(&mut inbuf, &mut fft_output);
        let max = max_index(&fft_output[0..fft_output.len() / 2], 1e-3);
        if let Some(max) = max {
            old_buckets.push_front(max);
        }
        if let Some(avg) = average_index(&old_buckets) {
            eprintln!("{}", input.frequency_at(avg));
            println!("{}", avg);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn make_frame_buffer<'a, T>(
    mic: &PCM,
    input: &mut Input<'a, T>,
) -> Result<Vec<Complex<T>>, String>
where
    T: OmniNormal + Num + Clone,
{
    if let Err(e) = input.read() {
        match e.errno() {
            Some(Errno::EAGAIN) => {}
            Some(Errno::EPIPE) => mic
                .try_recover(e, true)
                .map_err(|e| format!("Failed to recover from a EPIPE error: {}", e))?,
            Some(_) => return Err(format!("Failed to read input: {}", e)),
            None => {}
        }
    };
    Ok(input.get_frames())
}

fn make_fft<T>(input: &Input<'_, T>) -> Arc<dyn FFT<T>>
where
    T: FFTnum,
{
    let mut planner = FFTplanner::new(false);
    planner.plan_fft(input.buf_len())
}

fn max_index<T>(buf: &[Complex<T>], threshold: T) -> Option<usize>
where
    T: Float,
{
    buf.iter()
        .map(Complex::norm)
        .enumerate()
        // The first number is not related to frequencies, so is not needed.
        .skip(1)
        .flat_map(|(i, f)| NotNan::new(f).map(|nnan| (i, nnan)))
        .filter(|(_, f)| f.into_inner() > threshold)
        .max_by_key(|(_, f)| *f)
        .map(|(i, _)| i)
}

fn average_index(buf: &CircularBuffer<usize>) -> Option<usize> {
    buf.iter().sum::<usize>().checked_div(buf.len())
}
