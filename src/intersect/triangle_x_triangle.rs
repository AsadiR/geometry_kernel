use primitives::*;
use intersect::{plane_x_plane, line_x_segment, segment_x_segment};
use intersect::point_wrapper::PointWrapper;
use std::collections::BTreeSet;
use std::collections::HashMap;

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq,Eq)]
pub enum InfoTxT {
    Collinear,
    NotIntersecting,
    CoplanarIntersecting,
    CoplanarNotIntersecting,
    Intersecting,
    IntersectingInAPoint
}

impl InfoTxT {
    pub fn does_it_intersecting(&self) -> bool {
        return (*self == InfoTxT::Intersecting) || (*self == InfoTxT::CoplanarIntersecting) || (*self == InfoTxT::IntersectingInAPoint);
    }
}

#[derive(Clone)]
pub struct ResTxT {
    point: Option<Point>,
    segment: Option<Segment>,
    polygon: Option<Polygon>,
    info: InfoTxT
}

impl ResTxT {
    pub fn new(point: Option<Point>, segment: Option<Segment>, polygon: Option<Polygon>, info: InfoTxT) -> ResTxT {
        ResTxT {
            point: point,
            segment: segment,
            polygon: polygon,
            info: info
        }
    }

    pub fn get_tuple(self) -> (Option<Point>, Option<Segment>, Option<Polygon>, InfoTxT) {
        return (self.point, self.segment, self.polygon, self.info)
    }

    pub fn get_point(self) -> Point {
        return self.point.unwrap();
    }

    pub fn get_segment(self) -> Segment {
        return self.segment.unwrap();
    }

    pub fn get_polygon(self) -> Polygon {
        return self.polygon.unwrap();
    }

    pub fn get_info(&self) -> InfoTxT {
        self.info.clone()
    }
}


pub fn intersect(tr1 : &Triangle, tr2 : &Triangle) -> ResTxT {
    let plane2 = tr2.gen_plane();

    if plane2.normal.is_zero() {
        panic!("plane normal is a zero vector!")
    }

    let dist1 = signed_distance(tr1.get_ref(0), &plane2);
    let dist2 = signed_distance(tr1.get_ref(1), &plane2);
    let dist3 = signed_distance(tr1.get_ref(2), &plane2);

    if dist1.is_it_zero() & dist2.is_it_zero() & dist3.is_it_zero() {
        let polygon = intersect_triangles_in_the_plane(tr1, tr2);
        if polygon.points.len() == 0 {
            return ResTxT::new(None, None, None, InfoTxT::CoplanarNotIntersecting);
        } else if polygon.points.len() == 1 {
            let Polygon {mut points, normal} = polygon;
            return ResTxT::new(Some(points.remove(0)), None, None, InfoTxT::IntersectingInAPoint);
        } else if polygon.points.len() == 2 {
            let Polygon {mut points, normal} = polygon;
            let os = Some(Segment::new(points.remove(0), points.remove(0)));
            return ResTxT::new(None, os, None, InfoTxT::Intersecting);
        }  else {
            return ResTxT::new(None, None, Some(polygon), InfoTxT::CoplanarIntersecting);
        }
    }

    if (dist1 == dist2) & (dist1 == dist3) {
        return ResTxT::new(None, None, None, InfoTxT::Collinear);
    }

    if dist1.is_it_positive() & dist2.is_it_positive() & dist3.is_it_positive() |
        dist1.is_it_negative() & dist2.is_it_negative() & dist3.is_it_negative() {
        return ResTxT::new(None, None, None, InfoTxT::NotIntersecting);
    }

    let plane1 = tr1.gen_plane();

    let dist1 = signed_distance(tr2.get_ref(0), &plane1);
    let dist2 = signed_distance(tr2.get_ref(1), &plane1);
    let dist3 = signed_distance(tr2.get_ref(2), &plane1);

    if dist1.is_it_positive() & dist2.is_it_positive() & dist3.is_it_positive() |
        dist1.is_it_negative() & dist2.is_it_negative() & dist3.is_it_negative() {
        return ResTxT::new(None, None, None, InfoTxT::NotIntersecting);
    }

    let (op_line, _) = plane_x_plane::intersect(&plane1, &plane2);
    let line = op_line.unwrap();

    let (op1, os1) = intersect_line_and_triangle(&line, &tr1);
    let (op2, os2) = intersect_line_and_triangle(&line, &tr2);


    match (op1, os1, op2, os2) {
        (Some(p1), None, Some(p2), None) => {
            if p1 == p2 {
                return ResTxT::new(Some(p1), None, None, InfoTxT::IntersectingInAPoint);
            } else {
                return ResTxT::new(None, None, None, InfoTxT::NotIntersecting);
            }
        },
        (None, Some(s1), None, Some(s2)) => {
            let res = segment_x_segment::intersect_segments_on_the_line(&s1, &s2);
            match res {
                (None, Some(s), segment_x_segment::InfoSxS::IntersectingInASegment) => {
                    return ResTxT::new(None, Some(s), None, InfoTxT::Intersecting);
                },
                (Some(p), None, segment_x_segment::InfoSxS::IntersectingInAPointOnLine) => {
                    return ResTxT::new(Some(p), None, None, InfoTxT::IntersectingInAPoint);
                }
                _ => return ResTxT::new(None, None, None, InfoTxT::NotIntersecting)
            }
        }
        (None, Some(s1), Some(p2), None) => {
            if s1.contains_point(&p2) {
                return ResTxT::new(Some(p2), None, None, InfoTxT::IntersectingInAPoint);
            } else {
                return ResTxT::new(None, None, None, InfoTxT::NotIntersecting);
            }
        },
        (Some(p1), None, None, Some(s2)) => {
            if s2.contains_point(&p1) {
                return ResTxT::new(Some(p1), None, None, InfoTxT::IntersectingInAPoint);
            } else {
                return ResTxT::new(None, None, None, InfoTxT::NotIntersecting);
            }
        }
        _ => panic!("Unexpected case!")
    }

}


