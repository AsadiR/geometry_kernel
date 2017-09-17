
use num;
use num::Signed as NumSigned;
use num::Zero as NumZero;

use num::FromPrimitive;
use num::ToPrimitive;
use num::bigint::BigInt;
use num::rational::{Ratio, BigRational, Rational64};
use time::PreciseTime;
use std::default::Default;
use std::fmt;
use std::hash::{Hash, Hasher};

use primitives::signed_trait::Signed;
use primitives::zero_trait::Zero;
use std::cmp::{Ord, Ordering};

use primitives::number_trait::{NumberTrait, NumberT};

use std::ops::{Add, Sub, Mul, Div, Neg};

pub type Number = NumberT<BigRational>;

lazy_static! {
    static ref LEAST_F32_VALUE : Number = Number::new(0.000001);
    static ref VALUE_10_6 : BigInt = BigInt::from_u64(1000000).unwrap();
    static ref VALUE_10 : BigInt = BigInt::from_u64(10).unwrap();
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Neg for Number {
    type Output = Number;
    fn neg(self) -> Number {
        Number::from_value(-&self.value)
    }
}

impl<'a> Neg for &'a Number {
    type Output = Number;
    fn neg(self) -> Number {
        Number::from_value(-&self.value)
    }
}

impl Add<Number> for Number {
    type Output = Number;

    fn add(self, other: Number) -> Number {
        Number::from_value(self.value + other.value)
    }
}

impl<'a,'b> Add<&'b Number> for &'a Number {
    type Output = Number;

    fn add(self, other: &'b Number) -> Number {
        Number::from_value(&self.value + &other.value)
    }
}

impl<'a,'b> Add<&'b Number> for Number {
    type Output = Number;

    fn add(self, other: &'b Number) -> Number {
        Number::from_value(&self.value + &other.value)
    }
}

impl<'a> Add<Number> for &'a Number {
    type Output = Number;

    fn add(self, other: Number) -> Number {
        Number::from_value(&self.value + other.value)
    }
}

impl Sub<Number> for Number {
    type Output = Number;

    fn sub(self, other: Number) -> Number {
        Number::from_value(self.value - other.value)
    }
}

impl<'a,'b> Sub<&'b Number> for &'a Number {
    type Output = Number;

    fn sub(self, other: &'b Number) -> Number {
        Number::from_value(&self.value - &other.value)
    }
}

impl<'a,'b> Sub<&'b Number> for Number {
    type Output = Number;

    fn sub(self, other: &'b Number) -> Number {
        Number::from_value(&self.value - &other.value)
    }
}

impl<'a> Sub<Number> for &'a Number {
    type Output = Number;

    fn sub(self, other: Number) -> Number {
        Number::from_value(&self.value - other.value)
    }
}

impl Mul<Number> for Number {
    type Output = Number;

    fn mul(self, other: Number) -> Number {
        Number::from_value(self.value * other.value)
    }
}

impl<'a,'b> Mul<&'b Number> for &'a Number {
    type Output = Number;

    fn mul(self, other: &'b Number) -> Number {
        Number::from_value(&self.value * &other.value)
    }
}

impl<'a,'b> Mul<&'b Number> for Number {
    type Output = Number;

    fn mul(self, other: &'b Number) -> Number {
        Number::from_value(&self.value * &other.value)
    }
}

impl<'a> Mul<Number> for &'a Number {
    type Output = Number;

    fn mul(self, other: Number) -> Number {
        Number::from_value(&self.value * other.value)
    }
}


impl Div<Number> for Number {
    type Output = Number;

    fn div(self, other: Number) -> Number {
        Number::from_value(self.value / other.value)
    }
}

impl<'a,'b> Div<&'b Number> for &'a Number {
    type Output = Number;

    fn div(self, other: &'b Number) -> Number {
        Number::from_value(&self.value / &other.value)
    }
}

impl<'a,'b> Div<&'b Number> for Number {
    type Output = Number;

    fn div(self, other: &'b Number) -> Number {
        Number::from_value(self.value / &other.value)
    }
}

impl<'a> Div<Number> for &'a Number {
    type Output = Number;

    fn div(self, other: Number) -> Number {
        Number::from_value(&self.value / other.value)
    }
}

impl Signed<Number> for Number {
    fn is_it_positive(&self) -> bool {
        self.value.is_positive()
    }
    fn is_it_negative(&self) -> bool {
        self.value.is_negative()
    }
}


impl Zero<Number> for Number {
    fn is_it_zero(&self) -> bool {
        self.value.is_zero()
    }

    fn zero() -> Number {
        Number::from_value(NumZero::zero())
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Number) -> bool {
        self.value == other.value
    }
}

impl Eq for Number {}

impl Ord for Number {
    fn cmp(&self, other: &Number) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Number) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for NumberT<BigRational> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

impl NumberTrait<Number> for Number {
    fn convert_to_f32(self) -> f32 {

        let mut numer = self.value.numer().clone();
        let mut denom = self.value.denom().clone();

        loop {
            let opt_numer = numer.to_f32();
            let opt_denom = denom.to_f32();

            if opt_denom.is_none() || opt_numer.is_none() {
                if opt_denom.is_none() && (&numer < &VALUE_10_6) {
                    return 0.;
                }

                if opt_numer.is_none() && (&denom < &VALUE_10_6) {
                    panic!("ERROR: The value is too large: {0}", self.value);
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
        Number::from_value(num::abs(self.value))
    }


    fn new(x : f64) -> Number
    {
        Number::from_value(BigRational::from_float(x).unwrap())
    }


    fn new_from_f32(x : f32) -> Number
    {
        let res = BigRational::from_float(x).unwrap();
        return Number::from_value(res);
    }
}

