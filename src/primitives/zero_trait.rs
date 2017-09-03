use num::rational::{Ratio, BigRational, Rational64};
use num;
use num::Zero as numZero;

pub trait Zero<T> {
    fn is_it_zero(&self) -> bool;
    fn zero() -> T;
}

impl Zero<BigRational> for BigRational {
    fn is_it_zero(&self) -> bool {
        self.is_zero()
    }

    fn zero() -> BigRational {
        num::zero()
    }
}

impl Zero<f32> for f32 {
    fn is_it_zero(&self) -> bool {
        *self == 0.
    }

    fn zero() -> f32 {
        0.
    }
}