fn signed_distance(point : &Point, plane : &Plane) -> Number {
    plane.normal.dot_product(&point.get_vector()) + plane.get_ref_d()
}

pub fn intersect_line_and_triangle(line : &Line, tr : &Triangle) -> (Option<Point>, Option<Segment>) {
    let ss : Vec<Segment> = tr.get_sides();
    let segment_of_line = line.gen_segment();

    let mut set : BTreeSet<PointWrapper> = BTreeSet::new();

    for s in ss {
        let res = line_x_segment::intersect(line, &s);
        match res {
            (Some(point), None, line_x_segment::InfoLxS::IntersectingInAPoint) => {
                set.insert(PointWrapper::new(point, &segment_of_line));
            },
            (None, Some(segment), line_x_segment::InfoLxS::IntersectingInASegment) => {
                let Segment {org, dest} = segment;
                set.insert(PointWrapper::new(org, &segment_of_line));
                set.insert(PointWrapper::new(dest, &segment_of_line));
            }
            _ => {}
        }
    }
    if set.len() == 0{
        return (None, None);
    } else if set.len() == 1 {
        let mut v : Vec<Point> = Vec::new();
        for pw in set.into_iter() {
            v.push(pw.extract_point())
        }
        return (Some(v.remove(0)), None);
    } else {
        let mut v : Vec<Point> = Vec::new();
        for pw in set.into_iter() {
            let p = pw.extract_point();
            v.push(p);
        }
        return (None, Some(Segment::new(v.remove(0), v.pop().unwrap())));
    }
}

struct PointDirGraph {
    points : Vec<Point>,
    edges : Vec<BTreeSet<usize>>
}


impl PointDirGraph {
    fn init_sets(ss : &Vec<Segment>) -> Vec<BTreeSet<PointWrapper>> {
        // сортируем точки в порядке отдаления от org

        let mut sets : Vec<BTreeSet<PointWrapper>> = Vec::new();
        for s in ss.iter() {
            let mut set : BTreeSet<PointWrapper> = BTreeSet::new();
            set.insert(PointWrapper::new(s.org.clone(), s));
            set.insert(PointWrapper::new(s.dest.clone(), s));
            sets.push(set);
        }
        return sets;
    }

    fn add_edges_from_sets(&mut self, sets : Vec<BTreeSet<PointWrapper>>) {
        for set in sets {
            let mut previous_pw : Option<PointWrapper> = None;
            for pw in set {

                if previous_pw.is_none() {
                    previous_pw = Some(pw);
                } else {
                    self.add_edge(previous_pw.unwrap().extract_point(), pw.clone().extract_point());
                    previous_pw = Some(pw);
                }
            }
        }
    }

