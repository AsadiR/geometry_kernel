use primitives::*;
use triangulation::incremental_triangulation;
use triangulation::ear_clipping_triangulation;

use log::LogLevel;
// use time::PreciseTime;

pub enum TriangulationAlgorithm {
    Incremental
}

pub fn triangulate_ptree3d(mut t: Triangle, mut ss: Vec<Segment>) -> Vec<Triangle> {
    if ss.len() == 0 {
        return vec![t];
    }

    let mut plane: Plane = Plane::new(t.get_normal(), t.get(0));
    let normal_type : NormalType = classify_normal(plane.get_ref_normal());

    check_segments(&plane, &ss);

    map_segments_to_2d(&mut ss, &normal_type);
    map_to_2d(&mut plane, &normal_type);
    map_to_2d(&mut t, &normal_type);

    check_segments(&plane, &ss);

    let iv = Vector::new_from_f64(1., 0., 0.);
    let jv = Vector::new_from_f64(0., 1., 0.);

    let orientation : Number = iv.mixed_product(&jv, plane.get_ref_normal());

    if orientation.is_it_negative() {
        t.get_points_mut_ref().reverse();
    }


    let p_trees: Vec<PolygonTreeNode> = PolygonTreeNode::new_trees(t, ss);
    let mut mapped_ts : Vec<Triangle> = Vec::new();
    for p_tree in p_trees {
        mapped_ts.extend(ear_clipping_triangulation::triangulate2d(p_tree));
    }

    return unmap_ts_new(orientation, normal_type, mapped_ts);
}

fn unmap_ts_new(orientation : Number, normal_type : NormalType, mapped_ts : Vec<Triangle>) -> Vec<Triangle> {
    let mut unmapped_ts : Vec<Triangle> = Vec::new();
    for t in mapped_ts {
        let tr : Triangle;
        if orientation.is_it_positive() {
            tr = Triangle::new(vec![
                map_point_to_3d(&t.get_points_ref()[0], &normal_type),
                map_point_to_3d(&t.get_points_ref()[1], &normal_type),
                map_point_to_3d(&t.get_points_ref()[2], &normal_type)
            ]);
        } else {
            tr = Triangle::new(vec![
                map_point_to_3d(&t.get_points_ref()[1], &normal_type),
                map_point_to_3d(&t.get_points_ref()[0], &normal_type),
                map_point_to_3d(&t.get_points_ref()[2], &normal_type)
            ]);
        }
        unmapped_ts.push(tr);
    }

    return unmapped_ts;
}

fn unmap_ts(orientation : Number, normal_type : NormalType, mapped_ts : Vec<Triangle>) -> Vec<Triangle> {
    let mut unmapped_ts : Vec<Triangle> = Vec::new();
    for t in mapped_ts {
        let tr : Triangle;
        if orientation.is_it_negative() {
            tr = Triangle::new(vec![
                map_point_to_3d(&t.get_points_ref()[0], &normal_type),
                map_point_to_3d(&t.get_points_ref()[1], &normal_type),
                map_point_to_3d(&t.get_points_ref()[2], &normal_type)
            ]);
        } else {
            tr = Triangle::new(vec![
                map_point_to_3d(&t.get_points_ref()[1], &normal_type),
                map_point_to_3d(&t.get_points_ref()[0], &normal_type),
                map_point_to_3d(&t.get_points_ref()[2], &normal_type)
            ]);
        }
        unmapped_ts.push(tr);
    }

    return unmapped_ts;
}

