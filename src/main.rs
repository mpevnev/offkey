#![warn(clippy::all)]

mod input;
mod mic;
mod normalize;

use alsa::pcm::PCM;
use advanced_collections::circular_buffer::CircularBuffer;
use nix::Errno;
use ordered_float::NotNan;
use rustfft::FFTplanner;
use rustfft::FFTnum;
use rustfft::num_complex::Complex;
use rustfft::num_traits::{Float, Num, Zero};

use input::Input;
use normalize::OmniNormal;
use mic::{open_microphone, MicSettings};

fn main() -> Result<(), String> {
    let set = MicSettings {
        access: Some(alsa::pcm::Access::RWInterleaved),
        ..MicSettings::default()
    };
    let mic = open_microphone("default", set)
        .map_err(|e| format!("Failed to open the microphone: {}", e))?;
    let mut input: Input<'_, f64> = Input::from_pcm(&mic, 300)
        .map_err(|e| format!("Failed to initialize input: {}", e))?;
    let mut planner = FFTplanner::new(false);
    let mut old_buckets = CircularBuffer::new(10);
    loop {
        let mut inbuf = make_frame_buffer(&mic, &mut input)?;
        let outbuf = run_fft(&mut planner, &mut inbuf);
        let max = max_index(&outbuf[1..outbuf.len() / 2], 1e-3);
        if let Some(max) = max {
            old_buckets.push_front(max);
        }
        if let Some(avg) = average_index(&old_buckets) {
            eprintln!("{}", avg);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn make_frame_buffer<'a, T>(mic: &PCM, input: &mut Input<'a, T>) -> Result<Vec<Complex<T>>, String>
where T: OmniNormal + Num + Clone 
{
    if let Err(e) = input.read() {
        match e.errno() {
            Some(Errno::EAGAIN) => { },
            Some(Errno::EPIPE) => mic.try_recover(e, true)
                .map_err(|e| format!("Failed to recover from a EPIPE error: {}", e))?,
            Some(_) => return Err(format!("Failed to read input: {}", e)),
            None => { },
        }
    };
    Ok(input.get_frames())
}

fn run_fft<T>(planner: &mut FFTplanner<T>, buf: &mut [Complex<T>]) -> Vec<Complex<T>> 
where T: FFTnum + Clone
{
    let mut outbuf = vec![Complex::zero(); buf.len()];
    let fft = planner.plan_fft(buf.len());
    fft.process(buf, &mut outbuf);
    outbuf
}

fn max_index<T>(buf: &[Complex<T>], threshold: T) -> Option<usize> 
where T: Float
{
    buf.iter()
        .map(|c| c.re.abs())
        .enumerate()
        .flat_map(|(i, f)| NotNan::new(f).map(|nnan| (i, nnan)))
        .filter(|(_, f)| f.abs() > threshold)
        .max_by_key(|(_, f)| *f)
        .map(|(i, _)| i)
}

fn average_index(buf: &CircularBuffer<usize>) -> Option<usize> {
    buf.iter().sum::<usize>().checked_div(buf.len())
}
