use primitives::*;
use intersect::line_x_line;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Eq)]
pub enum InfoLxS {
    IntersectingInASegment,
    IntersectingInAPoint,
    Collinear,
    DisjointInThePlane,
    Skew
}


pub fn intersect(line : &Line, segment : &Segment) -> (Option<Point>, Option<Segment>, InfoLxS) {
    let line_of_segment : Line = segment.gen_line();

    let (sp, info) = line_x_line::intersect(line, &line_of_segment);
    match info {
        line_x_line::InfoLxL::Skew => (None, None, InfoLxS::Skew),
        line_x_line::InfoLxL::Collinear => (None, None, InfoLxS::Collinear),
        line_x_line::InfoLxL::Coincidence => {

            //let (os, info) = intersect_segments_on_the_line(&la.convert_to_segment(), &lb.convert_to_segment());
            (None, Some(line_of_segment.convert_to_segment()), InfoLxS::IntersectingInASegment)
        },


        line_x_line::InfoLxL::Intersecting => {
            let p = sp.unwrap();
            //println!("point: {:?}, lb.org: {:?}, lb.dest: {:?}", p, lb.org, lb.dest);
            if segment.contains_point(&p) {
                (Some(p), None, InfoLxS::IntersectingInAPoint)
            } else {
                //println!("Bad: {}, lb.org: {}, {}", p, lb.org, (p >= lb.org));
                (None, None, InfoLxS::DisjointInThePlane)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use primitives::*;
    use intersect::*;

    #[test]
    fn point_intersection() {
        let p1 = Point::new_from_f64(10., 0.0, 0.0);
        let p2 = Point::new_from_f64(9., 0.0, 0.0);
        let p3 = Point::new_from_f64(0.0, 1.0, 0.0);
        let p4 = Point::new_from_f64(0.0, -1.0, 0.0);

        let line = Line {org: p1, dest: p2};
        let segment = Segment {org: p3, dest: p4};

        let res = line_x_segment::intersect(&line, &segment);

        if let (Some(p),Option::None, line_x_segment::InfoLxS::IntersectingInAPoint) = res  {
            if p != Point::new_from_f64(0., 0., 0.) {
                panic!("Wrong result: {}", p);
            }
        } else {
            panic!("Wrong info: {:?}", res.2);
        };

    }

}

