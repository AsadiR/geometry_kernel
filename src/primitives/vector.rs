use primitives::point;
use primitives::number::*;
use std::mem::swap;
use primitives::to_2d_trait::To2D;

// use primitives::signed_trait::Signed;
// use primitives::zero_trait::Zero;
// use std::fmt::Debug;
// use std::hash::Hash;
use std::ops::{Add, Sub, Mul /*, Div, Neg*/};
// use std::cmp::Ord;
use std::fmt;


/// This structure repsresents a 3D vector.
#[derive(Hash)]
#[derive(Clone)]
pub struct Vector
{
    pub x: Number,
    pub y: Number,
    pub z: Number
}

impl fmt::Debug for Vector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point [{:?}, {:?}, {:?}]", self.x.clone().convert_to_f32(), self.y.clone().convert_to_f32(), self.z.clone().convert_to_f32())
    }
}

lazy_static! {
    static ref  ZERO : Vector = Vector {x: Number::new(0.), y: Number::new(0.), z: Number::new(0.)};
}


impl Vector {

    /// This method calculates a dot product of `self` and `other` and returns `Number`.
    /// # Arguments
    ///
    /// * `other` - The `Vector` to multiply on.
    pub fn dot_product(&self, other: &Vector) -> Number {
        &self.x*&other.x + &self.y*&other.y + &self.z*&other.z
    }

    /// This method calculates a cross product of `self` and `other` and returns `Number`.
    /// # Arguments
    ///
    /// * `other` - The `Vector` to multiply on.
    pub fn cross_product(&self, other: &Vector) -> Vector {
        //a2*b3  -   a3*b2,     a3*b1   -   a1*b3,     a1*b2   -   a2*b1
        Vector {x: &self.y*&other.z - &self.z*&other.y,
                y: &self.z*&other.x - &self.x*&other.z,
                z: &self.x*&other.y - &self.y*&other.x}
    }

    /// This method calculates a dot product of `self` and `other` and returns `Number`.
    /// # Arguments
    ///
    /// * `other` - The `Vector` to multiply on.
    pub fn mixed_product(&self, a: &Vector, b: &Vector) -> Number {
        self.dot_product(&(a.cross_product(b)))
    }

    // This method returns `true` if  it's a null.
    pub fn is_zero(&self) -> bool {
        *ZERO == *self
    }

    /// This method checks if `self` collinear to `other`.
    /// # Arguments
    ///
    /// * `other` - The `Vector` to compare with.
    pub fn is_collinear_to(&self, other : &Vector) -> bool {
        self.cross_product(other).is_zero()
    }

    /// This method creates a `Point` from the `Vector`.
    pub fn get_point(&self) -> point::Point {
        point::Point {
            x: self.x.clone(),
            y: self.y.clone(),
            z: self.z.clone()
        }
    }

    /// This method creates `Vector` from `x`, `y` and `z` coordinates.
    /// # Arguments
    ///
    /// * `x` - A `Number` representing the x coordinate.
    /// * `y` - A `Number` representing the y coordinate.
    /// * `z` - A `Number` representing the z coordinate.
    pub fn new(x : Number, y : Number, z : Number) -> Vector {
        Vector {x:x, y:y, z:z}
    }

    /// This method creates `Vector` from `x`, `y` and `z` coordinates.
    /// # Arguments
    ///
    /// * `x` - A `f64` representing the x coordinate.
    /// * `y` - A `f64` representing the y coordinate.
    /// * `z` - A `f64` representing the z coordinate.
    pub fn new_from_f64(x : f64, y : f64, z : f64) -> Vector {
        Vector {x: Number::new(x), y: Number::new(y), z: Number::new(z)}
    }

    /// This method returns a square length of the `Vector`.
    pub fn length2(&self) -> Number {
        &self.x*&self.x + &self.y*&self.y + &self.z*&self.z
    }

    pub(crate) fn get_signed_cos2(&self, other: &Vector) -> Number {
        let minus_one = Number::new(-1.);
        let zero = Number::new(0.);

        let dp = self.dot_product(other);

        let mut cos2 = dp.clone()*dp.clone()/self.length2()/other.length2();
        if dp < zero {
            cos2 = cos2 * &minus_one;
        }
        return cos2;
    }

}

