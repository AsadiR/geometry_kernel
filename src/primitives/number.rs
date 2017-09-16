
use num;
use num::FromPrimitive;
use num::ToPrimitive;
use num::bigint::BigInt;
use num::rational::{Ratio, BigRational, Rational64};
use time::PreciseTime;
use std::default::Default;


/// This is an alias for number type used for a geometry kernel.
pub type Number = BigRational;

/// This function creates `Number` from `f64`
/// # Arguments
///
/// * `x` - the value to convert to a `Number`.
pub fn new(x : f64) -> Number
{
    BigRational::from_float(x).unwrap()
}

/// This method return a zero `Number`.
pub fn zero() -> Number {
    num::zero()
}

/// This function creates `Number` from `f32`.
/// # Arguments
///
/// * `x` - the value to convert to a `Number`.
pub fn new_from_f32(x : f32) -> Number
{
    let res = BigRational::from_float(x).unwrap();
    return res
}

/// This trait contains a basic interface for `Number`.
pub trait NumberTrait {

    /// This method converts `Number` to `f32`.
    fn convert_to_f32(self) -> f32;
    /// This method returns an absolute value.
    fn abs(self) -> Number;
}

lazy_static! {
    static ref LEAST_F32_VALUE : Number = new(0.000001);
    static ref VALUE_10_6 : BigInt = BigInt::from_u64(1000000).unwrap();
    static ref VALUE_10 : BigInt = BigInt::from_u64(10).unwrap();
}

impl NumberTrait for Number {
    fn convert_to_f32(self) -> f32 {

        let mut numer = self.numer().clone();
        let mut denom = self.denom().clone();

        loop {
            let opt_numer = numer.to_f32();
            let opt_denom = denom.to_f32();

            if opt_denom.is_none() || opt_numer.is_none() {
                if opt_denom.is_none() && (&numer < &VALUE_10_6) {
                    return 0.;
                }

                if opt_numer.is_none() && (&denom < &VALUE_10_6) {
                    panic!("ERROR: The value is too large: {0}", self);
                }

                numer = numer / VALUE_10.clone();
                denom = denom / VALUE_10.clone();
            } else {
                return opt_numer.unwrap() / opt_denom.unwrap();
            }
        }
        panic!("Something goes wrong!");
    }

    fn abs(self) -> Number {
        num::abs(self)
    }
}



/*

pub type Number = f64;

pub fn new(x : f64) -> Number
{
    x
}


pub fn new_from_f32(x : f32) -> Number
{
    let start = PreciseTime::now();

    let res = x as f64;

    let end = PreciseTime::now();
    debug!("{} seconds for <new_from_32>", start.to(end));
    return res
}

pub fn to_f32(x : Number) -> f32 {
    x as f32
}



pub type Number = Rational64;

pub fn new(x : f64) -> Number
{
    Number::from_f64(x).unwrap()
}

pub fn new_from_f32(x : f32) -> Number
{
    let res = Number::from_f32(x).unwrap();
    return res
}

pub fn to_f32(x : Number) -> f32 {
    (*x.numer() as f32 / *x.denom() as f32)
}

*/