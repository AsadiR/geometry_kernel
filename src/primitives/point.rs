
use std::ops::Add;
use std::ops::Sub;
use std::fmt;
use primitives::vector;
use primitives::number;
use std::cmp::Ordering;
use std::f64::consts::PI;
use std::f64;
use std::mem::swap;

#[derive(Clone)]
#[derive(Hash)]
pub struct Point {
    pub x: number::Number,
    pub y: number::Number,
    pub z: number::Number
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point [{:?}, {:?}, {:?}]", number::to_f32(self.x.clone()), number::to_f32(self.y.clone()), number::to_f32(self.z.clone()))
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

    /*
    pub fn classify(&self, p0 : &Point, p1 : &Point) -> EPointPosition {
        let a = p1 - p0;
        let b = self - p0;
        let sa = a.x*b.y - b.x*a.y;
        match 1 {
            _ if sa > EPS => return EPointPosition::Left,
            _ if sa < -EPS => return EPointPosition::Right,
            _ if (a.x * b.x < 0.0) | (a.y * b.y < 0.0) => return EPointPosition::Behind,
            _ if a.length() < b.length() => return EPointPosition::Beyond,
            _ if *p0 == *self => return EPointPosition::Org,
            _ if *p1 == *self => return EPointPosition::Dest,
            _ => return EPointPosition::Between
        }
    }


    pub fn rotate_around_axis(&mut self, point_on_axis : &Point, axis_dir : &Vector, angle : f64) {
        /*
        Point r = v - PointOnAxis;
        return PointOnAxis + cos(RadFromDeg(Angle)) * r
            + ((1 - cos(RadFromDeg(Angle))) * AxisDir.dotProduct3D(r)) * AxisDir
            + sin(RadFromDeg(Angle)) * AxisDir.crossProduct(r);
        */
        assert!(f64::abs(axis_dir.length() - 1.) <= EPS, "AxisDir must be unit vector");

        let res : Point;
        {
            let v: &Point = self;

            let r: Vector = v - point_on_axis;
            let part1: Point = point_on_axis + &(&r * rad_from_deg(angle).cos());
            let part2: f64 = axis_dir.dot_product(&r) * (1. - rad_from_deg(angle).cos());
            let part3: Vector = (&axis_dir.cross_product(&r)) * rad_from_deg(angle).sin();
            res = &(&part1 + &(axis_dir * part2)) + &part3;
        }
        *self = res

    }
    */

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
        vector::Vector { x: other.x - self.x, y: other.y - self.y, z: other.z - self.z}
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
        vector::Vector { x: &other.x - &self.x, y: &other.y - &self.y, z: &other.z - &self.z}
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
        let expected_v = vector::Vector {x: number::new(1.0), y: number::new(1.0), z: number::new(1.0)};
        assert!(v == new_v);
        assert!(v == expected_v);
    }
}
