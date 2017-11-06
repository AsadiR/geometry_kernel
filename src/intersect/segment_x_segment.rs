
use primitives::*;
use intersect::line_x_line;
// use std::collections::BTreeSet;
use intersect::point_wrapper::PointWrapper;

#[derive(Debug)]
pub enum InfoSxS {
    IntersectingInASegment,
    IntersectingOnAPoint,
    Collinear,
    DisjointInTheLine,
    DisjointInThePlane,
    Skew,
    IntersectingInAPointOnLine
}

// http://mathhelpplanet.com/static.php?p=vzaimnoe-raspolozhenie-pryamyh-v-prostranstve
pub fn intersect(a : &Segment, b : &Segment) -> (Option<Point>, Option<Segment>, InfoSxS) {
    let la = if a.org >= a.dest  {
        Line {org: a.dest.clone(), dest: a.org.clone()}
    } else {
        Line {org: a.org.clone(), dest: a.dest.clone()}
    };

    let lb = if b.org >= b.dest {
        Line {org: b.dest.clone(), dest: b.org.clone()}
    } else {
        Line {org: b.org.clone(), dest: b.dest.clone()}
    };
    //println!("la {}", a);
    //println!("lb {}", b);

    let (sp, info) = line_x_line::intersect(&la, &lb);
    match info {
        line_x_line::InfoLxL::Skew => (None, None, InfoSxS::Skew),
        line_x_line::InfoLxL::Collinear => (None, None, InfoSxS::Collinear),
        line_x_line::InfoLxL::Coincidence => {
            let (op, os, info) = intersect_segments_on_the_line(&la.convert_to_segment(), &lb.convert_to_segment());
            (op, os, info)
        },
        line_x_line::InfoLxL::Intersecting => {
            let p = sp.unwrap();
            if (p >= la.org) & (p <= la.dest) & (p >= lb.org) & (p <= lb.dest) {
                //println!("Good");
                (Some(p), None, InfoSxS::IntersectingOnAPoint)
            } else {
                //println!("Bad: {}, lb.org: {}, {}", p, lb.org, (p >= lb.org));
                (None, None, InfoSxS::DisjointInThePlane)
            }
        }
    }
}

fn directed_segment(p1 : Point, p2 : Point, s : &Segment) -> Segment {
    let pw1 = PointWrapper::new(p1, s);
    let pw2 = PointWrapper::new(p2, s);
    if pw1 < pw2 {
        return Segment::new(pw1.extract_point(), pw2.extract_point());
    } else {
        return Segment::new(pw2.extract_point(), pw1.extract_point());
    }
}

// Intersect segments lying on the same line
pub fn intersect_segments_on_the_line(arg_sa : &Segment, arg_sb : &Segment) -> (Option<Point>, Option<Segment>, InfoSxS) {
    /*
    IMPORTANT: The result segment has the same direction as <arg_sb>
    */

    let mut sa = arg_sa.clone();
    let mut sb = arg_sb.clone();

    //println!("sa = {:?}, sb = {:?}", sa, sb);

    sa.flip_if_dest_less_than_org();
    sb.flip_if_dest_less_than_org();

    match 1 {
        _ if (sa.dest == sb.org) => {
            return (Some(sa.dest.clone()), None, InfoSxS::IntersectingInAPointOnLine);
        }

        _ if (sb.dest == sa.org) => {
            (Some(sb.dest.clone()), None, InfoSxS::IntersectingInAPointOnLine)
        }
        //sa поглащает sb
        _ if (sa.org <= sb.org) &
            (sa.dest >= sb.dest) => {
            (None, Some(directed_segment(sb.org, sb.dest, arg_sb)), InfoSxS::IntersectingInASegment)
        },
        //sb поглащает sa
        _ if (sb.org <= sa.org) &
            (sb.dest >= sa.dest) => {
            (None, Some(directed_segment(sa.org, sa.dest, arg_sb)), InfoSxS::IntersectingInASegment)
        },
        //
        _ if (sa.dest > sb.org) & (sa.dest < sb.dest) => {
            (None, Some(directed_segment(sb.org, sa.dest, arg_sb)), InfoSxS::IntersectingInASegment)
        },
        _ if (sb.dest > sa.org) &
            (sb.dest < sa.dest) => {
            (None, Some(directed_segment(sa.org, sb.dest, arg_sb)), InfoSxS::IntersectingInASegment)
        },
        _ => (None, None, InfoSxS::DisjointInTheLine)
    }
}


#[cfg(test)]
mod tests {
    use primitives::*;
    use intersect::*;