pub fn triangulate3d(mut points : Vec<Point>, mut plane: Plane, alg: TriangulationAlgorithm) -> Vec<Triangle> {
    // all points should be unique! Otherwise algorithm will hang out!
    debug!("Triangulation was started for points:");
    debug!("{:?}", points);

    assert!(points.len() >= 3, "Not enough points. Only {0} points were supplied!", points.len());

    if points.len() ==  3 {
        let mut t = Triangle::new(points);
        let dp = t.get_normal().dot_product(plane.get_ref_normal());
        if dp.is_it_negative() {
            t.reverse();
        }
        return vec![t];
    }

    if log_enabled!(LogLevel::Debug) {
        check_points(&plane, &points);
    }

    let normal_type : NormalType = classify_normal(plane.get_ref_normal());

    map_points_to_2d(&mut points, &normal_type);
    map_to_2d(&mut plane, &normal_type);

    let iv = Vector::new_from_f64(1., 0., 0.);
    let jv = Vector::new_from_f64(0., 1., 0.);
    let orientation : Number = iv.mixed_product(&jv, plane.get_ref_normal());

    debug!("plane: {:?}", plane);
    debug!("d: {0}", plane.get_ref_d().clone().convert_to_f32());

    if log_enabled!(LogLevel::Debug) {
        check_points(&plane, &points);
    }

    let mapped_ts : Vec<Triangle> = match &alg {
        &TriangulationAlgorithm::Incremental => incremental_triangulation::triangulate2d(points, plane),
        // _ => panic!("Wrong triangulation algorithm!")
    };

    return unmap_ts(orientation, normal_type, mapped_ts);
}

