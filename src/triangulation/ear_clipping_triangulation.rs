use std::collections::{HashMap, BTreeSet};
use primitives::*;
use primitives::point::EPointPosition;
// use log::LogLevel;
// use time::PreciseTime;


pub fn triangulate2d(polygon_tree : PolygonTreeNode) -> Vec<Triangle> {
    /*
    На вход алгоритм получает дерево вложенных кривых
    Для каждой пары родитель ребенок выполняем простую ear-clipping триангуляцию.
    Все треугольники складываю в вектор.

    Простая триангуляция:
    Если есть дырки, то добавляем фиктивные ребра.
    строим списки: ears, convex_vs, reflex_vs
    loop:
        Достаем ухо и удаляем его.
        Обновляем соседей во всех трех списках.

    */
    let mut ts : Vec<Triangle> = Vec::new();

    let mut nodes : Vec<PolygonTreeNode> = vec![polygon_tree];
    while !nodes.is_empty() {
        let cur_node = nodes.pop().unwrap();
        let cur_polygon = cur_node.get_cur_polygon();
        let child_polygons = cur_node.get_child_polygons();
        let local_ts = simple_triangulation(cur_polygon, child_polygons);
        ts.extend(local_ts);
        nodes.extend(cur_node.split_tree());
    }

    return ts;
}


fn update_convex_and_reflex_vs(
    cur_index: &usize,
    convex_vs : &mut BTreeSet<usize>,
    reflex_vs : &mut BTreeSet<usize>,
    boundary_points: &HashMap<usize, Point>,
    cur_to_prev : &HashMap<usize, usize>,
    cur_to_next : &HashMap<usize, usize>
) {
    // let n = boundary_points.len();
    let prev_index = cur_to_prev[cur_index];
    let next_index = cur_to_next[cur_index];

    // println!("{3} | {0}, {1}, {2}", prev_index, cur_index, next_index, n);

    let c_res = boundary_points[&next_index].classify(&boundary_points[&prev_index], &boundary_points[&cur_index]);
    if c_res == EPointPosition::Left {
        if reflex_vs.contains(&cur_index) {
            reflex_vs.remove(&cur_index);
        }
        convex_vs.insert(*cur_index);
    } else {
        if convex_vs.contains(&cur_index) {
            convex_vs.remove(&cur_index);
        }
        reflex_vs.insert(*cur_index);
    }
}

fn update_ears(
    cur_index: &usize,
    ears : &mut BTreeSet<usize>,
    reflex_vs : &BTreeSet<usize>,
    boundary_points: &HashMap<usize, Point>,
    cur_to_prev : &HashMap<usize, usize>,
    cur_to_next : &HashMap<usize, usize>
) {
    if reflex_vs.contains(cur_index) {
        //println!("point_reflex = {0}", boundary_points[cur_index]);
        return;
    }
    //println!();
    //println!("point_to_update = {0}", boundary_points[cur_index]);


    // let n = boundary_points.len();
    let prev_index_of_cv = cur_to_prev[cur_index];
    let next_index_of_cv = cur_to_next[cur_index];



    let t = Triangle::new(vec![
        boundary_points[&prev_index_of_cv].clone(),
        boundary_points[cur_index].clone(),
        boundary_points[&next_index_of_cv].clone()
    ]);

    //println!("{:?}", t);


    let mut ear = true;
    for index_of_rv in reflex_vs.iter() {
        if boundary_points[index_of_rv] == boundary_points[&prev_index_of_cv] ||
           boundary_points[index_of_rv] == boundary_points[&next_index_of_cv]
        {
            continue;
        }

        ear = ear && !t.does_triangle_contain_point(&boundary_points[index_of_rv], false);
        //println!("point_reflex = {0}", boundary_points[index_of_rv]);
        //println!("ear = {0}", ear);
    }

    if ear {
        ears.insert(*cur_index);
    } else {
        ears.remove(cur_index);
    }
}