    #[test]
    fn point_intersection() {
        let p1 = Point::new_from_f64(1.0, 0.0, 0.0);
        let p2 = Point::new_from_f64(-1.0, 0.0, 0.0);
        let p3 = Point::new_from_f64(0.0, 1.0, 0.0);
        let p4 = Point::new_from_f64(0.0, -1.0, 0.0);

        let s1 = Segment {org: p1, dest: p2};
        let s2 = Segment {org: p3, dest: p4};

        let res = segment_x_segment::intersect(&s1, &s2);

        if let (Some(p),Option::None, segment_x_segment::InfoSxS::IntersectingOnAPoint) = res  {
            if p != Point::new_from_f64(0., 0., 0.) {
                panic!("Wrong result: {}", p);
            }
        } else {
            panic!("Wrong info: {:?}", res.2);
        };

    }

    #[test]
    fn no_intersection_in_the_plane() {
        let p1 = Point::new_from_f64(-1.0, 0.0, 0.0);
        let p2 = Point::new_from_f64(-2.0, 0.0, 0.0);
        let p3 = Point::new_from_f64(0.0, 1.0, 0.0);
        let p4 = Point::new_from_f64(0.0, -1.0, 0.0);

        let s1 = Segment {org: p1, dest: p2};
        let s2 = Segment {org: p3, dest: p4};

        let res = segment_x_segment::intersect(&s1, &s2);

        if let (Option::None, Option::None, segment_x_segment::InfoSxS::DisjointInThePlane) = res  {}
        else {
            panic!("Wrong info: {:?}", res.2);
        };
    }

    #[test]
    fn skew_segments() {
        let p1 = Point::new_from_f64(-1.0, 0.0, 1.0);
        let p2 = Point::new_from_f64(-2.0, 0.0, 1.0);
        let p3 = Point::new_from_f64(0.0, 1.0, 0.0);
        let p4 = Point::new_from_f64(0.0, -1.0, 0.0);

        let s1 = Segment {org: p1, dest: p2};
        let s2 = Segment {org: p3, dest: p4};

        let res = segment_x_segment::intersect(&s1, &s2);

        if let (Option::None, Option::None, segment_x_segment::InfoSxS::Skew) = res  {}
        else {
            panic!("Wrong info: {:?}", res.2);
        };
    }

    #[test]
    fn collinear_segments() {
        let p1 = Point::new_from_f64(-1., 0., 1.);
        let p2 = Point::new_from_f64(-2., 0., 1.);
        let p3 = Point::new_from_f64(-1., 0., 0.);
        let p4 = Point::new_from_f64(5., 0., 0.);

        let s1 = Segment {org: p1, dest: p2};
        let s2 = Segment {org: p3, dest: p4};

        let res = segment_x_segment::intersect(&s1, &s2);

        if let (Option::None, Option::None, segment_x_segment::InfoSxS::Collinear) = res  {}
        else {
            panic!("Wrong info: {:?}", res.2);
        };
    }

    #[test]
    fn coincidence_segments() {
        let p1 = Point::new_from_f64(-1., 0., 0.);
        let p2 = Point::new_from_f64(-2., 0., 0.);
        let p3 = Point::new_from_f64(-1., 0., 0.);
        let p4 = Point::new_from_f64(-2., 0., 0.);

        let s1 = Segment {org: p1, dest: p2};
        let s2 = Segment {org: p3, dest: p4};

        let res = segment_x_segment::intersect(&s1, &s2);

        if let (Option::None, Some(s), segment_x_segment::InfoSxS::IntersectingInASegment) = res  {
            let pt1 = Point::new_from_f64(-2., 0., 0.);
            let pt2 = Point::new_from_f64(-1., 0., 0.);
            let expected_s = Segment {org: pt1, dest: pt2};
            if s != expected_s {
                panic!("Wrong result: {}", s);
            }
        } else {
                panic!("Wrong info: {:?}", res.2);
        };
    }

    #[test]
    fn coincidence_segments_s1_gr_s2() {
        let p1 = Point::new_from_f64(-5., 0., 0.);
        let p2 = Point::new_from_f64(5., 0., 0.);
        let p3 = Point::new_from_f64(-1., 0., 0.);
        let p4 = Point::new_from_f64(2., 0., 0.);

        let s1 = Segment {org: p1, dest: p2};
        let s2 = Segment {org: p3, dest: p4};

        let res = segment_x_segment::intersect(&s1, &s2);

        if let (Option::None, Some(s), segment_x_segment::InfoSxS::IntersectingInASegment) = res  {
            let pt1 = Point::new_from_f64(-1., 0., 0.);
            let pt2 = Point::new_from_f64(2., 0., 0.);
            let expected_s = Segment {org: pt1, dest: pt2};
            if s != expected_s {
                panic!("Wrong result: {}", s);
            }
        } else {
            panic!("Wrong info: {:?}", res.2);
        };
    }