    pub fn print_graph(&self) {
        println!("Points:");
        for (i, point) in self.points.iter().enumerate() {
            println!("{:?}) {:?}", i, point);
        }
        println!("Edges:");
        for (i, v) in self.edges.iter().enumerate() {
            print!("{:?} -> ", self.points[i]);
            for j in v.iter() {
                print!("{:?}; ", self.points[*j]);
            }
            print!("\n");
        }
    }

    pub fn new(tr1 : &Triangle, tr2 : &Triangle) -> PointDirGraph {
        let points : Vec<Point> = Vec::new();
        let edges : Vec<BTreeSet<usize>> = Vec::new();

        let mut pd_graph = PointDirGraph {
            points : points,
            edges : edges
        };

        // если у треугольников разная ориентация, то сохраняется ориентация первого треугольника
        let dp = tr1.get_normal().dot_product(&tr2.get_normal());

        let ss1 : Vec<Segment> = tr1.get_sides();;
        let ss2 : Vec<Segment> = if dp.is_it_positive() {
            tr2.get_sides()
        } else {
            let new_tr2 = Triangle::new(vec![tr2.get(1), tr2.get(0), tr2.get(2)]);
            new_tr2.get_sides()
        };

        let mut wp_sets1 = PointDirGraph::init_sets(&ss1);
        let mut wp_sets2 = PointDirGraph::init_sets(&ss2);

        // добавляем на каждый сегмент точки пересечения
        for (index1, s1) in ss1.iter().enumerate() {
            for (index2, s2) in ss2.iter().enumerate() {
                let res = segment_x_segment::intersect(s1, s2);
                match res {
                    (Some(point), Option::None, segment_x_segment::InfoSxS::IntersectingOnAPoint) => {
                        wp_sets1[index1].insert(PointWrapper::new(point.clone(), s1));
                        wp_sets2[index2].insert(PointWrapper::new(point, s2));
                    },
                    (Option::None, Some(segment), segment_x_segment::InfoSxS::IntersectingInASegment) => {
                        let Segment {org : org, dest : dest} = segment;
                        wp_sets1[index1].insert(PointWrapper::new(org.clone(), s1));
                        wp_sets1[index1].insert(PointWrapper::new(dest.clone(), s1));

                        wp_sets2[index2].insert(PointWrapper::new(org, s2));
                        wp_sets2[index2].insert(PointWrapper::new(dest, s2));
                    }
                    _ => {}
                };
            }
        }

        pd_graph.add_edges_from_sets(wp_sets1);
        pd_graph.add_edges_from_sets(wp_sets2);

        return pd_graph;
    }

    pub fn add_edge(&mut self, org : Point, dest : Point) {
        // все ребра нужно задавать против часовой стрелки, так как граф ориентированный

        let mut org_index : Option<usize> = None;
        let mut dest_index : Option<usize> = None;
        for (i, p) in self.points.iter().enumerate() {
            if *p == org {
                org_index = Some(i);
            } else if *p == dest {
                dest_index = Some(i);
            }
        }
        org_index = match org_index {
            Option::None => {
                self.points.push(org);
                self.edges.push(BTreeSet::new());
                Some(self.points.len() - 1)
            },
            value => value
        };
        dest_index = match dest_index {
            Option::None => {
                self.points.push(dest);
                self.edges.push(BTreeSet::new());
                Some(self.points.len() - 1)
            },
            value => value
        };
        self.edges[org_index.unwrap()].insert(dest_index.unwrap());
    }
}

fn update_map(point_to_verdict : &mut HashMap<Point, bool>, tr1 : &Triangle, tr2 : &Triangle, p : &Point) {
    if !point_to_verdict.contains_key(&p) {
        point_to_verdict.insert(
            p.clone(),
            tr1.does_triangle_contain_point(&p) && tr2.does_triangle_contain_point(&p)
        );
    }
}

