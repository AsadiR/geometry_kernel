use primitives::*;


pub enum InfoLxP {
    Collinear,
    LineContainedInPlane,
    Intersecting
}

#[allow(dead_code)]
pub fn intersect(l : &Line, p : &Plane) -> (Option<Point>, InfoLxP) {
    let dir_v = l.get_dir_vector();
    let dp = dir_v.dot_product(&p.normal);

    // numerator = (l.org - p.point)*p.normal
    let numerator = p.normal.dot_product(&(&l.org - &p.point));
    match 1 {
        _ if dp.is_it_zero() & numerator.is_it_zero() => (None, InfoLxP::LineContainedInPlane),
        _ if dp.is_it_zero() => (None, InfoLxP::Collinear),
        _ => {
            let d = -numerator/dp;
            let point = &l.org + &(&dir_v*d);
            (Some(point), InfoLxP::Intersecting)
        }
    }
}


#[cfg(test)]
mod tests {
    use primitives::*;
    use intersect::*;

    #[test]
    fn intersecting_line_and_plane() {
        // n*p - 1 = 0 <=> n*(p-p0)=0

        let p1 = Point::new_from_f64(0.0, 1.0, 0.0);
        let p2 = Point::new_from_f64(0.0, 2.0, 0.0);

        let l = Line { org: p1, dest: p2 };

        let n = Vector::new_from_f64(1.0, 1.0, 1.0);
        let p0 = Point::new_from_f64(1.0, 0.0, 0.0);

        let p = Plane::new(n, p0);

        let res = line_x_plane::intersect(&l, &p);

        if let (Some(expected_p), line_x_plane::InfoLxP::Intersecting) = res {
            if expected_p == Point::new_from_f64(0.0, 1.0, 0.0) {
                return;
            } else {
                panic!("Wrong result {}", expected_p);
            }
        } else {
            panic!("Wrong info");
        }
    }

    #[test]
    fn collinear_line_and_plane() {
        // n*p - 1 = 0 <=> n*(p-p0)=0

        let p1 = Point::new_from_f64(0.0, 2.0, 0.0);
        let p2 = Point::new_from_f64(2.0, 0.0, 0.0);

        let l = Line { org: p1, dest: p2 };

        let n = Vector::new_from_f64(1.0, 1.0, 1.0);
        let p0 = Point::new_from_f64(1.0, 0.0, 0.0);

        let p = Plane::new(n, p0);

        let res = line_x_plane::intersect(&l, &p);

        if let (Option::None, line_x_plane::InfoLxP::Collinear) = res {
            return;
        } else {
            panic!("Wrong info");
        }
    }

    #[test]
    fn line_contained_in_plane() {
        // n*p - 1 = 0 <=> n*(p-p0)=0

        let p1 = Point::new_from_f64(0.0, 1.0, 0.0);
        let p2 = Point::new_from_f64(1.0, 0.0, 0.0);

        let l = Line { org: p1, dest: p2 };

        let n = Vector::new_from_f64(1.0, 1.0, 1.0);
        let p0 = Point::new_from_f64(1.0, 0.0, 0.0);

        let p = Plane::new(n, p0);

        let res = line_x_plane::intersect(&l, &p);

        if let (Option::None, line_x_plane::InfoLxP::LineContainedInPlane) = res {
            return;
        } else {
            panic!("Wrong info");
        }
    }
}