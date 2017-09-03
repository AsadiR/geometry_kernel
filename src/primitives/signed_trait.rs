use num::rational::{Ratio, BigRational, Rational64};
use num;
use num::Signed as numSigned;

pub trait Signed<T> {
    fn is_it_positive(&self) -> bool;
    fn is_it_negative(&self) -> bool;
}

impl Signed<BigRational> for BigRational {
    fn is_it_positive(&self) -> bool {
        self.is_positive()
    }
    fn is_it_negative(&self) -> bool {
        self.is_negative()
    }
}

impl Signed<f32> for f32 {
    fn is_it_positive(&self) -> bool {
        *self > 0.
    }

    fn is_it_negative(&self) -> bool {
        *self < 0.
    }
}