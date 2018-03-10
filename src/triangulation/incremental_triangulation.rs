use std::collections::BTreeSet;
use primitives::*;
use matrix::*;
use intersect::line_x_line;
// use time::PreciseTime;

pub fn triangulate2d(mut points : Vec<Point>, plane : Plane) -> Vec<Triangle> {
    let e : Segment = hull_edge(&mut points);

    let mut ts : Vec<Triangle> = Vec::new();

    debug!("hull_edge_res = {}\n", e);

    let mut p = Point::new_from_f64(0.,0.,0.);
    let mut frontier : BTreeSet<Segment> = BTreeSet::new();
    frontier.insert(e);

    while !frontier.is_empty() {
        let e = frontier.iter().next_back().unwrap().clone();

        debug!("frontie {:?}", frontier);
        frontier.remove(&e);
        debug!("edge: {:?}", e);
        if mate(&e, &mut points, &mut p, &plane) {
            debug!("mate_point = {}", p);

            update_frontier(&mut frontier, &p, &e.org);
            update_frontier(&mut frontier, &e.dest, &p);
            let tr = Triangle::new(vec![
                e.org.clone(),
                e.dest.clone(),
                p.clone()
            ]);
            ts.push(tr);
        }
    }
    return ts;
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
    plane: &Plane
) -> bool {

    let mut best_p : Option<Point> = None;
    let mut best_t_min : Option<Number> = None;

    let f : Line = get_segment_normal(e, plane);

    for i in 0..points.len() {
        let c = points[i].classify(&e.org, &e.dest);
        if c == point::EPointPosition::Right {
            let cur_s = Segment::new(e.dest.clone(), points[i].clone());
            let g : Line = get_segment_normal(&cur_s, plane);
            let ot = line_x_line::intersect_p(&f, &g);

            //println!("f {:?}", f);
            //println!("g {:?}", g);
            assert!(ot.is_some());

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
        debug!("best_t: {0}", best_t_min.unwrap());
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

fn get_segment_normal(s : &Segment, plane: &Plane) -> Line {
    let e : Vector = &s.dest - &s.org;
    let point_m : Point = s.org.clone() + e.clone()* Number::new(1./2.);

    /*
        лежит в плоскости
        n.x*point_l.x + n.y*point_l.y + n.z*point_l.z = -d
        ML перпендикулярен e
        e.x*point_l.x + e.y*point_l.y + e.z*point_l.z = e.x*point_m.x + e.y*point_m.y + e.z*point_m.z
        point_l находится справа от s
        e.x*point_l.y - e.y*point_l.x  = e.x*s.org.y - e.y*s.org.x - 1
    */
    let normal = plane.get_ref_normal();
    let d = plane.get_ref_d();

    let mut a : Matrix<Number> = Matrix::new_from_vector(
        vec![Row::new_from_vector(vec![normal.x.clone(), normal.y.clone(), normal.z.clone()]),
             Row::new_from_vector(vec![e.x.clone(), e.y.clone(), e.z.clone()]),
             Row::new_from_vector(vec![-e.y.clone(), e.x.clone(), Number::new(0.)])]);

    let y : Row<Number> = Row::new_from_vector(
        vec![-d.clone(),
             &e.x*&point_m.x + &e.y*&point_m.y + &e.z*&point_m.z,
             &e.x*&s.org.y - &e.y*&s.org.x - Number::new(1.)]);

    let x = a.solve(y);
    //println!("{}", x);
    let mut vec_l : Vec<Number> = x.convert_to_vec();
    let point_l = Point::new(vec_l.remove(0), vec_l.remove(0), vec_l.remove(0));

    assert!((&point_l-&point_m).dot_product(&(&s.dest - &s.org)).is_it_zero());
    assert!(plane.does_it_contain_point(&point_l));
    assert_eq!((&e.x*(&point_l.y - &s.org.y) - (&point_l.x - &s.org.x)*&e.y), Number::new(-1.));

    debug!("plane: {:?}", plane);
    debug!("d: {0}", plane.get_ref_d().clone().convert_to_f32());
    return Line::new(point_m, point_l);
}