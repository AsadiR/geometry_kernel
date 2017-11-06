// use num::rational::{Ratio, BigRational, Rational64};
// use num;
// use num::Signed as numSigned;

/// This trait contains an interface for signed structures.
pub trait Signed<T> {
    /// This method returns `true` if the value is positive and `false` otherwise.
    fn is_it_positive(&self) -> bool;
    /// This method returns `true` if the value is positive and `false` otherwise.
    fn is_it_negative(&self) -> bool;
}



impl Signed<f32> for f32 {
    fn is_it_positive(&self) -> bool {
        *self > 0.
    }

    fn is_it_negative(&self) -> bool {
        *self < 0.
    }
}