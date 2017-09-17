use num::rational::{Ratio, BigRational, Rational64};
use num;
use num::Zero as numZero;

/// This trait contains an interface for nullable structures.
pub trait Zero<T> {
    /// This method return `true` if value is a null and `false` otherwise.
    fn is_it_zero(&self) -> bool;

    /// This static method returns a null value.
    fn zero() -> T;
}

impl Zero<f32> for f32 {
    fn is_it_zero(&self) -> bool {
        *self == 0.
    }

    fn zero() -> f32 {
        0.
    }
}