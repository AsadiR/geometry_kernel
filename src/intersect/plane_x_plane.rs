use primitives::*;
use matrix::*;



pub enum InfoPxP {
    Coincidence,
    Collinear,
    Intersecting
}

pub fn intersect(plane1 : &Plane, plane2 : &Plane) -> (Option<Line>, InfoPxP) {

    // (p - p0)*n = p*n + d => d = -p0*n
    let mut a = plane1.normal.cross_product(&plane2.normal);

    let d1 = plane1.get_d();
    let d2 = plane2.get_d();
    let n1 = &plane1.normal;
    let n2 = &plane2.normal;

    if a.is_zero() {
        //println!("d1: {}     d2: {}", d1, d2);
        if d1 == d2 {
            return (None, InfoPxP::Coincidence);
        }
        return (None, InfoPxP::Collinear);
    }


    /*
    a - direction vector of the line of intersection
    a = n1 x n2
    n1 u = -d1
    n2 u = -d2
    u - point on the line of intersection
    if a.z != 0 then:
        m = [n1 n2 k]^T
        k = [0 0 1]^T
        b = [-d1 -d2 0]^T
        m u = b
     else
        u.z == const
        m = [n1 n2 k]^T
        k = [1 0 0]^T | k = [0 1 0]^T
        b = [-d1 -d2 0]^T
        m u = b
    */


    let bv = match 1 {
        _ if !a.z.is_it_zero() =>  vec![-d1, -d2, plane1.point.z.clone()],
        _ if !a.y.is_it_zero() => vec![-d1, -d2, plane1.point.y.clone()],
        _  =>  vec![-d1, -d2, plane1.point.x.clone()]
    };

    let b : Row<Number> = Row::new_from_vector(bv);

    let mut last_row : Row<Number> = match 1 {
        _ if !a.z.is_it_zero() =>
            Row::new_from_vector(vec![number::new(0.),  number::new(0.),  number::new(1.)]),

        _ if !a.y.is_it_zero() =>
            Row::new_from_vector(vec![number::new(0.),  number::new(1.),  number::new(0.)]),

        _  =>
            Row::new_from_vector(vec![number::new(1.),  number::new(0.),  number::new(0.)])
    };

    let mv = vec![
        Row::new_from_vector(vec![n1.x.clone(), n1.y.clone(), n1.z.clone()]),
        Row::new_from_vector(vec![n2.x.clone(), n2.y.clone(), n2.z.clone()]),
        last_row
    ];

    let mut m : Matrix<Number> = Matrix::new_from_vector(mv);
    let u = m.solve(b);
    let mut u_vec = u.convert_to_vec();
    let (u0, u1, u2) = (u_vec.remove(0), u_vec.remove(0), u_vec.remove(0));

    let l_org = Point::new(u0.clone(), u1.clone(), u2.clone());
    let l_dest = Point::new(u0+a.x, u1+a.y, u2+a.z);

    let l = Line {
        org: l_org,
        dest: l_dest
    };

    (Some(l), InfoPxP::Intersecting)
}


#[cfg(test)]
mod tests {
    use primitives::*;
    use intersect::*;

    #[test]
    fn plane_x_plane_intersection() {
        let n = Vector::new_from_f64(7.0, 0., 0.);
        let p0 = Point::new_from_f64(0., 0., 0.);

        let plane1 = Plane { normal: n, point: p0};

        let n = Vector::new_from_f64(0., 7.0, 0.);
        let p0 = Point::new_from_f64(0., 0., 0.);

        let plane2 = Plane { normal: n, point: p0};

        let res = plane_x_plane::intersect(&plane1, &plane2);

        let expected_v = Vector::new_from_f64(0., 0., 3.);

        if let (Some(l), plane_x_plane::InfoPxP::Intersecting) = res {
            let v = &l.dest - &l.org;
            if  expected_v.is_collinear_to(&v) {
                return;
            } else {
                panic!("Wrong result {}", l);
            }
        } else {
            panic!("Wrong info");
        }
    }

    #[test]
    fn collinear_planes() {
        let n = Vector::new_from_f64(7.0, 0., 0.);
        let p0 = Point::new_from_f64(1., 0., 0.);

        let plane1 = Plane { normal: n, point: p0};

        let n = Vector::new_from_f64(7., 0.0, 0.);
        let p0 = Point::new_from_f64(5., 0., 0.);

        let plane2 = Plane { normal: n, point: p0};

        
        let res = plane_x_plane::intersect(&plane1, &plane2);

        if let (None, plane_x_plane::InfoPxP::Collinear) = res {
            return;
        } else {
            panic!("Wrong info");
        }
    }

    #[test]
    fn coincidence_planes() {
        let n = Vector::new_from_f64(7., 0., 0.);
        let p0 = Point::new_from_f64(5., 0., 0.);

        let plane1 = Plane { normal: n, point: p0};

        let n = Vector::new_from_f64(7., 0., 0.);
        let p0 = Point::new_from_f64(5., 0., 0.);

        let plane2 = Plane { normal: n, point: p0};

        
        let res = plane_x_plane::intersect(&plane1, &plane2);

        if let (None, plane_x_plane::InfoPxP::Coincidence) = res {
            return;
        } else {
            panic!("Wrong info");
        }
    }
}