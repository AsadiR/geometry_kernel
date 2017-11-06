use primitives::*;
use matrix::*;
// use std::mem;


pub enum InfoLxL {
    Skew,
    Collinear,
    Coincidence,
    Intersecting
}


// http://mathhelpplanet.com/static.php?p=vzaimnoe-raspolozhenie-pryamyh-v-prostranstve
pub fn intersect(a : &Line, b : &Line) -> (Option<Point>, InfoLxL) {
    let m1 : &Point = &a.org;
    let l1 : Vector = &a.dest - &a.org;
    let m2 : &Point = &b.org;
    let l2 : Vector = &b.dest - &b.org;
    let mut m : Vector = &b.org - &a.org;
    if m.is_zero() {
        m = &b.org - &a.dest;
    }


    //Are lines skew?
    let mp = m.mixed_product(&l1, &l2);
    if mp != Number::zero() {
        return (None, InfoLxL::Skew);
    }


    //Are lines coincidence?
    let c_cond = l1.is_collinear_to(&m) && l2.is_collinear_to(&m);
    if c_cond {
        //println!("la {:?} lb {:?}", a, b);
        //println!("Inside intersect {:?}, {:?}", l1, m);
        return (None, InfoLxL::Coincidence);
    }

    //Are lines parallel? If yes then return (None, None).
    let p_cond = l1.is_collinear_to(&l2);
    if !c_cond & p_cond {
        return (None, InfoLxL::Collinear);
    }


    /*
    1) m1.x + l1.x*t = m2.x + l2.x*s
    2) m1.y + l1.y*t = m2.y + l2.y*s
    3) m1.z + l1.z*t = m2.z + l2.z*s

    ax = y;

    t:       s:       y:
    l1.x    -l2.x   (m2.x - m1.x)
    l1.y    -l2.y   (m2.y - m1.y)
    l1.z    -l2.z   (m2.z - m1.z)
    */

    //I can improve it!
    let nv = l1.cross_product(&l2);

    let mut a : Matrix<Number> = Matrix::new_from_vector(
        vec![Row::new_from_vector(vec![l1.x.clone(), -l2.x.clone(), nv.x.clone()]),
             Row::new_from_vector(vec![l1.y.clone(), -l2.y.clone(), nv.y.clone()]),
             Row::new_from_vector(vec![l1.z.clone(), -l2.z.clone(), nv.z.clone()])]);

    let y : Row<Number> = Row::new_from_vector(vec![&m2.x-&m1.x+&nv.x, &m2.y-&m1.y+&nv.y, &m2.z-&m1.z+&nv.z]);


    //println!("matrix:");
    //println!("{}", a);
    //println!("{}", y);



    let x = a.solve(y);
    //println!("{}", x);
    let t = x.get(&0);

    let p = m1 + &(&l1 * t);

    (Some(p), InfoLxL::Intersecting)
}

// Дублирование во благо производительности
pub fn intersect_p(a : &Line, b : &Line) -> Option<Number> {
    let m1 : &Point = &a.org;
    let l1 : Vector = &a.dest - &a.org;
    let m2 : &Point = &b.org;
    let l2 : Vector = &b.dest - &b.org;
    let mut m : Vector = &b.org - &a.org;
    if m.is_zero() {
        m = &b.org - &a.dest;
    }

    let mp = m.mixed_product(&l1, &l2);
    if mp != Number::zero() {
        return None;
    }

    let c_cond = l1.is_collinear_to(&m) && l2.is_collinear_to(&m);
    if c_cond {
        return None;
    }

    let p_cond = l1.is_collinear_to(&l2);
    if !c_cond & p_cond {
        return None;
    }

    let nv = l1.cross_product(&l2);

    let mut a : Matrix<Number> =Matrix::new_from_vector(
        vec![Row::new_from_vector(vec![l1.x.clone(), -l2.x.clone(), nv.x.clone()]),
             Row::new_from_vector(vec![l1.y.clone(), -l2.y.clone(), nv.y.clone()]),
             Row::new_from_vector(vec![l1.z.clone(), -l2.z.clone(), nv.z.clone()])]);

    let y : Row<Number> = Row::new_from_vector(vec![&m2.x-&m1.x+&nv.x, &m2.y-&m1.y+&nv.y, &m2.z-&m1.z+&nv.z]);

    let x = a.solve(y);
    //println!("{}", x);
    let t = x.get(&0);

    return Some(t);
}


