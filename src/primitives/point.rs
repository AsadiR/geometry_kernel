
use std::ops::Add;
use std::ops::Sub;
use std::fmt;
use primitives::vector;
use primitives::number;
use std::cmp::Ordering;
use std::f64::consts::PI;
use std::mem::swap;
use log::LogLevel;
use primitives::number::NumberTrait;
use primitives::signed_trait::Signed;

#[derive(Clone)]
#[derive(Hash)]
pub struct Point {
    pub x: number::Number,
    pub y: number::Number,
    pub z: number::Number
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point [{:?}, {:?}, {:?}]", self.x.clone().to_f32(), self.y.clone().to_f32(), self.z.clone().to_f32())
    }
}

#[derive(PartialEq, Eq)]
pub enum EPointPosition {
    Left,
    Right,
    Behind,
    Beyond,
    Org,
    Dest,
    Between
}


impl Point {
    pub fn convert_to_vector(self) -> vector::Vector {
        vector::Vector {x: self.x, y: self.y, z: self.z}
    }

    pub fn get_vector(&self) -> vector::Vector {
        vector::Vector {x: self.x.clone(), y: self.y.clone(), z: self.z.clone()}
    }

    pub fn new(x : number::Number, y : number::Number, z : number::Number) -> Point {
        Point{x: x, y: y, z: z}
    }

    pub fn new_from_f64(x : f64, y : f64, z : f64) -> Point {
        Point{x: number::new(x), y: number::new(y), z: number::new(z)}
    }

    pub fn swap_yz(& mut self) {
        swap(&mut self.y, &mut self.z);
    }

    pub fn swap_xy(& mut self) {
        swap(&mut self.x, &mut self.y);
    }

    pub fn swap_xz(& mut self) {
        swap(&mut self.x, &mut self.z);
    }


    pub fn classify(&self, p0 : &Point, p1 : &Point) -> EPointPosition {
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

    pub fn rotate_around_axis_90(&mut self, point_on_axis : &Point, axis_dir : &vector::Vector) {
        //assert!(f64::abs(axis_dir.length() - 1.) <= EPS, "AxisDir must be unit vector");

        let res : Point;
        {
            let v: &Point = self;

            let r: vector::Vector = v - point_on_axis;
            let part1 : Point = point_on_axis.clone();
            let part2: number::Number = axis_dir.dot_product(&r);
            let part3: vector::Vector = axis_dir.cross_product(&r);
            res = (part1 + axis_dir * part2) + part3;
        }
        *self = res

    }


}

fn rad_from_deg(x : f64) -> f64 {
    return (PI / 180.) * x;
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
    use primitives::number;
    use primitives::vector;
    use primitives::point;

    #[test]
    fn point_plus_vector() {
        let p = point::Point {x: number::new(1.0), y: number::new(1.0), z: number::new(1.0)};
        let v = vector::Vector {x: number::new(1.0), y: number::new(1.0), z: number::new(1.0)};
        let new_p1 = p.clone() + v.clone();
        let new_p2 = p + v;
        let expected_p = point::Point {x: number::new(2.0), y: number::new(2.0), z: number::new(2.0)};
        assert!(new_p1 == new_p2);
        assert!(new_p1 == expected_p);
    }

    #[test]
    fn point_subtract_point() {
        let end = point::Point {x: number::new(1.0), y: number::new(1.0), z: number::new(1.0)};
        let begin = point::Point {x: number::new(2.0), y: number::new(2.0), z: number::new(2.0)};
        let v = end.clone() - begin.clone();
        let new_v = end - begin;
        let expected_v = vector::Vector {x: number::new(-1.0), y: number::new(-1.0), z: number::new(-1.0)};
        assert!(v == new_v);
        assert!(v == expected_v);
    }
}
