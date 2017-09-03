use std::collections::BTreeSet;
use std::f64;
use primitives::*;
use matrix::*;
use intersect::line_x_line;

pub fn triangulate(mut points : Vec<Point>) -> Vec<Triangle> {
    // all points should be unique! Otherwise algorithm will hang out!
    println!("Triangulation was started for points:");
    println!("{:?}", points);

    assert!(points.len() > 3, "Not enough points!");

    let mut ts : Vec<Triangle> = Vec::new();

    let mut p = Point::new_from_f64(0.,0.,0.);
    let mut orientation : Number = number::new(0.);
    let mut normal_ = get_normal(&points, & mut orientation);
    let mut d_ = normal_.dot_product(&points[0].get_vector());

    check_points(&normal_, &d_, &points);
    let normal_type : NormalType = classify_normal(&normal_);

    modify_points(&mut points, &normal_type, &mut orientation, &mut normal_, &mut d_);

    if orientation.is_it_negative() {
        normal_ = &normal_ * number::new(-1.);
        d_ = normal_.dot_product(&points[0].get_vector());
    }

    check_points(&normal_, &d_, &points);

    //let n = points.len();
    let e : Segment = hull_edge(&mut points);
    //println!("ps {:?}", points);
    println!("hull_edge_res = {}\n", e);

    let mut frontier : BTreeSet<Segment> = BTreeSet::new();
    frontier.insert(e);

    while !frontier.is_empty() {
        let e = frontier.iter().next_back().unwrap().clone();

        println!("frontie {:?}", frontier);
        frontier.remove(&e);
        println!("edge: {:?}", e);
        if mate(&e, &mut points, &mut p, &normal_, &d_) {
            println!("mate_point = {}", p);

            update_frontier(&mut frontier, &p, &e.org);
            update_frontier(&mut frontier, &e.dest, &p);
            let tr : Triangle;
            if orientation.is_it_negative() {
                tr = Triangle::new(vec![
                    inv_point_transform(&e.org, &normal_type),
                    inv_point_transform(&e.dest, &normal_type),
                    inv_point_transform(&p, &normal_type)
                ]);
            } else {
                tr = Triangle::new(vec![
                    inv_point_transform(&e.dest, &normal_type),
                    inv_point_transform(&e.org, &normal_type),
                    inv_point_transform(&p, &normal_type)
                ]);
            }
            ts.push(tr);
        }
        println!();
    }

    return ts;
}


fn inv_point_transform(p : &Point, normal_type : &NormalType) -> Point {
    let mut pc = p.clone();
    match *normal_type {
        //NormalType::ABC => {},
        NormalType::AB => {
            pc.swap_xy();
            pc.swap_yz();
        },
        //NormalType::AC => {},
        //NormalType::BC => {},
        NormalType::A => {
            pc.swap_xy();
            pc.swap_xz();
        },

        NormalType::B => {
            pc.swap_xy();
            pc.swap_yz();
        },
        //NormalType::C => {}
        _ => {}
    };
    return pc;
}

fn hull_edge(points : &mut Vec<Point>) -> Segment {
    let mut m = 0;
    let n = points.len();
    for i in 1..n {
        if points[i] < points[m] {
            m = i;
        }
    }
    points.swap(0,m);
    m = 1;
    for i in 2..n {
        let c = points[i].classify(&points[0], &points[m]);
        if (c == point::EPointPosition::Left) | (c == point::EPointPosition::Between) {
            m = i;
        }
    }

    Segment {
        org: points[0].clone(),
        dest: points[m].clone()
    }
}