fn polygon_to_point_xmax(p : &Polygon) -> Point {
    let mut opt_max_x : Option<Number> = Option::None;
    let mut opt_max_p : Option<Point> = Option::None;
    for point in p.get_points_ref().iter() {
        if opt_max_x.is_none() || opt_max_x.clone().unwrap() < point.x {
            opt_max_x = Some(point.x.clone());
            opt_max_p = Some(point.clone());
        }
    }
    return opt_max_p.unwrap();
}


fn get_edges(
    boundary_points : &HashMap<usize, Point>,
    cur_to_next : &mut HashMap<usize, usize>
) -> Vec<Segment> {
    let mut res : Vec<Segment> = Vec::new();
    let n = boundary_points.len();
    for i in 0..n {
        let next_index = cur_to_next[&i];
        res.push(Segment::new(boundary_points[&i].clone(), boundary_points[&next_index].clone()))
    }
    return res;
}

fn update_convex_and_reflex_for_all(
    convex_vs : &mut BTreeSet<usize>,
    reflex_vs : &mut BTreeSet<usize>,
    boundary_points: &HashMap<usize, Point>,
    cur_to_prev : &mut HashMap<usize, usize>,
    cur_to_next : &mut HashMap<usize, usize>
) {
    let n = boundary_points.len();
    for i in 0..n {
        cur_to_prev.insert(i, (i + n - 1) % n);
        cur_to_next.insert(i, (i + 1) % n);

        update_convex_and_reflex_vs(&i, convex_vs, reflex_vs, boundary_points, cur_to_prev, cur_to_next);
    }
}