pub fn intersect_triangles_in_the_plane(tr1: &Triangle, tr2: &Triangle) -> Polygon {
    if tr1.degradation_level() != 0 || tr2.degradation_level() !=0 {
        panic!("Degradation detected:\n tr1 = {:?}\n tr2 = {:?}", tr1, tr2);
    }

    let mut polygon : Polygon = Polygon::new(Vec::new(), tr1.get_normal());


    let t1_p0_in_t2 = tr2.does_triangle_contain_point(tr1.get_ref(0));
    let t1_p1_in_t2 = tr2.does_triangle_contain_point(tr1.get_ref(1));
    let t1_p2_in_t2 = tr2.does_triangle_contain_point(tr1.get_ref(2));

    let t2_p0_in_t1 = tr1.does_triangle_contain_point(tr2.get_ref(0));
    let t2_p1_in_t1 = tr1.does_triangle_contain_point(tr2.get_ref(1));
    let t2_p2_in_t1 = tr1.does_triangle_contain_point(tr2.get_ref(2));


    if t1_p0_in_t2 && t1_p1_in_t2 && t1_p2_in_t2 {
        polygon.add_point(tr1.get(0));
        polygon.add_point(tr1.get(1));
        polygon.add_point(tr1.get(2));
        return polygon;
    }

    if t2_p0_in_t1 && t2_p1_in_t1 && t2_p2_in_t1 {
        polygon.add_point(tr2.get(0));
        polygon.add_point(tr2.get(1));
        polygon.add_point(tr2.get(2));
        return polygon;
    }

    let mut point_to_verdict: HashMap<Point, bool> = HashMap::new();
    point_to_verdict.insert(tr1.get(0), t1_p0_in_t2);
    point_to_verdict.insert(tr1.get(1), t1_p1_in_t2);
    point_to_verdict.insert(tr1.get(2), t1_p2_in_t2);

    point_to_verdict.insert(tr2.get(0), t2_p0_in_t1);
    point_to_verdict.insert(tr2.get(1), t2_p1_in_t1);
    point_to_verdict.insert(tr2.get(2), t2_p2_in_t1);


    let pd_graph = PointDirGraph::new(tr1, tr2);
    let mut index_of_prev : Option<usize> = None;
    let mut index_of_first : Option<usize> = None;

    //pd_graph.print_graph();

    // на каждом шаге выбираем точку лежащую внутри обоих треугольников
    let mut get_next = || -> Option<Point> {
        if index_of_prev.is_none() {
            for (i, p) in pd_graph.points.iter().enumerate() {
                update_map(&mut point_to_verdict, tr1, tr2, p);
                if point_to_verdict[p] {
                    index_of_first = Some(i);
                    index_of_prev = Some(i);
                    return Some(p.clone());
                } else {
                    continue
                }
            }
            return None;
        } else {
            for index_of_suc in pd_graph.edges[index_of_prev.unwrap()].iter() {
                let p : &Point = pd_graph.points.get(*index_of_suc).unwrap();
                update_map(&mut point_to_verdict, tr1, tr2, p);
                if point_to_verdict[p] {
                    index_of_prev = Some(*index_of_suc);
                    if index_of_first == index_of_prev {
                        // exit condition
                        return None;
                    } else {
                        return Some(p.clone());
                    }
                } else {
                    continue
                }
            }
            return None;
    }
    };

    loop {
        let op : Option<Point> = get_next();
        if op.is_some() {
            polygon.add_point(op.unwrap());
        } else {
            break;
        }
    }

    return polygon;
}


#[cfg(test)]
mod tests {
    use primitives::*;
    use intersect::*;
    use intersect::triangle_x_triangle::PointDirGraph;

