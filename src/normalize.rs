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

macro_rules! impl_for_int {
    ($integer:ident, $float:ident) => {
        impl Normal<$float> for $integer {
            fn normalize(self) -> $float {
                let min = $integer::min_value() as $float;
                let max = $integer::max_value() as $float;
                let s = self as $float;
                (s - min) / (max - min)
            }
        }
    };
}

macro_rules! impl_for_float {
    ($source:ident, $target:ident) => {
        impl Normal<$target> for $source {
            fn normalize(self) -> $target {
                self as $target
            }
        }
    };
}

macro_rules! impl_both_for_int {
    ($integer:ident) => {
        impl_for_int!($integer, f32);
        impl_for_int!($integer, f64);
    };
}

impl_both_for_int!(i8);
impl_both_for_int!(i16);
impl_both_for_int!(i32);
impl_both_for_int!(u8);
impl_both_for_int!(u16);
impl_both_for_int!(u32);

impl_for_float!(f32, f64);
impl_for_float!(f32, f32);
impl_for_float!(f64, f32);
impl_for_float!(f64, f64);
