use primitives::vector;
use primitives::number_trait;
use std::cmp::Ordering;
use std::f64::consts::PI;
use std::mem::swap;
use log::LogLevel;
use primitives::number::*;

// for template constraint
use primitives::signed_trait::Signed;
use primitives::zero_trait::Zero;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, Sub, Mul, Div, Neg};
use std::cmp::Ord;
use std::fmt;


/// This structure describes a point in a 3D space.
#[derive(Clone)]
#[derive(Hash)]
pub struct Point
{
    pub x: Number,
    pub y: Number,
    pub z: Number
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "Point [{:?}, {:?}, {:?}]", self.x.clone().convert_to_f32(), self.y.clone().convert_to_f32(), self.z.clone().convert_to_f32())
        write!(f, "Point [{0} <{1}>, {2} <{3}>, {4}<{5}>]",
               self.x.value, self.x.clone().convert_to_f32(),
               self.y.value, self.y.clone().convert_to_f32(),
               self.z.value, self.z.clone().convert_to_f32())
    }
}

#[derive(PartialEq, Eq)]
pub(crate) enum EPointPosition {
    Left,
    Right,
    Behind,
    Beyond,
    Org,
    Dest,
    Between
}


impl Point {
    /// This method converts the `Point` to a `Vector`.
    pub fn convert_to_vector(self) -> vector::Vector {
        vector::Vector {x: self.x, y: self.y, z: self.z}
    }

    /// This method creates a `Vector` from the `Point`.
    pub fn get_vector(&self) -> vector::Vector {
        vector::Vector {x: self.x.clone(), y: self.y.clone(), z: self.z.clone()}
    }

    /// This method creates `Point` from `x`, `y` and `z` coordinates.
    /// # Arguments
    ///
    /// * `x` - A `Number` representing the x coordinate.
    /// * `y` - A `Number` representing the y coordinate.
    /// * `z` - A `Number` representing the z coordinate.
    pub fn new(x : Number, y : Number, z : Number) -> Point {
        Point{x: x, y: y, z: z}
    }

    /// This method creates `Point` from `x`, `y` and `z` coordinates.
    /// # Arguments
    ///
    /// * `x` - A `f64` representing the x coordinate.
    /// * `y` - A `f64` representing the y coordinate.
    /// * `z` - A `f64` representing the z coordinate.
    pub fn new_from_f64(x : f64, y : f64, z : f64) -> Point {
        Point{x: Number::new(x), y: Number::new(y), z: Number::new(z)}
    }

    pub(crate) fn swap_yz(& mut self) {
        swap(&mut self.y, &mut self.z);
    }

    pub(crate) fn swap_xy(& mut self) {
        swap(&mut self.x, &mut self.y);
    }

    pub(crate) fn swap_xz(& mut self) {
        swap(&mut self.x, &mut self.z);
    }


    pub(crate) fn classify(&self, p0 : &Point, p1 : &Point) -> EPointPosition {
        let a = p1 - p0;
        let b = self - p0;
        let sa = &a.x*&b.y - &b.x*&a.y;
        match 1 {
            _ if sa.is_it_positive() => return EPointPosition::Left,
            _ if sa.is_it_negative() => return EPointPosition::Right,
            _ if (&a.x * &b.x).is_it_negative() | (&a.y * &b.y).is_it_negative() =>
                return EPointPosition::Behind,
            _ if a.length2() < b.length2() => return EPointPosition::Beyond,
            _ if *p0 == *self => return EPointPosition::Org,
            _ if *p1 == *self => return EPointPosition::Dest,
            _ => return EPointPosition::Between
        }
    }
}

impl Add<vector::Vector> for Point {
    type Output = Point;

    fn add(self, other: vector::Vector) -> Point {
        Point { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }
}


impl Sub<Point> for Point {
    type Output = vector::Vector;
    fn sub(self, other: Point) -> vector::Vector {
        vector::Vector { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z}
    }
}

impl<'a,'b> Add<&'b vector::Vector> for &'a Point {
    type Output = Point;

    fn add(self, other: &'b vector::Vector) -> Point {
        Point { x: &self.x + &other.x, y: &self.y + &other.y, z: &self.z + &other.z }
    }
}

impl<'a,'b> Sub<&'b Point> for &'a Point {
    type Output = vector::Vector;

    fn sub(self, other: &'b Point) -> vector::Vector {
        vector::Vector { x: &self.x - &other.x, y: &self.y - &other.y, z: &self.z - &other.z}
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        (self.x == other.x) & (self.y == other.y) & (self.z == other.z)
    }
}

impl Eq for Point {}

impl Ord for Point {
    fn cmp(&self, other: &Point) -> Ordering {
        match self {
            _ if *self == *other => Ordering::Equal,
            _ if (self.x < other.x) | (self.x == other.x) & (self.y < other.y) | (self.x == other.x) & (self.y == other.y) & (self.z < other.z) => Ordering::Less,
            _ => Ordering::Greater
        }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


#[cfg(test)]
mod tests {
    use primitives::number::*;
    use primitives::vector;
    use primitives::point;

    #[test]
    fn point_plus_vector() {
        let p = point::Point {x: Number::new(1.0), y: Number::new(1.0), z: Number::new(1.0)};
        let v = vector::Vector {x: Number::new(1.0), y: Number::new(1.0), z: Number::new(1.0)};
        let new_p1 = p.clone() + v.clone();
        let new_p2 = p + v;
        let expected_p = point::Point {x: Number::new(2.0), y: Number::new(2.0), z: Number::new(2.0)};
        assert!(new_p1 == new_p2);
        assert!(new_p1 == expected_p);
    }

    #[test]
    fn point_subtract_point() {
        let end = point::Point {x: Number::new(1.0), y: Number::new(1.0), z: Number::new(1.0)};
        let begin = point::Point {x: Number::new(2.0), y: Number::new(2.0), z: Number::new(2.0)};
        let v = end.clone() - begin.clone();
        let new_v = end - begin;
        let expected_v = vector::Vector {x: Number::new(-1.0), y: Number::new(-1.0), z: Number::new(-1.0)};
        assert!(v == new_v);
        assert!(v == expected_v);
    }
}