    #[test]
    fn points_graph_test1() {
        // треугольники должны лежать в одной плоскости

        let p1 = Point::new_from_f64(0., 0., 0.);
        let p2 = Point::new_from_f64(1., 0., 0.);
        let p3 = Point::new_from_f64(0., 1., 0.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(0., 0., 0.);
        let p2 = Point::new_from_f64(1./2., 0., 0.);
        let p3 = Point::new_from_f64(0., 1./2., 0.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let pd_graph = PointDirGraph::new(&tr1, &tr2);
        //pd_graph.print_graph();

        assert!(pd_graph.points.len() == 5);
        assert!(pd_graph.edges.len() == 5);
        assert!(pd_graph.edges[0].len() == 1);
        assert!(pd_graph.edges[1].len() == 2);
    }

    #[test]
    fn points_graph_test2() {
        // треугольники должны лежать в одной плоскости

        let p1 = Point::new_from_f64(0., 0., 0.);
        let p2 = Point::new_from_f64(1., 0., 0.);
        let p3 = Point::new_from_f64(0., 1., 0.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(0., -1., 0.);
        let p2 = Point::new_from_f64(1., -1., 0.);
        let p3 = Point::new_from_f64(1., 1., 0.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let pd_graph = PointDirGraph::new(&tr1, &tr2);
        //pd_graph.print_graph();

        assert!(pd_graph.points.len() == 8);
        assert!(pd_graph.edges.len() == 8);
        assert!(pd_graph.edges[0].len() == 1);
        assert!(pd_graph.edges[1].len() == 2);
    }

    #[test]
    fn intersect_triangles_in_the_plane() {
        // треугольники должны лежать в одной плоскости

        let p1 = Point::new_from_f64(0., 0., 0.);
        let p2 = Point::new_from_f64(1., 0., 0.);
        let p3 = Point::new_from_f64(0., 1., 0.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(0., -1., 0.);
        let p2 = Point::new_from_f64(1., -1., 0.);
        let p3 = Point::new_from_f64(1., 1., 0.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let polygon : Polygon = triangle_x_triangle::intersect_triangles_in_the_plane(&tr1, &tr2);

        assert!(polygon.points.len() == 3);
    }

    #[test]
    fn intersect_triangles_in_the_plane2() {
        // треугольники должны лежать в одной плоскости

        let p1 = Point::new_from_f64(0., 0., 0.);
        let p2 = Point::new_from_f64(1., 0., 0.);
        let p3 = Point::new_from_f64(0., 1., 0.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(3., 5., 0.);
        let p2 = Point::new_from_f64(2., 6., 0.);
        let p3 = Point::new_from_f64(3., 3., 0.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let polygon : Polygon = triangle_x_triangle::intersect_triangles_in_the_plane(&tr1, &tr2);

        assert!(polygon.points.len() == 0);
    }

    #[test]
    fn intersect_triangles_in_the_plane3() {
        // треугольники должны лежать в одной плоскости

        let p1 = Point::new_from_f64(0., 0., 0.);
        let p2 = Point::new_from_f64(1., 0., 0.);
        let p3 = Point::new_from_f64(0., 1., 0.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(3., 5., 0.);
        let p2 = Point::new_from_f64(2., 6., 0.);
        let p3 = Point::new_from_f64(1./2., 1./2., 0.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let polygon : Polygon = triangle_x_triangle::intersect_triangles_in_the_plane(&tr1, &tr2);

        assert!(polygon.points.len() == 1);
    }

    fn intersect_triangles_in_the_plane4() {
        // треугольники должны лежать в одной плоскости

        let p1 = Point::new_from_f64(0., 0., 0.);
        let p2 = Point::new_from_f64(1., 0., 0.);
        let p3 = Point::new_from_f64(0., 1., 0.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(0., 1., 0.);
        let p2 = Point::new_from_f64(1., 0., 0.);
        let p3 = Point::new_from_f64(3., 3., 0.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let polygon : Polygon = triangle_x_triangle::intersect_triangles_in_the_plane(&tr1, &tr2);

        assert!(polygon.points.len() == 2);
    }

    fn intersect_triangles_in_the_plane5() {
        // треугольники должны лежать в одной плоскости

        let p1 = Point::new_from_f64(0., 0., 0.);
        let p2 = Point::new_from_f64(1., 0., 0.);
        let p3 = Point::new_from_f64(0., 1., 0.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(0., 0., 0.);
        let p2 = Point::new_from_f64(2., 0., 0.);
        let p3 = Point::new_from_f64(0., 2., 0.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let polygon : Polygon = triangle_x_triangle::intersect_triangles_in_the_plane(&tr1, &tr2);

        assert!(polygon.points.len() == 3);
    }


    #[test]
    fn triangles_in_the_plane() {
        let p1 = Point::new_from_f64(1., 0., 0.);
        let p2 = Point::new_from_f64(0., 1., 0.);
        let p3 = Point::new_from_f64(1., 1., 0.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(1., -10., 0.);
        let p2 = Point::new_from_f64(0., -10., 0.);
        let p3 = Point::new_from_f64(1., -9., 0.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let res = triangle_x_triangle::intersect(&tr1, &tr2);

        if let (Option::None, Option::None, Option::None, triangle_x_triangle::InfoTxT::CoplanarNotIntersecting) = res.clone().get_tuple()  {
            return;
        } else {
            panic!("Wrong info: {:?}", res.get_info());
        };
    }

    #[test]
    fn collinear_triangles() {
        let p1 = Point::new_from_f64(1., 0., 0.);
        let p2 = Point::new_from_f64(0., 1., 0.);
        let p3 = Point::new_from_f64(1., 1., 0.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(1., 0., 10.);
        let p2 = Point::new_from_f64(0., 1., 10.);
        let p3 = Point::new_from_f64(1., 1., 10.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let res = triangle_x_triangle::intersect(&tr1, &tr2);

        if let (Option::None, Option::None, Option::None, triangle_x_triangle::InfoTxT::Collinear) = res.clone().get_tuple()  {
            return;
        } else {
            panic!("Wrong info: {:?}", res.get_info());
        };
    }

    #[test]
    fn intersect_triangles_1p_on_the_line() {
        let p1 = Point::new_from_f64(-1., 0., 0.);
        let p2 = Point::new_from_f64(1., 0., 0.);
        let p3 = Point::new_from_f64(0., 0., 1.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(0., -2., 0.);
        let p2 = Point::new_from_f64(-2., 0., 0.);
        let p3 = Point::new_from_f64(0., 1., 0.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let ep1 = Point::new_from_f64(-1., 0., 0.);
        let ep2 = Point::new_from_f64(0., 0., 0.);

        let es = Segment {org: ep1, dest: ep2};

        let res = triangle_x_triangle::intersect(&tr1, &tr2);

        if let (Option::None, Some(s), Option::None, triangle_x_triangle::InfoTxT::Intersecting) = res.clone().get_tuple()  {
            if s != es {
                panic!("Wrong result: {}", s);
            }
        } else {
            panic!("Wrong info: {:?}", res.get_info());
        };
    }

    #[test]
    fn intersect_triangles_2p_on_the_line() {
        let p1 = Point::new_from_f64(-2., 0., 0.);
        let p2 = Point::new_from_f64(0., 2., 0.);
        let p3 = Point::new_from_f64(2., 0., 0.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(-1., 1., 0.);
        let p2 = Point::new_from_f64(0., 1., 1.);
        let p3 = Point::new_from_f64(1., 1., 0.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let ep1 = Point::new_from_f64(-1., 1., 0.);
        let ep2 = Point::new_from_f64(1., 1., 0.);

        let es = Segment {org: ep1, dest: ep2};

        let res = triangle_x_triangle::intersect(&tr1, &tr2);

        if let (Option::None, Some(s), Option::None, triangle_x_triangle::InfoTxT::Intersecting) = res.clone().get_tuple()  {
            if s != es {
                panic!("Wrong result: {}", s);
            }
        } else {
            panic!("Wrong info: {:?}", res.get_info());
        };
    }

    #[test]
    fn intersect_triangles_bug_test1() {
        let p1 = Point::new_from_f64(-4., -4., 0.);
        let p2 = Point::new_from_f64(4., -4., 0.);
        let p3 = Point::new_from_f64(-4., 4., 0.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(0., 2., 2.);
        let p2 = Point::new_from_f64(-4., 2., 2.);
        let p3 = Point::new_from_f64(0., 2., -2.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let ep = Point::new_from_f64(-2., 2., 0.);

        let res = triangle_x_triangle::intersect(&tr1, &tr2);

        if let (Some(p), None, Option::None, triangle_x_triangle::InfoTxT::IntersectingInAPoint) = res.clone().get_tuple()  {
            if p != ep {
                panic!("Wrong result: {}", p);
            }
        } else {
            panic!("Wrong info: {:?}", res.get_info());
        };
    }

    #[test]
    fn intersect_triangles_bug_test2() {
        let p1 = Point::new_from_f64(-4., 4., 4.);
        let p2 = Point::new_from_f64(4., 4., -4.);
        let p3 = Point::new_from_f64(4., 4., 4.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(-2., 7., 2.);
        let p2 = Point::new_from_f64(2., 7., 2.);
        let p3 = Point::new_from_f64(2., 1., 2.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        let ep = Point::new_from_f64(-2., 2., 0.);

        let res : triangle_x_triangle::ResTxT = triangle_x_triangle::intersect(&tr2, &tr1);


        if let (None, Some(s), None, triangle_x_triangle::InfoTxT::Intersecting) = res.clone().get_tuple()  {

        } else {
            panic!("Wrong info: {:?}", res.get_info());
        };
    }

}