#[cfg(test)]
mod tests {
    use primitives::*;
    use intersect::*;

    #[test]
    fn line_intersection_abc() {
        let p1 = Point::new_from_f64(1.0, 1.0, 1.0);
        let p2 = Point::new_from_f64(0.0, 0.0, 0.0);
        let p3 = Point::new_from_f64(-1.0, -1.0, 1.0);
        let p4 = Point::new_from_f64(0.0, 0.0, 0.0);

        let l1 = Line { org: p1, dest: p2 };
        let l2 = Line { org: p3, dest: p4 };

        let res = line_x_line::intersect(&l1, &l2);

        if let (Some(expected_p), line_x_line::InfoLxL::Intersecting) = res {
            if expected_p == Point::new_from_f64(0.0, 0.0, 0.0) {
                return;
            } else {
                panic!("Wrong result {}", expected_p);
            }
        } else {
            panic!("Wrong info");
        }
    }

    #[test]
    fn line_intersection_ab() {
        let p1 = Point::new_from_f64(1.0, 0.0, 1.0);
        let p2 = Point::new_from_f64(-1.0, 0.0, 1.0);
        let p3 = Point::new_from_f64(0.0, 1.0, 1.0);
        let p4 = Point::new_from_f64(0.0, -1.0, 1.0);

        let l1 = Line { org: p1, dest: p2 };
        let l2 = Line { org: p3, dest: p4 };

        let res = line_x_line::intersect(&l1, &l2);

        if let (Some(expected_p), line_x_line::InfoLxL::Intersecting) = res {
            if expected_p == Point::new_from_f64(0.0, 0.0, 1.0) {
                return;
            } else {
                panic!("Wrong result {}", expected_p);
            }
        } else {
            panic!("Wrong info");
        }
    }

    #[test]
    fn lines_skew() {
        let p1 = Point::new_from_f64(0.0, 0.0, 0.0);
        let p2 = Point::new_from_f64(0.0, 0.0, 1.0);
        let p3 = Point::new_from_f64(1.0, 6.0, 0.0);
        let p4 = Point::new_from_f64(0.0, 6.0, 0.0);

        let l1 = Line { org: p1, dest: p2 };
        let l2 = Line { org: p3, dest: p4 };

        let res = line_x_line::intersect(&l1, &l2);

        if let (Option::None, line_x_line::InfoLxL::Skew) = res {
            return;
        } else {
            panic!("Wrong info");
        }
    }

    #[test]
    fn lines_coincidence() {
        let p1 = Point::new_from_f64(0.0, 0.0, 0.0);
        let p2 = Point::new_from_f64(0.0, 0.0, 1.0);
        let p3 = Point::new_from_f64(0.0, 0.0, -1.0);
        let p4 = Point::new_from_f64(0.0, 0.0, 2.0);

        let l1 = Line { org: p1, dest: p2 };
        let l2 = Line { org: p3, dest: p4 };

        let res = line_x_line::intersect(&l1, &l2);

        if let (Option::None, line_x_line::InfoLxL::Coincidence) = res {
            return;
        } else {
            panic!("Wrong info");
        }
    }

    #[test]
    fn lines_parallel() {
        let p1 = Point::new_from_f64(1.0, 1.0, 1.0);
        let p2 = Point::new_from_f64(0.0, 0.0, 1.0);
        let p3 = Point::new_from_f64(1.0, 1.0, 0.0);
        let p4 = Point::new_from_f64(0.0, 0.0, 0.0);

        let l1 = Line { org: p1, dest: p2 };
        let l2 = Line { org: p3, dest: p4 };

        let res = line_x_line::intersect(&l1, &l2);

        if let (Option::None, line_x_line::InfoLxL::Collinear) = res {
            return;
        } else {
            panic!("Wrong info");
        }
    }
}