fn map_point_to_3d(p : &Point, normal_type : &NormalType) -> Point {
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

pub(crate) fn check_points(plane: &Plane, points : &Vec<Point> ) {
    //println!("normal: {}\n", plane.get_ref_normal());
    //println!("d: {}\n", plane.get_ref_d());
    for i in 0..points.len() {
        //println!("point: {} \n", points[i]);
        assert!(plane.does_it_contain_point(&points[i]), "Point {:?} is not co-planar!", points[i]);
    }
}

pub(crate) fn check_segments(plane: &Plane, segments : &Vec<Segment> ) {
    for i in 0..segments.len() {
        //println!("point: {} \n", segments[i].org));
        //println!("point: {} \n", segments[i].dest);
        assert!(plane.does_it_contain_point(&segments[i].org), "Point {:?} is not co-planar!", segments[i].org);
        assert!(plane.does_it_contain_point(&segments[i].dest), "Point {:?} is not co-planar!", segments[i].dest);
    }
    // println!("Validated");
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

fn map_to_2d<T: To2D>(
    smth : &mut T,
    nt : &NormalType,
) {
    match *nt {
        NormalType::ABC => return,
        NormalType::AB => {
            smth.swap_yz();
            smth.swap_xy();
            return;
        },
        NormalType::AC => return,
        NormalType::BC => return,
        NormalType::A => {
            smth.swap_xz();
            smth.swap_xy();
            return;
        }
        NormalType::B => {
            smth.swap_yz();
            smth.swap_xy();
            return;
        }
        NormalType::C => return,
    }
}

fn map_segments_to_2d(
    segments: &mut Vec<Segment>,
    nt : &NormalType,
) {
    for s in segments.iter_mut() {
        map_to_2d(&mut s.org, nt);
        map_to_2d(&mut s.dest, nt);
    }
}

fn map_points_to_2d(
    points : &mut Vec<Point>,
    nt : &NormalType,
) {
    for p in points.iter_mut() {
        map_to_2d(p, nt);
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use primitives::*;
    use triangulation::*;
    // use env_logger::init  as env_logger_init;

    #[test]
    fn triangulation_abc() {
        // x+y+z = 1
        let a = Point::new_from_f64(0.0, 0.0, 1.0);
        let b = Point::new_from_f64(0.5, 0.0, 0.5);
        let c = Point::new_from_f64(1.0, 0.0, 0.0);
        let d = Point::new_from_f64(0.5, 0.5, 0.0);
        let e = Point::new_from_f64(0.0, 1.0, 0.0);

        let one_third = Number::new(1.)/ Number::new(3.);
        let f = Point::new(one_third.clone(), one_third.clone(), one_third);

        let plane = Plane::new_3p(&a, &b, &d);
        let ps : Vec<Point> = vec![a, b, c, d, e, f];

        let ts = triangulate3d(ps.clone(), plane, TriangulationAlgorithm::Incremental);

        info!("len: {} \n", ts.len());
        info!("vec: {:?} \n", ts);

        assert!(ts.len() == 5);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_abc_.stl").unwrap();

        assert!(mesh.num_of_points() == 6);

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

        let plane = Plane::new_3p(&a, &b, &c);
        let ps : Vec<Point> = vec![a, b, c, d, e];

        let ts = triangulate3d(ps.clone(), plane, TriangulationAlgorithm::Incremental);

        info!("len: {} \n", ts.len());
        info!("vec: {:?} \n", ts);

        assert!(ts.len() == 4);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_ab.stl").unwrap();

        assert!(mesh.num_of_points() == 5);

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

        let minus_one_third = Number::new(-1.) / Number::new(3.);
        let four_third = Number::new(4.) / Number::new(3.);

        let d = Point::new(minus_one_third.clone(), minus_one_third, four_third);
        let e = Point::new_from_f64(5., -3., -4.);

        let plane = Plane::new_3p(&a, &b, &d);
        let ps : Vec<Point> = vec![a, b, c, d, e];

        let ts = triangulate3d(ps.clone(), plane, TriangulationAlgorithm::Incremental);

        info!("len: {} \n", ts.len());
        info!("vec: {:?} \n", ts);

        assert!(ts.len() == 4);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_ac.stl").unwrap();

        assert!(mesh.num_of_points() == 5);

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

        let plane = Plane::new_3p(&a, &b, &c);
        let ps : Vec<Point> = vec![a, b, c, d, e];

        let ts = triangulate3d(ps.clone(), plane, TriangulationAlgorithm::Incremental);;

        info!("len: {} \n", ts.len());
        info!("vec: {:?} \n", ts);

        assert!(ts.len() == 4);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_bc.stl").unwrap();

        assert!(mesh.num_of_points() == 5);

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

        let plane = Plane::new_3p(&a, &b, &c);
        let ps : Vec<Point> = vec![a, b, c, d, e];

        let ts = triangulate3d(ps.clone(), plane, TriangulationAlgorithm::Incremental);

        info!("len: {} \n", ts.len());
        info!("vec: {:?} \n", ts);

        assert!(ts.len() == 3);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_a.stl").unwrap();

        assert!(mesh.num_of_points() == 5);

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

        let one_third = Number::new(1.) / Number::new(3.);

        let d = Point::new(one_third.clone(), Number::new(1.), one_third);
        let e = Point::new_from_f64(5., 1., -1.);

        let plane = Plane::new_3p(&a, &b, &c);
        let ps : Vec<Point> = vec![a, b, c, d, e];

        let ts = triangulate3d(ps.clone(), plane, TriangulationAlgorithm::Incremental);

        info!("len: {} \n", ts.len());
        info!("vec: {:?} \n", ts);

        assert!(ts.len() == 4);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_b.stl").unwrap();

        assert!(mesh.num_of_points() == 5);

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

        let plane = Plane::new_3p(&a, &b, &c);
        let ps : Vec<Point> = vec![a, b, c, d, e, f];

        let ts = triangulate3d(ps.clone(), plane, TriangulationAlgorithm::Incremental);

        info!("len: {} \n", ts.len());
        info!("vec: {:?} \n", ts);

        assert!(ts.len() == 4);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);


        let mut f = File::create("res_of_tests/inc_tr/test_c.stl").unwrap();

        assert!(mesh.num_of_points() == 6);

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }


    #[test]
    fn triangulation_c2_loop() {
        let a = Point::new_from_f64(-2., 0., 1.);
        let b = Point::new_from_f64(-1., 0., 1.);
        let c = Point::new_from_f64(-1., 1., 1.);
        let d = Point::new_from_f64(1., 1., 1.);
        let e = Point::new_from_f64(2., 0., 1.);
        let f = Point::new_from_f64(1., 0., 1.);
        let g = Point::new_from_f64(0., 3., 1.);

        let plane = Plane::new_3p(&a, &b, &c);
        let ps : Vec<Point> = vec![a, b, c, d, e, f, g];

        let ts = triangulate3d(ps.clone(), plane, TriangulationAlgorithm::Incremental);

        info!("len: {} \n", ts.len());
        info!("vec: {:?} \n", ts);

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