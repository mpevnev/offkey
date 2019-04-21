use advanced_collections::circular_buffer::CircularBuffer;
use alsa::pcm::{Format, IO, PCM};
use itertools::Itertools;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Num;

use crate::sample::{FromAnySample, FromSample};

use IOBuf::*;

/* ---------- main things ---------- */

pub struct Input<'a, T> {
    source: IOBuf<'a>,
    buf: CircularBuffer<Complex<T>>,
    num_channels: usize,
    sample_frequency: f64,
}

pub enum IOBuf<'a> {
    I8(IO<'a, i8>, Vec<i8>),
    U8(IO<'a, u8>, Vec<u8>),
    I16(IO<'a, i16>, Vec<i16>),
    U16(IO<'a, u16>, Vec<u16>),
    I32(IO<'a, i32>, Vec<i32>),
    U32(IO<'a, u32>, Vec<u32>),
    F32(IO<'a, f32>, Vec<f32>),
    F64(IO<'a, f64>, Vec<f64>),
}

impl<'a, T: Default> Input<'a, T> {
    pub fn from_pcm(pcm: &'a PCM, buffer_millis: usize) -> alsa::Result<Self> {
        use std::iter::repeat_with;
        let params = pcm.hw_params_current()?;
        let period_size = params.get_period_size()?.max(1) as usize;
        let num_channels = params.get_channels()?.max(1) as usize;
        let scratchsize = period_size * num_channels;
        let src = match params.get_format()? {
            Format::S8 => I8(pcm.io_i8()?, vec![0; scratchsize]),
            Format::U8 => U8(pcm.io_u8()?, vec![0; scratchsize]),
            Format::S16LE | Format::S16BE => I16(pcm.io_i16()?, vec![0; scratchsize]),
            Format::U16LE | Format::U16BE => U16(pcm.io_u16()?, vec![0; scratchsize]),
            Format::S32LE | Format::S32BE => I32(pcm.io_i32()?, vec![0; scratchsize]),
            Format::U32LE | Format::U32BE => U32(pcm.io_u32()?, vec![0; scratchsize]),
            Format::FloatLE | Format::FloatBE => {
                F32(pcm.io_f32()?, vec![0.0; scratchsize])
            }
            Format::Float64LE | Format::Float64BE => {
                F64(pcm.io_f64()?, vec![0.0; scratchsize])
            }
            _ => return Err(alsa::Error::unsupported("Unsupported sample format")),
        };
        let rate = params.get_rate()?.max(1) as usize;
        let buf = repeat_with(Complex::default)
            .take(rate * buffer_millis / 1000)
            .collect::<CircularBuffer<_>>();
        Ok(Input {
            source: src,
            buf,
            num_channels,
            sample_frequency: rate as f64,
        })
    }
}

impl<'a, T: FromAnySample + Num + Clone> Input<'a, T> {
    pub fn read(&mut self) -> alsa::Result<()> {
        let numch = self.num_channels;
        match &mut self.source {
            I8(io, scratch) => read_into_buf(&mut self.buf, io, scratch, numch),
            U8(io, scratch) => read_into_buf(&mut self.buf, io, scratch, numch),
            I16(io, scratch) => read_into_buf(&mut self.buf, io, scratch, numch),
            U16(io, scratch) => read_into_buf(&mut self.buf, io, scratch, numch),
            I32(io, scratch) => read_into_buf(&mut self.buf, io, scratch, numch),
            U32(io, scratch) => read_into_buf(&mut self.buf, io, scratch, numch),
            F32(io, scratch) => read_into_buf(&mut self.buf, io, scratch, numch),
            F64(io, scratch) => read_into_buf(&mut self.buf, io, scratch, numch),
        }
    }
}

impl<'a, T: Clone> Input<'a, T> {
    pub fn get_frames(&self) -> Vec<Complex<T>> {
        self.buf.iter().cloned().collect()
    }
}

impl<'a, T> Input<'a, T> {
    pub fn buf_len(&self) -> usize {
        self.buf.len()
    }

    pub fn frequency_at(&self, index: usize) -> f64 {
        let index = index as f64;
        let len = self.buf_len() as f64;
        index * self.sample_frequency / len
    }
}

/* ---------- helpers ---------- */

fn read_into_buf<I, T>(
    buf: &mut CircularBuffer<Complex<T>>,
    io: &IO<'_, I>,
    scratch: &mut [I],
    num_channels: usize,
) -> alsa::Result<()>
where
    I: Copy,
    T: FromSample<I> + Num + Clone,
{
    let read = dbg!(io.readi(scratch))?;
    buf.extend(
        scratch
            .iter()
            .take(read)
            .cloned()
            .map(T::from_sample)
            .chunks(num_channels)
            .into_iter()
            .map(average)
            .map(|r| Complex::new(r, T::zero())),
    );
    Ok(())
}

fn average<T, I>(iter: T) -> I
where
    I: Num,
    T: Iterator<Item = I>,
{
    let mut count = I::zero();
    let mut total = I::zero();
    for v in iter {
        count = count + I::one();
        total = total + v;
    }
    total / count
}