fn mate(
    e : &Segment,
    points : &mut Vec<Point>,
    p : &mut Point,
    normal_ : &Vector,
    d_ : &Number
) -> bool {

    let mut best_p : Option<Point> = None;
    let mut best_t_min : Option<Number> = None;

    let f : Line = get_segment_normal(e, normal_, d_);
    //println!("f {:?}", f);

    for i in 0..points.len() {
        let c = points[i].classify(&e.org, &e.dest);
        if c == point::EPointPosition::Right {
            let cur_s = Segment::new(e.dest.clone(), points[i].clone());
            let g : Line = get_segment_normal(&cur_s, normal_, d_);
            let ot = line_x_line::intersect_p(&f, &g);

            /*
            println!("------");
            println!("point: {:?}", points[i]);
            println!("cur_len2: {0}", cur_len2.clone().unwrap());
            println!("ot1: {0}", ot1.clone().unwrap());
            */

            if best_t_min.is_none() || (ot < best_t_min) {
                best_t_min = ot;
                best_p = Some(points[i].clone());
            }

        }
    }

    if best_p.is_some() {
        println!("best_t: {0}", best_t_min.unwrap());
        *p = best_p.unwrap();
        return true;
    } else {
        return false;
    }
}

fn update_frontier(
    frontier : &mut BTreeSet<Segment>,
    a : &Point,
    b : &Point
) {
    let mut e = Segment{
        org: a.clone(),
        dest: b.clone()
    };
    if frontier.contains(&e) {
        frontier.remove(&e);
    } else {
        e.flip();
        frontier.insert(e);
    }
}



fn get_normal(points : & Vec<Point>, orientation : &mut Number) -> Vector {
    for i in 0..(points.len()-2) {
        let v1 = &points[i] - &points[i+1];
        let v2 = &points[i+1] - &points[i+2];
        let mut normal = v1.cross_product(&v2);
        let iv = Vector::new_from_f64(1., 0., 0.);
        let jv = Vector::new_from_f64(0., 1., 0.);
        *orientation = iv.mixed_product(&jv, &normal);
        if !normal.is_zero() {
            return normal;
        }
    }
    panic!("All points are collinear");
}

fn check_points(normal_ : &Vector, d_ : &Number, points : &Vec<Point> ) {
    //println!("normal: {}\n", normal_);
    //println!("d: {}\n", d_);
    for i in 0..points.len() {
        //println!("point: {} \n", points[i]);
        assert!(is_point_in_plane(&points[i], normal_, d_), "Point {:?} is not co-planar!", points[i]);
    }
}

fn is_point_in_plane(point : &Point, normal : &Vector, d : &Number) -> bool {
    let value = normal.dot_product(&point.get_vector()) - d;
    //println!("value {0}", value);
    return value.is_it_zero()
}

enum NormalType {
    ABC, AB, AC, BC, A, B, C
}

fn classify_normal(n : &Vector) -> NormalType {
    let nx = n.x.is_it_zero();
    let ny = n.y.is_it_zero();
    let nz = n.z.is_it_zero();

    match (nx, ny, nz) {
        (false, false, false) => return NormalType::ABC,
        (false, false, true)  => return NormalType::AB,
        (false, true, false)  => return NormalType::AC,
        (true, false, false)  => return NormalType::BC,
        (false, true, true)   => return NormalType::A,
        (true, false, true)   => return NormalType::B,
        (true, true, false)   => return NormalType::C,
        _ => panic!("Normal vector cannot be zero!")
    }
}

fn get_segment_normal(s : &Segment, normal_ : &Vector, d_ : &Number) -> Line {
    let e : Vector = &s.dest - &s.org;
    let M : Point = s.org.clone() + e.clone()*number::new(1./2.);

    /*
        лежит в плоскости
        n.x*L.x + n.y*L.y + n.z*L.z = d
        ML перпендикулярен e
        e.x*L.x + e.y*L.y + e.z*L.z = e.x*M.x + e.y*M.y + e.z*M.z
        L находится справа от s
        e.x*L.y - e.y*L.x  = e.x*s.org.y - e.y*s.org.x - 1
    */


    let mut a : Matrix<Number> =Matrix::new_from_vector(
        vec![Row::new_from_vector(vec![normal_.x.clone(), normal_.y.clone(), normal_.z.clone()]),
             Row::new_from_vector(vec![e.x.clone(), e.y.clone(), e.z.clone()]),
             Row::new_from_vector(vec![-e.y.clone(), e.x.clone(), number::new(0.)])]);

    let mut y : Row<Number> = Row::new_from_vector(
        vec![d_.clone(),
             &e.x*&M.x + &e.y*&M.y + &e.z*&M.z,
             &e.x*&s.org.y - &e.y*&s.org.x - number::new(1.)]);

    let x = a.solve(y);
    //println!("{}", x);
    let mut vec_L : Vec<Number> = x.convert_to_vec();
    let L = Point::new(vec_L.remove(0), vec_L.remove(0), vec_L.remove(0));

    assert!((&L-&M).dot_product(&(&s.dest - &s.org)).is_it_zero());
    assert!((normal_.dot_product(&L.get_vector()) - d_).is_it_zero());
    assert!((&e.x*(&L.y - &s.org.y) - (&L.x - &s.org.x)*&e.y) == number::new(-1.));

    return Line::new(M, L);
}

