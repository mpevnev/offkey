use advanced_collections::circular_buffer::CircularBuffer;
use alsa::pcm::{Format, IO, PCM};
use nix::errno::Errno;
use rustfft::num_complex::Complex;
use rustfft::num_traits::{Num, NumCast};

use Source::*;

/* ---------- helper macros ---------- */

macro_rules! read_more {
    ($io:ident, $scratch:ident, $buf:ident) => {{
        $scratch.resize_with($buf.capacity(), Default::default);
        dbg!($scratch.len());
        dbg!($scratch.capacity());
        match $io.readi($scratch) {
            Err(err) if err.errno() != Some(Errno::EAGAIN) => return Err(err),
            _ => { },
        };
        $buf.extend(
            $scratch
                .iter()
                .cloned()
                .map(NumCast::from)
                .map(Option::unwrap),
        );
        dbg!($buf.len());
        dbg!($buf.capacity());
        Ok(())
    }};
}

/* ---------- main things ---------- */

pub struct Input<'a, T> {
    source: Source<'a>,
    buf: CircularBuffer<Complex<T>>,
}

pub enum Source<'a> {
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
    pub fn from_pcm(pcm: &'a PCM, bufsize: usize) -> alsa::Result<Self> {
        let params = pcm.hw_params_current()?;
        let bufsize = bufsize * (params.get_channels()? as usize);
        let src = match params.get_format()? {
            Format::S8 => I8(pcm.io_i8()?, Vec::new()),
            Format::U8 => U8(pcm.io_u8()?, Vec::new()),
            Format::S16LE | Format::S16BE => I16(pcm.io_i16()?, Vec::new()),
            Format::U16LE | Format::U16BE => U16(pcm.io_u16()?, Vec::new()),
            Format::S32LE | Format::S32BE => I32(pcm.io_i32()?, Vec::new()),
            Format::U32LE | Format::U32BE => U32(pcm.io_u32()?, Vec::new()),
            Format::FloatLE | Format::FloatBE => F32(pcm.io_f32()?, Vec::new()),
            Format::Float64LE | Format::Float64BE => F64(pcm.io_f64()?, Vec::new()),
            _ => return Err(alsa::Error::unsupported("Unsupported sample format")),
        };
        Ok(Input {
            source: src,
            buf: CircularBuffer::new(bufsize),
        })
    }
}

impl<'a> Source<'a> {
    pub fn read<T: NumCast>(
        &mut self,
        buf: &mut CircularBuffer<T>,
    ) -> alsa::Result<()> {
        match self {
            I8(io, scratch) => read_more!(io, scratch, buf),
            U8(io, scratch) => read_more!(io, scratch, buf),
            I16(io, scratch) => read_more!(io, scratch, buf),
            U16(io, scratch) => read_more!(io, scratch, buf),
            I32(io, scratch) => read_more!(io, scratch, buf),
            U32(io, scratch) => read_more!(io, scratch, buf),
            F32(io, scratch) => read_more!(io, scratch, buf),
            F64(io, scratch) => read_more!(io, scratch, buf),
        }
    }
}

impl<'a, T: NumCast + Num> Input<'a, T> {
    pub fn read(&mut self) -> alsa::Result<()> {
        self.source.read(&mut self.buf)
    }
}

impl<'a, T: Clone> Input<'a, T> {
    pub fn get_frames(&self) -> Vec<Complex<T>> {
        self.buf.iter().cloned().collect()
    }
}
