use primitives::point::Point;
use primitives::vector::Vector;
use primitives::line::Line;
use primitives::number;
use primitives::number::NumberTrait;

use std::fmt;
use std::cmp::Ordering;
use std::f64;
use std::mem::swap;

#[derive(PartialEq,Eq)]
#[derive(Clone)]
#[derive(Debug, Hash)]
pub struct Segment {
    pub org : Point,
    pub dest: Point
}

impl Segment {
    pub fn new(org : Point, dest : Point) -> Segment {
        if org == dest {
            panic!("org == dest");
        }

        Segment {
            org : org,
            dest : dest
        }
    }

    pub fn get_point_projection(&self, p: &Point) -> Point {
        let op = p - &self.org;
        let od = &self.dest - &self.org;

        let dot_od_od = od.length2();
        let dot_op_od = op.dot_product(&od);

        return &self.org + &(od*(dot_od_od / dot_op_od));
    }

    pub fn get_org_dest(self) -> (Point, Point) {
        return (self.org, self.dest);
    }

    pub fn contains_point(&self, p: &Point) -> bool {
        // <p>, <s.org> and <s.dest> have to belong the same line!
        return (self.org <= self.dest) && (p >= &self.org) && (p <= &self.dest) ||
               (self.org > self.dest) && (p >= &self.dest) && (p <= &self.org);
    }

    pub fn gen_line(&self) -> Line {
        return Line::new(self.org.clone(), self.dest.clone());
    }

    pub fn rot(&mut self, normal : &Vector, d : &number::Number) {
        let two = number::new(2.);
        let m = Point {
            x: (&self.dest.x + &self.org.x)/&two,
            y: (&self.dest.y + &self.org.y)/&two,
            z: (&self.dest.z + &self.org.z)/&two
        };
        /*
        let mut temp = self.dest;

        self.org.rotate_around_axis(&m, &normal, 90.);
        self.dest = self.org;
        temp.rotate_around_axis(&m, &normal, 90.);
        self.org = temp;
        */
        self.org.rotate_around_axis_90(&m, &normal);
        self.dest.rotate_around_axis_90(&m, &normal);
        //swap(&mut self.org, &mut self.dest);


        //check rotate
        assert!((self.org.get_vector().dot_product(&normal) - d).abs() == number::zero());
        assert!((self.dest.get_vector().dot_product(&normal) - d).abs() == number::zero());
        assert!(&(&self.dest.get_vector() + &self.org.get_vector()) * number::new(0.5) == m.get_vector());
    }

    pub fn flip_if_dest_less_than_org(&mut self) {
        if self.dest < self.org {
            self.flip()
        }
    }

    pub fn flip(&mut self) {
        swap(&mut self.org, &mut self.dest);
    }

}

pub enum SegmentsInfo {Parallel, Skew}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Segment: {}, {})", self.org, self.dest)
    }
}

impl Ord for Segment {
    fn cmp(&self, other: &Segment) -> Ordering {
        match self {
            _ if self.org == other.org &&  self.dest == other.dest || self.org == other.dest &&  self.dest == other.org
                => Ordering::Equal,
            _ if self.org < other.org => Ordering::Less,
            _ if self.org > other.org => Ordering::Greater,
            _ if self.dest < other.dest => Ordering::Less,
            _ if self.dest > other.dest => Ordering::Greater,
            _ => panic!("Smth goes wrong!")
        }

    }
}


impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Segment) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


#[cfg(test)]
mod tests {
    use primitives::point::Point;
    use primitives::vector::Vector;
    use primitives::segment::Segment;

    #[ignore]
    #[test]
    fn rotation() {
        // x+y+z = 1
        let p1 = Point::new_from_f64(0., 0., 1.);
        let p2 = Point::new_from_f64(1., 0., 0.);
        let p3 = Point::new_from_f64(0., 1., 0.);

        let mut e = Segment { org: p1.clone(), dest: p2.clone() };

        let v1 = &p1 - &p2;
        let v2 = &p3 - &p2;
        let mut normal: Vector = v1.cross_product(&v2);
        // normal.normalize();

        let d = normal.dot_product(&p1.get_vector());
        e.rot(&normal, &d);
        e.rot(&normal, &d);

        assert!(p2 == e.org);
        assert!(p1 == e.dest);
    }
}



