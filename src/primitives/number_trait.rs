
use primitives::*;
use std::fmt::Debug;
use std::hash::{Hash};
use std::ops::{Add, Sub, Mul, Div /*, Neg*/};
use std::cmp::Ord;
// use std::fmt;


/// This trait contains a basic interface for `Number`.
pub trait NumberTrait <T>
    where
    T:
    Clone + Debug + Hash + Ord +
    Signed<T> + Zero<T> +
    Add<T> + Sub<T> + Mul<T> + Div<T> +
    for<'b>  Add<&'b T> + for<'b>  Sub<&'b T> + for<'b>  Mul<&'b T> + for<'b>  Div<&'b T>,
    for<'b> &'b T:
    Add<T> + Sub<T> + Mul<T> + Div<T> +
    Add<&'b T> + Sub<&'b T> + Mul<&'b T> + Div<&'b T>
{
    /// This method converts `T` to `f32`.
    fn convert_to_f32(self) -> f32;

    /// This method returns an absolute value.
    fn abs(self) -> T;

    /// This function creates `T` from `f32`.
    /// # Arguments
    ///
    /// * `x` - the value to convert to `T`.
    fn new_from_f32(x : f32) -> T;

    /// This function creates `T` from `f64`
    /// # Arguments
    ///
    /// * `x` - the value to convert to `T`.
    fn new(x : f64) -> T;

    /// This function calculates an approximation for cos
    fn approx_cos(&self, n: usize) -> T;
    /// This function calculates an approximation for sin
    fn approx_sin(&self, n: usize) -> T;

    /// This function calculates k-th power
    fn pow(&self, k: usize) -> T;
}


#[derive (Debug, Clone)]
pub struct NumberT<T> {
    pub value : T
}


impl<T> NumberT<T> {
    pub fn from_value(value: T) -> NumberT<T> {
        NumberT {
            value: value
        }
    }
}