    #[test]
    fn coincidence_segments_s2_gr_s1() {
        let p1 = Point::new_from_f64(-1., 0., 0.);
        let p2 = Point::new_from_f64(2., 0., 0.);
        let p3 = Point::new_from_f64(-5., 0., 0.);
        let p4 = Point::new_from_f64(5., 0., 0.);

        let s1 = Segment {org: p1, dest: p2};
        let s2 = Segment {org: p3, dest: p4};

        let res = segment_x_segment::intersect(&s1, &s2);

        if let (Option::None, Some(s), segment_x_segment::InfoSxS::IntersectingInASegment) = res  {
            let pt1 = Point::new_from_f64(-1., 0., 0.);
            let pt2 = Point::new_from_f64(2., 0., 0.);
            let expected_s = Segment {org: pt1, dest: pt2};
            if s != expected_s {
                panic!("Wrong result: {}", s);
            }
        } else {
            panic!("Wrong info: {:?}", res.2);
        };
    }

    #[test]
    fn partial_coincidence1() {
        let p1 = Point::new_from_f64(-2., 0., 0.);
        let p2 = Point::new_from_f64(2., 0., 0.);
        let p3 = Point::new_from_f64(1., 0., 0.);
        let p4 = Point::new_from_f64(3., 0., 0.);

        let s1 = Segment {org: p1, dest: p2};
        let s2 = Segment {org: p3, dest: p4};

        let res = segment_x_segment::intersect(&s1, &s2);

        if let (Option::None, Some(s), segment_x_segment::InfoSxS::IntersectingInASegment) = res  {
            let pt1 = Point::new_from_f64(1., 0., 0.);
            let pt2 = Point::new_from_f64(2., 0., 0.);
            let expected_s = Segment {org: pt1, dest: pt2};
            if s != expected_s {
                panic!("Wrong result: {}", s);
            }
        } else {
            panic!("Wrong info: {:?}", res.2);
        };
    }

    #[test]
    fn partial_coincidence1_flipped() {
        let p1 = Point::new_from_f64(-2., 0., 0.);
        let p2 = Point::new_from_f64(2., 0., 0.);
        let p3 = Point::new_from_f64(1., 0., 0.);
        let p4 = Point::new_from_f64(3., 0., 0.);

        let s1 = Segment {org: p2, dest: p1};
        let s2 = Segment {org: p4, dest: p3};

        let res = segment_x_segment::intersect(&s1, &s2);

        if let (Option::None, Some(s), segment_x_segment::InfoSxS::IntersectingInASegment) = res  {
            let pt1 = Point::new_from_f64(1., 0., 0.);
            let pt2 = Point::new_from_f64(2., 0., 0.);
            let expected_s = Segment {org: pt1, dest: pt2};
            if s != expected_s {
                panic!("Wrong result: {}", s);
            }
        } else {
            panic!("Wrong info: {:?}", res.2);
        };
    }

    #[test]
    fn partial_coincidence2() {
        let p1 = Point::new_from_f64(1., 0., 0.);
        let p2 = Point::new_from_f64(3., 0., 0.);
        let p3 = Point::new_from_f64(-2., 0., 0.);
        let p4 = Point::new_from_f64(2., 0., 0.);

        let s1 = Segment {org: p1, dest: p2};
        let s2 = Segment {org: p3, dest: p4};

        let res = segment_x_segment::intersect(&s1, &s2);

        if let (Option::None, Some(s), segment_x_segment::InfoSxS::IntersectingInASegment) = res  {
            let pt1 = Point::new_from_f64(1., 0., 0.);
            let pt2 = Point::new_from_f64(2., 0., 0.);
            let expected_s = Segment {org: pt1, dest: pt2};
            if s != expected_s {
                panic!("Wrong result: {}", s);
            }
        } else {
            panic!("Wrong info: {:?}", res.2);
        };
    }

    #[test]
    fn no_intersection_on_the_line() {
        let p1 = Point::new_from_f64(2., 0., 0.);
        let p2 = Point::new_from_f64(4., 0., 0.);
        let p3 = Point::new_from_f64(-2., 0., 0.);
        let p4 = Point::new_from_f64(-4., 0., 0.);

        let s1 = Segment {org: p1, dest: p2};
        let s2 = Segment {org: p3, dest: p4};

        let res = segment_x_segment::intersect(&s1, &s2);

        if let (Option::None, Option::None, segment_x_segment::InfoSxS::DisjointInTheLine) = res  {}
        else {
            panic!("Wrong info: {:?}", res.2);
        };
    }


}

