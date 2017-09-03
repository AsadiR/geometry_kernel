
use num;
use num::FromPrimitive;
use num::ToPrimitive;
use num::bigint::BigInt;
use num::rational::{Ratio, BigRational, Rational64};
use time::PreciseTime;
use std::default::Default;

pub type Number = BigRational;

pub fn new(x : f64) -> Number
{
    BigRational::from_float(x).unwrap()
}

pub fn zero() -> Number {
    num::zero()
}

pub fn new_from_f32(x : f32) -> Number
{
    let res = BigRational::from_float(x).unwrap();
    return res
}

pub trait NumberTrait {
    fn to_f32(self) -> f32;
    fn abs(self) -> Number;
}

impl NumberTrait for Number {
    fn to_f32(self) -> f32 {
        (self.numer().to_f32().unwrap() / self.denom().to_f32().unwrap())
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