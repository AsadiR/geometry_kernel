use primitives::point::Point;
use primitives::vector::Vector;
use std::fmt;
use primitives::segment::Segment;


#[derive(Debug)]
pub struct Line {
    pub org : Point,
    pub dest: Point
}

impl Line {
    pub fn new(org : Point, dest : Point) -> Line {
        if org == dest {
            panic!("org == dest");
        }

        Line {
            org : org,
            dest : dest
        }
    }

    pub fn convert_to_segment(self) -> Segment {
        Segment {org: self.org, dest: self.dest}
    }

    pub fn gen_segment(&self) -> Segment {
        Segment {org: self.org.clone(), dest: self.dest.clone()}
    }

    pub fn  get_dir_vector(&self) -> Vector {
        &self.dest - &self.org
    }

    #[allow(dead_code)]
    pub fn check_accessory(&self, point : &Point) -> bool {
        let dir_vec = self.get_dir_vector();
        let check_vec = &self.org - point;
        let cp = dir_vec.cross_product(&check_vec);
        cp.is_zero()
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Line: {}, {})", self.org, self.dest)
    }
}