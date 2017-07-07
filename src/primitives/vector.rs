use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::fmt;
use primitives::point;
use primitives::number;
use std::f64;
//use core::num::Float;
//use std::num::Float;



#[derive(Clone)]
pub struct Vector {
    pub x: number::Number,
    pub y: number::Number,
    pub z: number::Number
}

impl fmt::Debug for Vector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point [{:?}, {:?}, {:?}]", number::to_f32(self.x.clone()), number::to_f32(self.y.clone()), number::to_f32(self.z.clone()))
    }
}

lazy_static! {
    pub static ref  ZERO : Vector = Vector {x: number::new(0.), y: number::new(0.), z: number::new(0.)};
}


impl Vector {
    pub fn dot_product(&self, other: &Vector) -> number::Number {
        &self.x*&other.x + &self.y*&other.y + &self.z*&other.z
    }

    pub fn cross_product(&self, other: &Vector) -> Vector {
        //a2*b3  -   a3*b2,     a3*b1   -   a1*b3,     a1*b2   -   a2*b1
        Vector {x: &self.y*&other.z - &self.z*&other.y,
                y: &self.z*&other.x - &self.x*&other.z,
                z: &self.x*&other.y - &self.y*&other.x}
    }

    pub fn mixed_product(&self, a: &Vector, b: &Vector) -> number::Number {
        self.dot_product(&(a.cross_product(b)))
    }

    pub fn is_zero(&self) -> bool {
        *ZERO == *self
    }

    pub fn is_collinear_to(&self, other : &Vector) -> bool {
        self.cross_product(other).is_zero()
    }

    pub fn gen_point(&self) -> point::Point {
        point::Point {
            x: self.x.clone(),
            y: self.y.clone(),
            z: self.z.clone()
        }
    }
    pub fn new(x : number::Number, y : number::Number, z : number::Number) -> Vector {
        Vector {x:x, y:y, z:z}
    }

    pub fn length(&self) -> number::Number {
        // sqrt!!!
        //(self.x*self.x + self.y*self.y + self.z*self.z).sqrt()
        &self.x*&self.x + &self.y*&self.y + &self.z*&self.z
    }

    pub fn normalize(&mut self) {
        let l = self.length();
        self.x = &self.x / &l;
        self.y = &self.y / &l;
        self.z = &self.z / &l;
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

impl Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector { x: self.x - &other.x, y: self.y - &other.y, z: self.z - &other.z }
    }
}

impl Mul<number::Number> for Vector {
    type Output = Vector;

    fn mul(self, other: number::Number) -> Vector {
        Vector { x: self.x*&other, y: self.y*&other, z: self.z*&other }
    }
}


impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use primitives::number;
    use primitives::vector;
    use primitives::point;

    #[test]
    fn vector_plus_vector() {
        let v1 = vector::Vector {x: number::new(1.0), y: number::new(1.0), z: number::new(1.0)};
        let v2 = vector::Vector {x: number::new(2.0), y: number::new(1.0), z: number::new(2.0)};
        let new_v = v1 + v2;
        let expected_v = vector::Vector {x: number::new(3.0), y: number::new(2.0), z: number::new(3.0)};
        assert!(new_v == expected_v);
    }

    #[test]
    fn vector_minus_vector() {
        let v1 = vector::Vector {x: number::new(1.0), y: number::new(1.0), z: number::new(1.0)};
        let v2 = vector::Vector {x: number::new(2.0), y: number::new(1.0), z: number::new(2.0)};
        let new_v = v2 - v1;
        let expected_v = vector::Vector {x: number::new(1.0), y: number::new(0.0), z: number::new(1.0)};
        assert!(new_v == expected_v);
    }

    #[test]
    fn vector_dp_vector() {
        let v1 = vector::Vector {x: number::new(1.0), y: number::new(1.0), z: number::new(1.0)};
        let v2 = vector::Vector {x: number::new(2.0), y: number::new(1.0), z: number::new(2.0)};
        let dp = v2.dot_product(&v1);
        let expected_dp = number::new(5.0);
        assert!(dp == expected_dp);
    }

    #[test]
    fn vector_cp_vector() {
        let v1 = vector::Vector {x: number::new(1.0), y: number::new(1.0), z: number::new(1.0)};
        let v2 = vector::Vector {x: number::new(2.0), y: number::new(1.0), z: number::new(2.0)};
        let d = v2.cross_product(&v1);
        let v1_dp_d = v1.dot_product(&d);
        let v2_dp_d = v2.dot_product(&d);
        assert!(v1_dp_d == number::new(0.0));
        assert!(v2_dp_d == number::new(0.0));
    }

    #[test]
    fn mp_of_three_vectors() {
        let a = vector::Vector {x: number::new(2.0), y: number::new(0.0), z: number::new(0.0)};
        let b = vector::Vector {x: number::new(2.0), y: number::new(1.0), z: number::new(0.0)};
        let c = vector::Vector {x: number::new(2.0), y: number::new(1.0), z: number::new(3.0)};
        let mp_abc = a.mixed_product(&b, &c);
        let mp_cab = c.mixed_product(&a, &b);
        let mp_bca = b.mixed_product(&c, &a);
        assert!(mp_abc == mp_cab);
        assert!(mp_cab == mp_bca);
        assert!(mp_bca == mp_abc);
    }

}