fn modify_points(
    points : &mut Vec<Point>,
    nt : &NormalType,
    orientation : &mut Number,
    normal_ : &mut Vector,
    d_ : &mut Number
) {
    match *nt {
        NormalType::ABC => return,
        NormalType::AB => {
            for p in &mut *points {
                p.swap_yz();
                p.swap_xy();
            }
            *normal_ = get_normal(points, orientation);
            *d_ = normal_.dot_product(&points[0].get_vector());
            return;
        },
        NormalType::AC => return,
        NormalType::BC => return,
        NormalType::A => {
            for p in &mut *points {
                p.swap_xz();
                p.swap_xy();
            }
            *normal_ = get_normal(points, orientation);
            *d_ = normal_.dot_product(&points[0].get_vector());
            return;
        }
        NormalType::B => {
            for p in &mut *points {
                p.swap_yz();
                p.swap_xy();
            }
            *normal_ = get_normal(points, orientation);
            *d_ = normal_.dot_product(&points[0].get_vector());
            return;
        }
        NormalType::C => return,
    }
}


#[cfg(test)]
mod tests {
    //use bo::*;
    //use qm::*;
    use std::fs::File;
    use std::collections::BTreeSet;
    use primitives::*;
    use triangulation::incremental_triangulation::triangulate;


    #[test]
    fn triangulation_abc() {
        // x+y+z = 1
        let a = Point::new_from_f64(0.0, 0.0, 1.0);
        let b = Point::new_from_f64(0.5, 0.0, 0.5);
        let c = Point::new_from_f64(1.0, 0.0, 0.0);
        let d = Point::new_from_f64(0.5, 0.5, 0.0);
        let e = Point::new_from_f64(0.0, 1.0, 0.0);

        let one_third = number::new(1.)/number::new(3.);
        let f = Point::new(one_third.clone(), one_third.clone(), one_third);

        let ps : Vec<Point> = vec![a, b, c, d, e, f];

        let ts = triangulate(ps.clone());

        println!("len: {} \n", ts.len());
        println!("vec: {:?} \n", ts);

        assert!(ts.len() == 5);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_abc_.stl").unwrap();

        assert!(mesh.size() == 6);

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }

    #[test]
    fn triangulation_ab() {
        // x+y = 1
        let a = Point::new_from_f64(0.0, 1.0, 0.0);
        let b = Point::new_from_f64(1.0, 0.0, 1.0);
        let c = Point::new_from_f64(-1.0, 2.0, 1.0);
        let d = Point::new_from_f64(1.0, 0.0, -1.0);
        let e = Point::new_from_f64(-1.0, 2.0, -1.0);

        let ps : Vec<Point> = vec![a, b, c, d, e];

        let ts = triangulate(ps.clone());

        println!("len: {} \n", ts.len());
        println!("vec: {:?} \n", ts);

        assert!(ts.len() == 4);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_ab.stl").unwrap();

        assert!(mesh.size() == 5);

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }

    #[test]
    fn triangulation_ac() {
        // x+z = 1
        let a = Point::new_from_f64(-1.0, 0.0, 2.0);
        let b = Point::new_from_f64(1.0, 2.0, 0.0);
        let c = Point::new_from_f64(-2.0, -1.0, 3.0);

        let minus_one_third = number::new(-1.) / number::new(3.);
        let four_third = number::new(4.) / number::new(3.);

        let d = Point::new(minus_one_third.clone(), minus_one_third, four_third);
        let e = Point::new_from_f64(5., -3., -4.);

        let ps : Vec<Point> = vec![a, b, c, d, e];

        let ts = triangulate(ps.clone());

        println!("len: {} \n", ts.len());
        println!("vec: {:?} \n", ts);

        assert!(ts.len() == 4);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_ac.stl").unwrap();

        assert!(mesh.size() == 5);

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }

