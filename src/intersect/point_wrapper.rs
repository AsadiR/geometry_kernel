use primitives::*;
use std::cmp::Ordering;

#[derive(Debug)]
#[derive(Clone)]
pub struct PointWrapper {
    point : Point,
    t : Number
}

impl PointWrapper {
    pub fn new(p : Point, s : &Segment) -> PointWrapper {
        // p должна лежать внутри s, иначе сортировка не будет иметь смысла
        // сортируем по степени удаленности от начала отрезка

        let mut t : Number;
        if s.dest.x != s.org.x {
            t = (&p.x - &s.org.x) / (&s.dest.x - &s.org.x);
        } else if s.dest.y != s.org.y {
            t = (&p.y - &s.org.y) / (&s.dest.y - &s.org.y);
        } else if s.dest.z != s.org.z {
            t = (&p.z - &s.org.z) / (&s.dest.z - &s.org.z);
        } else {
            panic!("Segment with coincident points is not allowed here!")
        }

        PointWrapper {
            point : p,
            t : t
        }
    }

    pub fn extract_point(self) -> Point {
        self.point
    }
}

impl PartialEq for PointWrapper {
    fn eq(&self, other: &PointWrapper) -> bool {
        self.point == other.point
    }
}

impl Eq for PointWrapper {}

impl Ord for PointWrapper {
    fn cmp(&self, other: &PointWrapper) -> Ordering {
        self.t.cmp(&other.t)
    }
}

impl PartialOrd for PointWrapper {
    fn partial_cmp(&self, other: &PointWrapper) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

