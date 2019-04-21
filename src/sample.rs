pub trait FromSample<T> {
    fn from_sample(sample: T) -> Self;
}

pub trait FromAnySample:
    FromSample<i8>
    + FromSample<i16>
    + FromSample<i32>
    + FromSample<u8>
    + FromSample<u16>
    + FromSample<u32>
    + FromSample<f32>
    + FromSample<f64>
{
}

impl<T> FromAnySample for T where
    T: FromSample<i8>
        + FromSample<i16>
        + FromSample<i32>
        + FromSample<u8>
        + FromSample<u16>
        + FromSample<u32>
        + FromSample<f32>
        + FromSample<f64>
{}

macro_rules! impl_from_int_lossless {
    ($float:ident; $($integer:ident),+) => {
        $(
            impl FromSample<$integer> for $float {
                fn from_sample(sample: $integer) -> $float {
                    let min = $float::from($integer::min_value());
                    let max = $float::from($integer::max_value());
                    let s = $float::from(sample);
                    (s - min) / (max - min)
                }
            }
        )+
    }
}

macro_rules! impl_from_int_lossy {
    ($float:ident; $($integer:ident),+) => {
        $(
            impl FromSample<$integer> for $float {
                fn from_sample(sample: $integer) -> $float {
                    let min = $integer::min_value() as $float;
                    let max = $integer::max_value() as $float;
                    let s = sample as $float;
                    (s - min) / (max - min)
                }
            }
        )+
    }
}

macro_rules! impl_for_float_identity {
    ($($float:ident),+) => {
        $(
            impl FromSample<$float> for $float {
                fn from_sample(sample: $float) -> $float {
                    sample
                }
            }
        )+
    }
}

impl_from_int_lossless!(f32; i8, u8, i16, u16);
impl_from_int_lossless!(f64; i8, u8, i16, u16, i32, u32);
impl_from_int_lossy!(f32; i32, u32);
impl_for_float_identity!(f32, f64);

impl FromSample<f32> for f64 {
    fn from_sample(sample: f32) -> f64 {
        f64::from(sample)
    }
}

impl FromSample<f64> for f32 {
    fn from_sample(sample: f64) -> f32 {
        sample as f32
    }
}
