pub trait Normal<T> {
    fn normalize(self) -> T;
}

pub trait NormalTarget<T> {
    fn from_unnormalized(unnorm: T) -> Self;
}

pub trait OmniNormal:
    NormalTarget<i8>
    + NormalTarget<i16>
    + NormalTarget<i32>
    + NormalTarget<u8>
    + NormalTarget<u16>
    + NormalTarget<u32>
    + NormalTarget<f32>
    + NormalTarget<f64>
{
}

impl<A: Normal<B>, B> NormalTarget<A> for B {
    fn from_unnormalized(un: A) -> Self {
        un.normalize()
    }
}

impl<T> OmniNormal for T where
    T: NormalTarget<i8>
        + NormalTarget<i16>
        + NormalTarget<i32>
        + NormalTarget<u8>
        + NormalTarget<u16>
        + NormalTarget<u32>
        + NormalTarget<f32>
        + NormalTarget<f64>
{}

macro_rules! impl_for_int_lossless {
    ($integer:ident, $($float:ident),*) => {
        $(
            impl Normal<$float> for $integer {
                fn normalize(self) -> $float {
                    let min = $float::from($integer::min_value());
                    let max = $float::from($integer::max_value());
                    let s = $float::from(self);
                    (s - min) / (max - min)
                }
            }
        )*
    };
}

macro_rules! impl_for_int_lossy {
    ($integer:ident, $($float:ident),*) => {
        $(
            impl Normal<$float> for $integer {
                fn normalize(self) -> $float {
                    let min = $integer::min_value() as $float;
                    let max = $integer::max_value() as $float;
                    let s = self as $float;
                    (s - min) / (max - min)
                }
            }
        )*
    }
}

macro_rules! impl_for_float_identity {
    ($($float:ident),*) => {
        $(
            impl Normal<$float> for $float {
                fn normalize(self) -> $float {
                    self
                }
            }
        )*
    }
}

impl_for_int_lossless!(i8, f32, f64);
impl_for_int_lossless!(u8, f32, f64);
impl_for_int_lossless!(i16, f32, f64);
impl_for_int_lossless!(u16, f32, f64);
impl_for_int_lossless!(i32, f64);
impl_for_int_lossless!(u32, f64);

impl_for_int_lossy!(i32, f32);
impl_for_int_lossy!(u32, f32);

impl_for_float_identity!(f32, f64);

impl Normal<f32> for f64 {
    fn normalize(self) -> f32 {
        self as f32
    }
}

impl Normal<f64> for f32 {
    fn normalize(self) -> f64 {
        f64::from(self)
    }
}
