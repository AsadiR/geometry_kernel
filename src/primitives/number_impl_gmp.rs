use gmp::mpq::Mpq;
use gmp::mpz::Mpz;

// use time::PreciseTime;
// use std::default::Default;
use std::fmt;
use std::hash::{Hash, Hasher};
use num::pow;

use primitives::signed_trait::Signed;
use primitives::zero_trait::Zero;
use std::cmp::{Ord, Ordering};
use std::collections::HashMap;

use primitives::number_trait::{NumberTrait, NumberT};

use std::ops::{Add, Sub, Mul, Div, Neg};

pub type Number = NumberT<Mpq>;

lazy_static! {
    static ref ZERO_VALUE : Number = Number::new(0.);
    // static ref LEAST_F32_VALUE : Number = Number::new(0.000001);
    static ref VALUE_10_6 : Mpz = Mpz::from_str_radix("100000", 10).unwrap();
    static ref VALUE_10 : Mpz = Mpz::from_str_radix("10", 10).unwrap();
    static ref PI : Number = Number::new(3.14159265359f64);
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
        self.value > ZERO_VALUE.value
    }
    fn is_it_negative(&self) -> bool {
        self.value < ZERO_VALUE.value
    }
}


impl Zero<Number> for Number {
    fn is_it_zero(&self) -> bool {
        self.value.is_zero()
    }

    fn zero() -> Number {
        Number::from_value(Mpq::zero())
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

impl Hash for NumberT<Mpq> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.get_num().hash(state);
        self.value.get_den().hash(state);
    }
}

impl NumberTrait<Number> for Number {
    fn convert_to_f32(self) -> f32 {

        let mut numer = self.value.get_num().clone();
        let mut denom = self.value.get_den().clone();
        //println!("num {0}", self);
        loop {
            //let opt_numer = numer.to_str_radix(10).parse::<f32>().ok();
            //let opt_denom = denom.to_str_radix(10).parse::<f32>().ok();
            let cur_numer = numer.to_str_radix(10).parse::<f32>().ok().unwrap();
            let cur_denom = denom.to_str_radix(10).parse::<f32>().ok().unwrap();

            //println!("num {:?}", opt_numer);
            //println!("denom {:?}", opt_denom);

            if cur_denom.is_infinite() || cur_numer.is_infinite() {
                if cur_denom.is_infinite() && (&numer.abs() < &VALUE_10_6) {
                    return 0.;
                }

                if cur_numer.is_infinite() && (&denom.abs() < &VALUE_10_6) {
                    panic!("ERROR: The value is too large: {0}", self.value);
                }

                numer = numer / VALUE_10.clone();
                denom = denom / VALUE_10.clone();
            } else {
                return cur_numer / cur_denom;
            }
        }
        // panic!("Something goes wrong!");
    }

    fn abs(self) -> Number {
        Number::from_value(Mpq::abs(&self.value))
    }


    fn new(x : f64) -> Number
    {
        let mut v = Mpq::new();
        v.set_d(x);
        return Number::from_value(v);
    }


    fn new_from_f32(x : f32) -> Number
    {
        let mut v = Mpq::new();
        v.set_d(x as f64);
        return Number::from_value(v);
    }

    fn approx_cos(&self, n: usize) -> Number {
        unsafe {
            static mut OPT_HASH: Option<HashMap<Number, Number>> = None;
            if OPT_HASH.is_none() {
                OPT_HASH = Some(HashMap::new());
            }

            if OPT_HASH.iter().next().unwrap().contains_key(self) {
                return OPT_HASH.iter().next().unwrap().get(self).unwrap().clone();
            }

            let mut res = ZERO_VALUE.clone();
            let x = self * PI.clone() / Number::new(180f64);

            for k in 0..n {
                let l = 2 * k;
                let factor = Number::new(pow(-1., k)) / factorial(Number::new(l as f64));
                res = res + factor * x.pow(l.clone());
            }

            OPT_HASH.iter_mut().next().unwrap().insert(self.clone(), res.clone());
            // println!("self: {0}, cos {1}", self, res.clone().convert_to_f32());
            return res;
        }
    }

    fn approx_sin(&self, n: usize) -> Number {
        unsafe {
            static mut OPT_HASH: Option<HashMap<Number, Number>> = None;
            if OPT_HASH.is_none() {
                OPT_HASH = Some(HashMap::new());
            }

            if OPT_HASH.iter().next().unwrap().contains_key(self) {
                return OPT_HASH.iter().next().unwrap().get(self).unwrap().clone();
            }

            let mut res = ZERO_VALUE.clone();
            let x = self * PI.clone() / Number::new(180f64);

            for k in 0..n {
                let l = 2 * k + 1;
                let factor = Number::new(pow(-1., k)) / factorial(Number::new(l as f64));
                res = res + factor * x.pow(l.clone());
            }

            OPT_HASH.iter_mut().next().unwrap().insert(self.clone(), res.clone());
            // println!("self: {0}, sin {1}", self, res.clone().convert_to_f32());
            return res;
        }
    }

    fn pow(&self, k: usize) -> Number {
        if k == 0 {
            return Number::new(1.);
        }

        let mut res = self.clone();
        for _ in 1..k {
            res = res * self;
        }

        return res;
    }
}


fn factorial(value: Number) -> Number {
    if value == *ZERO_VALUE {
        return Number::new(1f64);
    } else {
        return &value * factorial(&value - Number::new(1f64));
    }
}
