use primitives::point::Point;
// use primitives::vector::Vector;
use primitives::line::Line;
// use primitives::number_trait;
// use primitives::number_trait::NumberTrait;
use primitives::zero_trait::Zero;

use std::fmt;
use std::cmp::Ordering;
// use std::f64;
use std::mem::swap;

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

        if dot_op_od.is_it_zero() {
            return self.org.clone();
        }

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

    pub fn flip_if_dest_less_than_org(&mut self) {
        if self.dest < self.org {
            self.flip()
        }
    }

    pub fn flip(&mut self) {
        swap(&mut self.org, &mut self.dest);
    }

}

// pub enum SegmentsInfo {Parallel, Skew}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Segment: {}, {})", self.org, self.dest)
    }
}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Segment) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Segment {}

impl PartialEq for Segment {
    fn eq(&self, other: &Segment) -> bool {
        self.org == other.org &&  self.dest == other.dest || self.org == other.dest &&  self.dest == other.org
    }
}


impl Ord for Segment {
    fn cmp(&self, other: &Segment) -> Ordering {
        match self {
            _ if *self == *other => Ordering::Equal,
            _ if self.org < other.org => Ordering::Less,
            _ if self.org > other.org => Ordering::Greater,
            _ if self.dest < other.dest => Ordering::Less,
            _ if self.dest > other.dest => Ordering::Greater,
            _ => panic!("Smth goes wrong!")
        }

    }
}