    #[test]
    fn triangulation_bc() {
        // y+z = 1
        let a = Point::new_from_f64(-1., -1., 2.);
        let b = Point::new_from_f64(1./2., 1./2., 1./2.);
        let c = Point::new_from_f64(2., 0., 1.);
        let d = Point::new_from_f64(3., -1., 2.);
        let e = Point::new_from_f64(5., 1., 0.);

        let ps : Vec<Point> = vec![a, b, c, d, e];

        let ts = triangulate(ps.clone());

        println!("len: {} \n", ts.len());
        println!("vec: {:?} \n", ts);

        assert!(ts.len() == 4);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_bc.stl").unwrap();

        assert!(mesh.size() == 5);

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }

    #[test]
    fn triangulation_a() {
        // y+z = 1
        let a = Point::new_from_f64(1., 2., 0.);
        let b = Point::new_from_f64(1., 0., 2.);
        let c = Point::new_from_f64(1., 0., 0.);
        let d = Point::new_from_f64(1., 2., 2.);
        let e = Point::new_from_f64(1., 1., 3.);

        let ps : Vec<Point> = vec![a, b, c, d, e];

        let ts = triangulate(ps.clone());

        println!("len: {} \n", ts.len());
        println!("vec: {:?} \n", ts);

        assert!(ts.len() == 3);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_a.stl").unwrap();

        assert!(mesh.size() == 5);

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }

    #[test]
    fn triangulation_b() {
        // y = 1
        let a = Point::new_from_f64(-4., 1., 3.);
        let b = Point::new_from_f64(-2., 1., 0.);
        let c = Point::new_from_f64(-1., 1., 2.);

        let one_third = number::new(1.) / number::new(3.);

        let d = Point::new(one_third.clone(), number::new(1.), one_third);
        let e = Point::new_from_f64(5., 1., -1.);

        let ps : Vec<Point> = vec![a, b, c, d, e];

        let ts = triangulate(ps.clone());

        println!("len: {} \n", ts.len());
        println!("vec: {:?} \n", ts);

        assert!(ts.len() == 4);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_b.stl").unwrap();

        assert!(mesh.size() == 5);

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }

    #[test]
    fn triangulation_c() {
        // z = 1
        let a = Point::new_from_f64(1., 1., 1.);
        let b = Point::new_from_f64(1., 2., 1.);
        let c = Point::new_from_f64(-1., 1., 1.);
        let d = Point::new_from_f64(-1., 2., 1.);
        let e = Point::new_from_f64(-1., 3., 1.);
        let f = Point::new_from_f64(1., 3., 1.);

        let ps : Vec<Point> = vec![a, b, c, d, e, f];

        let ts = triangulate(ps.clone());

        println!("len: {} \n", ts.len());
        println!("vec: {:?} \n", ts);

        assert!(ts.len() == 4);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_c.stl").unwrap();

        assert!(mesh.size() == 6);

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }


    #[test]
    fn triangulation_c2_loop() {
        // z = 1
        let a = Point::new_from_f64(-2., 0., 1.);
        let b = Point::new_from_f64(-1., 0., 1.);
        let c = Point::new_from_f64(-1., 1., 1.);
        let d = Point::new_from_f64(1., 1., 1.);
        let e = Point::new_from_f64(2., 0., 1.);
        let f = Point::new_from_f64(1., 0., 1.);
        let g = Point::new_from_f64(0., 3., 1.);


        let ps : Vec<Point> = vec![a, b, c, d, e, f, g];

        let ts = triangulate(ps.clone());

        println!("len: {} \n", ts.len());
        println!("vec: {:?} \n", ts);

        assert!(ts.len() == 7);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_c2_loop.stl").unwrap();


        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }
}