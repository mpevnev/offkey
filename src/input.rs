use advanced_collections::circular_buffer::CircularBuffer;
use alsa::pcm::{Format, IO, PCM};
use rustfft::num_complex::Complex;
use rustfft::num_traits::Num;

use crate::normalize::OmniNormal;

use IOBuf::*;

/* ---------- helper macros ---------- */

macro_rules! read_more {
    ($io:ident, $scratch:ident, $buftype:ident, $buf:ident) => {{
        let read = $io.readi($scratch)?;
        $buf.extend(
            $scratch
            .iter()
            .take(read)
            .cloned()
            .map(T::from_unnormalized)
            .map(|r| Complex::new(r, T::zero()))
            );
        Ok(())
    }};
}

/* ---------- main things ---------- */

pub struct Input<'a, T> {
    source: IOBuf<'a>,
    buf: CircularBuffer<Complex<T>>,
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
        let scratchsize = 64 * period_size * num_channels;
        let src = match params.get_format()? {
            Format::S8 => I8(pcm.io_i8()?, vec![0; scratchsize]),
            Format::U8 => U8(pcm.io_u8()?, vec![0; scratchsize]),
            Format::S16LE | Format::S16BE => I16(pcm.io_i16()?, vec![0; scratchsize]),
            Format::U16LE | Format::U16BE => U16(pcm.io_u16()?, vec![0; scratchsize]),
            Format::S32LE | Format::S32BE => I32(pcm.io_i32()?, vec![0; scratchsize]),
            Format::U32LE | Format::U32BE => U32(pcm.io_u32()?, vec![0; scratchsize]),
            Format::FloatLE | Format::FloatBE => F32(pcm.io_f32()?, vec![0.0; scratchsize]),
            Format::Float64LE | Format::Float64BE => {
                F64(pcm.io_f64()?, vec![0.0; scratchsize])
            }
            _ => return Err(alsa::Error::unsupported("Unsupported sample format")),
        };
        let rate = params.get_rate()?.max(1) as usize;
        let mut buf = repeat_with(Complex::default)
            .take(2 * rate * buffer_millis / 1000 * num_channels)
            .collect::<CircularBuffer<_>>();
        buf.resize(buf.capacity());
        Ok(Input {
            source: src,
            buf,
        })
    }
}

impl<'a> IOBuf<'a> {
    pub fn read<T>(&mut self, buf: &mut CircularBuffer<Complex<T>>) -> alsa::Result<()>
        where T: OmniNormal + Num + Clone  {
        match self {
            I8(io, scratch) => read_more!(io, scratch, T, buf),
            U8(io, scratch) => read_more!(io, scratch, T, buf),
            I16(io, scratch) => read_more!(io, scratch, T, buf),
            U16(io, scratch) => read_more!(io, scratch, T, buf),
            I32(io, scratch) => read_more!(io, scratch, T, buf),
            U32(io, scratch) => read_more!(io, scratch, T, buf),
            F32(io, scratch) => read_more!(io, scratch, T, buf),
            F64(io, scratch) => read_more!(io, scratch, T, buf),
        }
    }
}

impl<'a, T: OmniNormal + Num + Clone> Input<'a, T> {
    pub fn read(&mut self) -> alsa::Result<()> {
        self.source.read(&mut self.buf)
    }
}

impl<'a, T: Clone> Input<'a, T> {
    pub fn get_frames(&self) -> Vec<Complex<T>> {
        self.buf.iter().cloned().collect()
    }
}