fn simple_triangulation(boundary: Polygon, holes: Vec<Polygon>) -> Vec<Triangle> {
    /*
    Обход против часовой стрелки.
    Для reflex вершины с индексом i: v_(i+1) лежит справа от вектора (v_(i-1), v_i),
    иначе вершина convex. Для классификации нужно использовать метод classify!

    Вершина является ухом если:
    1. Она convex
    2. Внутри треугольника нет других точек. Достаточно проверять только reflex точки.

    Направление обхода на границе должно быть противоположным направлению обхода в дырах.
    Расширяем boundary: добавляем соединяющее ребро и дыру в обратном порядке.

    */
    //println!("boundary: {:?}", &boundary);

    fn key(&(ref point_xmax, _) : &(Point, Polygon)) -> Number {
        return point_xmax.x.clone()
    }

    let mut ts : Vec<Triangle> = Vec::new();
    let mut ears : BTreeSet<usize> = BTreeSet::new();
    let mut convex_vs : BTreeSet<usize> = BTreeSet::new();
    let mut reflex_vs : BTreeSet<usize> = BTreeSet::new();

    let mut cur_to_prev : HashMap<usize, usize> = HashMap::new();
    let mut cur_to_next : HashMap<usize, usize> = HashMap::new();

    let mut boundary_points: HashMap<usize, Point> = HashMap::new();
    for (i, p) in boundary.get_points().into_iter().enumerate() {
        boundary_points.insert(i, p);
    }
    update_convex_and_reflex_for_all(&mut convex_vs, &mut reflex_vs, &boundary_points, &mut cur_to_prev, &mut cur_to_next);


    let mut m_and_hole : Vec<(Point, Polygon)> = Vec::new();

    for p in holes {
        m_and_hole.push((polygon_to_point_xmax(&p), p));
    }

    m_and_hole.sort_unstable_by_key(key);
    m_and_hole.reverse();


    let zero = Number::new(0.);
    let one = Number::new(1.);
    let minus_one = Number::new(-1.);

    for (pm, hole) in m_and_hole {
        //println!("\nNext hole!");

        let mut opt_nice_pi : Option<Point> = Option::None;
        let mut nice_t = zero.clone();
        let mut opt_nice_s : Option<Segment> = Option::None;

        let visible_point : Point;

        for s in get_edges(&boundary_points, &mut cur_to_next) {
            // ищем пересечение ребра s с положительным x-лучем с началом в pm
            let dir = &s.dest - &s.org;
            //println!("dir {0}", dir);

            if dir.y == zero {
                // ребро параллельно лучу, пересечений нет.
                continue;
            }

            let t = (&pm.y - &s.org.y) / &dir.y;
            //println!("t {0}", t);

            if t >= zero && t <= one {
                let pi = &s.org + &(&dir*t.clone());
                if pi.x < pm.x {
                    continue;
                }

                // println!("{:?}", opt_nice_pi);

                if opt_nice_pi.is_none() || pi.x < opt_nice_pi.clone().unwrap().x {
                    opt_nice_pi = Some(pi);
                    nice_t = t;
                    opt_nice_s = Some(s);
                }
            }
        }

        //println!("pm {0}", pm);
        //println!("hole {:?}", hole);
        assert!(opt_nice_pi.is_some());
        let pi = opt_nice_pi.unwrap();
        //println!("pi {0}", pi);
        //println!("nice_t {0}", nice_t);

        if nice_t == zero || nice_t == one {
            // видимая точка для pm - это pi
            visible_point = pi;
        } else {
            let s = opt_nice_s.unwrap();
            let pp = if s.org.x > s.dest.x {
                s.org
            } else {
                s.dest
            };
            let t = Triangle::new(vec![pm.clone(), pi.clone(), pp.clone()]);

            let mut ps_in_t : Vec<Point> = Vec::new();

            //println!("reflex_vs.len() = {0}", reflex_vs.len());
            for cur_index_of_rv in reflex_vs.iter() {
                let cur_point = boundary_points[cur_index_of_rv].clone();
                if cur_point != pp && t.does_triangle_contain_point(&cur_point, false) {
                    ps_in_t.push(cur_point);
                }
            }
            //println!("ps_in_t.len() = {0}", ps_in_t.len());
            if ps_in_t.len() == 0 {
                // видимая точка для pm - это pp
                visible_point = pp;
            } else {
                let get_signed_cos2 = |pr : &Point| -> Number {
                    let pm_pr : Vector = pr - &pm;
                    let mut cos2 = &pm_pr.x*&pm_pr.x/pm_pr.length2();
                    if pm_pr.x < zero {
                        cos2 = cos2 * &minus_one;
                    }
                    return cos2;
                };

                let mut cos2 = get_signed_cos2(&pp);
                let mut length2 = (&pp - &pm).length2();
                let mut pr = pp.clone();

                for p in ps_in_t {
                    let new_cos2 = get_signed_cos2(&p);
                    let new_length2 = (&p - &pm).length2();
                    if new_cos2 > cos2 {
                        cos2 = new_cos2;
                        length2 = new_length2;
                        pr = p;
                    } else if new_cos2 == cos2 && new_length2 < length2 {
                        length2 = new_length2;
                        pr = p;
                    }
                }

                assert!(pr != pp);
                // видимая точка для pm - это pr

                // println!("pr!!");
                visible_point = pr;
            }
        }

        //println!("visible_point for {0} is {1}", pm, visible_point);

        // TODO fix
        // добавление в boundary_points новых ребер

        let mut n = 0;
        let mut hole_added = false;

        let mut boundary_points_new: HashMap<usize, Point> = HashMap::new();
        for i in 0..boundary_points.len() {
            let p = &boundary_points[&i];

            boundary_points_new.insert(n, p.clone());
            n += 1;

            if p == &visible_point && !hole_added {
                let mut other : Vec<Point> = Vec::new();
                let mut go_flag = false;

                for p_from_hole in hole.get_points_ref().iter().rev() {
                    if p_from_hole == &pm {
                        go_flag = true;
                    }

                    if go_flag {
                        boundary_points_new.insert(n, p_from_hole.clone());
                        n += 1;
                    } else {
                        other.push(p_from_hole.clone());
                    }
                }

                for p_from_hole in other {
                    boundary_points_new.insert(n, p_from_hole);
                    n += 1;
                }

                boundary_points_new.insert(n, pm.clone());
                n += 1;
                boundary_points_new.insert(n, visible_point.clone());
                n += 1;
                hole_added = true;
            }
        }

        boundary_points = boundary_points_new;
        update_convex_and_reflex_for_all(&mut convex_vs, &mut reflex_vs, &boundary_points, &mut cur_to_prev, &mut cur_to_next);
    }

    for cur_index_of_cv in convex_vs.iter() {
        update_ears(cur_index_of_cv, &mut ears, &reflex_vs, &boundary_points, &cur_to_prev, &cur_to_next);
    }

    /*
    for i in 0..boundary_points.len() {
        println!("bp {0}", boundary_points[&i]);
    }
    */

    //println!();
    while boundary_points.len() >= 3 {
        //println!("len boundary_points = {0}", boundary_points.len());
        //println!("len_ears {0}, len_convex_vs {1}, len_reflex_vs {2}", ears.len(), convex_vs.len(), reflex_vs.len());

        /*
        if ears.len() == 0 {
            let mut mesh = Mesh::new();
            mesh.add_triangles(ts);

            let mut f = File::create("res_of_tests/ear_cl_tr/test.stl").unwrap();

            match mesh.write_stl(&mut f) {
                Ok(_) => (),
                Err(_) => panic!()
            };

            println!("convex {:?}", convex_vs);
            println!("reflex {:?}", reflex_vs);

            let mut keys : Vec<usize> = Vec::new();
            keys.extend(boundary_points.keys().clone());
            keys.sort();
            for i in keys {
                println!("key {0} bp {1}", i, boundary_points[&i]);
            }
            panic!();
        }
        */

        let ear_index = ears.iter().next().unwrap().clone();
        //println!("ear {0}", boundary_points[&ear_index]);

        ears.remove(&ear_index);
        convex_vs.remove(&ear_index);
        let prev_index = cur_to_prev[&ear_index];
        let next_index = cur_to_next[&ear_index];
        let t = Triangle::new(vec![
            boundary_points[&prev_index].clone(),
            boundary_points[&ear_index].clone(),
            boundary_points[&next_index].clone()
        ]);
        //println!("t {:?}", t);
        ts.push(t);
        boundary_points.remove(&ear_index);
        cur_to_prev.insert(next_index, prev_index);
        cur_to_next.insert(prev_index, next_index);

        update_convex_and_reflex_vs(&prev_index, &mut convex_vs, &mut reflex_vs, &boundary_points, &cur_to_prev, &cur_to_next);
        update_convex_and_reflex_vs(&next_index, &mut convex_vs, &mut reflex_vs, &boundary_points, &cur_to_prev, &cur_to_next);

        for i in convex_vs.iter() {
            update_ears(i, &mut ears, &reflex_vs, &boundary_points, &cur_to_prev, &cur_to_next);
        }
        // update_ears(&prev_index, &mut ears, &reflex_vs, &boundary_points, &cur_to_prev, &cur_to_next);
        // update_ears(&next_index, &mut ears, &reflex_vs, &boundary_points, &cur_to_prev, &cur_to_next);

        //println!();
    }

    //println!("len ts {0}", ts.len());
    return ts;
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use primitives::*;
    use triangulation::*;
    // use env_logger::init  as env_logger_init;

    #[test]
    fn triangulation_without_holes1() {
        let a = Point::new_from_f64(1.0, 0.5, 0.0);
        let b = Point::new_from_f64(1.0, 1.0, 0.0);
        let c = Point::new_from_f64(0.5, 0.5, 0.0);
        let d = Point::new_from_f64(0.5, 1.0, 0.0);
        let e = Point::new_from_f64(0.0, 0.0, 0.0);
        let f = Point::new_from_f64(1.0, 0.0, 0.0);
        let g = Point::new_from_f64(0.25, 0.25, 0.0);

        let p : Polygon = Polygon::new(vec![a, b, c, d, e, f, g], Vector::new_from_f64(0., 0., 1.));
        let p_tree : PolygonTreeNode = PolygonTreeNode::new(p);

        let ts = ear_clipping_triangulation::triangulate2d(p_tree);

        //info!("len: {} \n", ts.len());
        //info!("vec: {:?} \n", ts);

        // assert!(ts.len() == 5);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);

        let mut f = File::create("res_of_tests/ear_cl_tr/test_without_holes1.stl").unwrap();

        // assert!(mesh.num_of_points() == 6);

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }

    #[test]
    fn triangulation_square_with_square_hole() {
        let a = Point::new_from_f64(1.0, 0.0, 0.0);
        let b = Point::new_from_f64(1.0, 1.0, 0.0);
        let c = Point::new_from_f64(0.0, 1.0, 0.0);
        let d = Point::new_from_f64(0.0, 0.0, 0.0);

        let e = Point::new_from_f64(0.25, 0.25, 0.0);
        let f = Point::new_from_f64(0.75, 0.25, 0.0);
        let g = Point::new_from_f64(0.75, 0.75, 0.0);
        let h = Point::new_from_f64(0.25, 0.75, 0.0);

        let outer : Polygon = Polygon::new(vec![a, b, c, d], Vector::new_from_f64(0., 0., 1.));
        let inner : Polygon = Polygon::new(vec![e, f, g, h], Vector::new_from_f64(0., 0., 1.));

        let mut p_tree : PolygonTreeNode = PolygonTreeNode::new(outer);
        p_tree.add_children(vec![PolygonTreeNode::new(inner)]);

        let ts = ear_clipping_triangulation::triangulate2d(p_tree);

        //info!("len: {} \n", ts.len());
        //info!("vec: {:?} \n", ts);

        assert_eq!(ts.len(), 10);

        // assert!(ts.len() == 5);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);

        let mut f = File::create("res_of_tests/ear_cl_tr/test_square_with_square_hole.stl").unwrap();

        // assert!(mesh.num_of_points() == 6);

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }

    #[test]
    fn test_of_visible_point_first_case() {
        let a = Point::new_from_f64(1.0, 0.0, 0.0);
        let b = Point::new_from_f64(2.0, 0.5, 0.0);
        let c = Point::new_from_f64(1.0, 1.0, 0.0);
        let d = Point::new_from_f64(0.0, 1.0, 0.0);
        let e = Point::new_from_f64(0.0, 0.0, 0.0);


        let f = Point::new_from_f64(0.25, 0.25, 0.0);
        let g = Point::new_from_f64(0.75, 0.5, 0.0);
        let h = Point::new_from_f64(0.25, 0.75, 0.0);

        let outer : Polygon = Polygon::new(vec![a, b, c, d, e], Vector::new_from_f64(0., 0., 1.));
        let inner : Polygon = Polygon::new(vec![f, g, h], Vector::new_from_f64(0., 0., 1.));

        let mut p_tree : PolygonTreeNode = PolygonTreeNode::new(outer);
        p_tree.add_children(vec![PolygonTreeNode::new(inner)]);

        let ts = ear_clipping_triangulation::triangulate2d(p_tree);

        //info!("len: {} \n", ts.len());
        //info!("vec: {:?} \n", ts);

        assert_eq!(ts.len(), 9);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);

        let mut f = File::create("res_of_tests/ear_cl_tr/test_of_visible_point_first_case.stl").unwrap();

        // assert!(mesh.num_of_points() == 6);

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }

    #[test]
    fn test_of_visible_point_third_case() {
        let a = Point::new_from_f64(0.0, 0.0, 0.0);
        let b = Point::new_from_f64(3.0, 0.0, 0.0);
        let c = Point::new_from_f64(5.0, 4.0, 0.0);
        let d = Point::new_from_f64(4.5, 3.5, 0.0);
        let e = Point::new_from_f64(4.0, 3.0, 0.0);
        let f = Point::new_from_f64(2.0, 4.0, 0.0);
        let g = Point::new_from_f64(0.0, 4.0, 0.0);

        let h = Point::new_from_f64(1.0, 1.0, 0.0);
        let i = Point::new_from_f64(2.0, 2.0, 0.0);
        let j = Point::new_from_f64(1.0, 3.0, 0.0);

        let outer : Polygon = Polygon::new(vec![a, b, c, d, e, f, g], Vector::new_from_f64(0., 0., 1.));
        let inner : Polygon = Polygon::new(vec![h, i, j], Vector::new_from_f64(0., 0., 1.));

        let mut p_tree : PolygonTreeNode = PolygonTreeNode::new(outer);
        p_tree.add_children(vec![PolygonTreeNode::new(inner)]);

        let ts = ear_clipping_triangulation::triangulate2d(p_tree);

        assert_eq!(ts.len(), 11);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);

        let mut f = File::create("res_of_tests/ear_cl_tr/test_of_visible_point_third_case.stl").unwrap();

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }

    #[test]
    fn triangulation_hard_test() {
        let p1 = Point::new_from_f64(0.0, 0.0, 0.0);
        let p2 = Point::new_from_f64(30.0, 0.0, 0.0);
        let p3 = Point::new_from_f64(0.0, 6.0, 0.0);


        let p4 = Point::new_from_f64(3.0, 2.0, 0.0);
        let p5 = Point::new_from_f64(5.0, 1.0, 0.0);
        let p6 = Point::new_from_f64(7.0, 2.0, 0.0);
        let p7 = Point::new_from_f64(5.0, 4.0, 0.0);
        let p8 = Point::new_from_f64(3.0, 4.0, 0.0);
        let p9 = Point::new_from_f64(3.0, 3.0, 0.0);

        let p10 = Point::new_from_f64(4.0, 2.0, 0.0);
        let p11 = Point::new_from_f64(5.0, 2.0, 0.0);
        let p12 = Point::new_from_f64(5.0, 3.0, 0.0);
        let p13 = Point::new_from_f64(4.0, 3.0, 0.0);

        let p14 = Point::new_from_f64(8.0, 1.0, 0.0);
        let p15 = Point::new_from_f64(10.0, 1.0, 0.0);
        let p16 = Point::new_from_f64(9.0, 2.0, 0.0);
        let p17 = Point::new_from_f64(10.0, 3.0, 0.0);
        let p18 = Point::new_from_f64(8.0, 3.0, 0.0);

        let p19 = Point::new_from_f64(11.0, 1.0, 0.0);
        let p20 = Point::new_from_f64(13.0, 1.0, 0.0);
        let p21 = Point::new_from_f64(13.0, 2.0, 0.0);


        let pl1 : Polygon = Polygon::new(vec![p1, p2, p3], Vector::new_from_f64(0., 0., 1.));
        let pl11 : Polygon = Polygon::new(vec![p4, p5, p6, p7, p8, p9], Vector::new_from_f64(0., 0., 1.));
        let pl111 : Polygon = Polygon::new(vec![p10, p11, p12, p13], Vector::new_from_f64(0., 0., 1.));
        let pl12 : Polygon = Polygon::new(vec![p14, p15, p16, p17, p18], Vector::new_from_f64(0., 0., 1.));
        let pl13 : Polygon = Polygon::new(vec![p19, p20, p21], Vector::new_from_f64(0., 0., 1.));

        let mut pn1 : PolygonTreeNode = PolygonTreeNode::new(pl1);
        let mut pn11 : PolygonTreeNode = PolygonTreeNode::new(pl11);

        pn11.add_children(vec![PolygonTreeNode::new(pl111)]);
        pn1.add_children(vec![
            pn11,
            PolygonTreeNode::new(pl12),
            PolygonTreeNode::new(pl13)
        ]);

        let ts = ear_clipping_triangulation::triangulate2d(pn1);
        // println!("len: {} \n", ts.len());
        assert_eq!(ts.len(), 37);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);

        let mut f = File::create("res_of_tests/ear_cl_tr/hard_test.stl").unwrap();

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }

    /*
    TODO:
    Отладить поиск взаимных точек для всех случаев!
    Добавить тест с четырьмя вложенными фигурами.
    Сделать отображение для дерева
    */
}