impl To2D for Vector {
    fn swap_yz(& mut self) {
        swap(&mut self.y, &mut self.z);
    }

    fn swap_xy(& mut self) {
        swap(&mut self.x, &mut self.y);
    }

    fn swap_xz(& mut self) {
        swap(&mut self.x, &mut self.z);
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Vector) -> bool {
        (self.x == other.x) & (self.y == other.y) & (self.z == other.z)
    }
}

impl Eq for Vector {}

impl Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Vector {
        Vector { x: self.x + &other.x, y: self.y + &other.y, z: self.z + &other.z }
    }
}

impl<'a,'b> Add<&'b Vector> for &'a Vector {
    type Output = Vector;

    fn add(self, other: &Vector) -> Vector {
        Vector { x: &self.x + &other.x, y: &self.y + &other.y, z: &self.z + &other.z }
    }
}

impl Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector { x: self.x - &other.x, y: self.y - &other.y, z: self.z - &other.z }
    }
}

impl Mul<Number> for Vector {
    type Output = Vector;

    fn mul(self, other: Number) -> Vector {
        Vector { x: self.x*&other, y: self.y*&other, z: self.z*&other }
    }
}

impl<'a> Mul<Number> for &'a Vector {
    type Output = Vector;

    fn mul(self, other: Number) -> Vector {
        Vector { x: &self.x*&other, y: &self.y*&other, z: &self.z*&other }
    }
}


impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use primitives::number::*;
    use primitives::vector;
    //use primitives::point;

    #[test]
    fn vector_plus_vector() {
        let v1 = vector::Vector {x: Number::new(1.0), y: Number::new(1.0), z: Number::new(1.0)};
        let v2 = vector::Vector {x: Number::new(2.0), y: Number::new(1.0), z: Number::new(2.0)};
        let new_v = v1 + v2;
        let expected_v = vector::Vector {x: Number::new(3.0), y: Number::new(2.0), z: Number::new(3.0)};
        assert!(new_v == expected_v);
    }

    #[test]
    fn vector_minus_vector() {
        let v1 = vector::Vector {x: Number::new(1.0), y: Number::new(1.0), z: Number::new(1.0)};
        let v2 = vector::Vector {x: Number::new(2.0), y: Number::new(1.0), z: Number::new(2.0)};
        let new_v = v2 - v1;
        let expected_v = vector::Vector {x: Number::new(1.0), y: Number::new(0.0), z: Number::new(1.0)};
        assert!(new_v == expected_v);
    }

    #[test]
    fn vector_dp_vector() {
        let v1 = vector::Vector {x: Number::new(1.0), y: Number::new(1.0), z: Number::new(1.0)};
        let v2 = vector::Vector {x: Number::new(2.0), y: Number::new(1.0), z: Number::new(2.0)};
        let dp = v2.dot_product(&v1);
        let expected_dp = Number::new(5.0);
        assert!(dp == expected_dp);
    }

    #[test]
    fn vector_cp_vector() {
        let v1 = vector::Vector {x: Number::new(1.0), y: Number::new(1.0), z: Number::new(1.0)};
        let v2 = vector::Vector {x: Number::new(2.0), y: Number::new(1.0), z: Number::new(2.0)};
        let d = v2.cross_product(&v1);
        let v1_dp_d = v1.dot_product(&d);
        let v2_dp_d = v2.dot_product(&d);
        assert!(v1_dp_d == Number::new(0.0));
        assert!(v2_dp_d == Number::new(0.0));
    }

    #[test]
    fn mp_of_three_vectors() {
        let a = vector::Vector {x: Number::new(2.0), y: Number::new(0.0), z: Number::new(0.0)};
        let b = vector::Vector {x: Number::new(2.0), y: Number::new(1.0), z: Number::new(0.0)};
        let c = vector::Vector {x: Number::new(2.0), y: Number::new(1.0), z: Number::new(3.0)};
        let mp_abc = a.mixed_product(&b, &c);
        let mp_cab = c.mixed_product(&a, &b);
        let mp_bca = b.mixed_product(&c, &a);
        assert!(mp_abc == mp_cab);
        assert!(mp_cab == mp_bca);
        assert!(mp_bca == mp_abc);
    }

}
