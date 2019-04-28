use std::sync::Arc;

use alsa::pcm::PCM;
use ordered_float::NotNan;
use rustfft::num_complex::Complex;
use rustfft::num_traits::{Float, Num};
use rustfft::{FFTnum, FFTplanner, FFT};

use crate::input::Input;
use crate::sample::FromAnySample;

/* ---------- main things ---------- */

pub struct Analyser<'a, T> {
    fft: Arc<dyn FFT<T>>,
    fft_input: Vec<Complex<T>>,
    fft_output: Vec<Complex<T>>,
    alsa_source: Input<'a, T>,
}

impl<'a, T> Analyser<'a, T>
where
    T: FFTnum + Default,
{
    pub fn new(pcm: &'a PCM, millis_for_analysis: usize) -> alsa::Result<Self> {
        let alsa_source = Input::new(pcm, millis_for_analysis)?;
        let bufsize = alsa_source.buf_len();
        let mut planner = FFTplanner::new(false);
        Ok(Analyser {
            fft: planner.plan_fft(bufsize),
            fft_input: default_vec(bufsize),
            fft_output: default_vec(bufsize),
            alsa_source,
        })
    }
}

impl<'a, T> Analyser<'a, T> 
where
    T: FromAnySample + Num + Clone,
{
    pub fn read_data(&mut self) -> alsa::Result<()> {
        self.alsa_source.read()?;
        self.alsa_source.clone_fft_data(&mut self.fft_input);
        Ok(())
    }
}

impl<'a, T> Analyser<'a, T>
where
    T: FFTnum,
{
    pub fn do_fft(&mut self) {
        self.fft.process(&mut self.fft_input, &mut self.fft_output);
    }
}

impl<'a, T> Analyser<'a, T>
where
    T: Float,
{
    pub fn dominant_frequency(&self) -> Option<f64> {
        let index = self.fft_output.iter()
            .map(Complex::norm)
            .map(NotNan::new)
            .enumerate()
            .skip(1)
            .flat_map(|(i, norm)| norm.map(|nonnan| (i, nonnan)))
            .max_by_key(|(_, norm)| *norm)
            .map(|(i, _)| i);
        index.map(|i| self.alsa_source.frequency_at(i))
    }
}

/* ---------- helpers ---------- */

fn default_vec<T: Default>(size: usize) -> Vec<T> {
    let mut res = Vec::new();
    res.resize_with(size, T::default);
    res
}
