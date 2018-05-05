use intersect::point_wrapper::PointWrapper;
use std::collections::{BTreeSet, HashMap};
use primitives::*;
use primitives::point::EPointPosition;
// use triangulation::triangulation3d::check_segments;

#[derive(Clone)]
#[derive(Debug, Hash)]
pub struct Polygon {
    points: Vec<Point>,
    normal: Vector
}

impl Polygon {
    pub fn new(points : Vec<Point>, normal : Vector) -> Polygon {
        Polygon {
            points,
            normal
        }
    }

    pub fn get_points(self) -> Vec<Point> {
        return self.points;
    }

    pub(crate) fn get_segments(&self) -> Vec<Segment> {
        let mut res: Vec<Segment> = Vec::new();
        for cur_i in 0..self.points.len() {
            let next_i = (cur_i + 1)%self.points.len();
            let s = Segment::new(self.points[cur_i].clone(), self.points[next_i].clone());
            res.push(s);
        }
        return res;
    }

    pub fn get_points_ref(&self) -> &Vec<Point> {
        return &self.points;
    }

    pub fn get_normal_ref(&self) -> &Vector {
        return &self.normal;
    }

    pub fn set_normal(&mut self, normal: Vector) {
        self.normal = normal;
    }

    pub fn add_point(&mut self, p : Point) {
        self.points.push(p);
    }

    pub fn get_points_and_normal(self) -> (Vec<Point>, Vector) {
        return (self.points, self.normal);
    }

    // Для двух непересекающихся полигонов проверяется: находится ли other внутри self.
    pub fn contains(&self, other: &Polygon) -> bool {
        // достаточно проверить лишь одну точку

        let one = Number::new(1.);
        let zero = Number::new(0.);

        let pm = other.points[0].clone();


        let mut num_of_intersections = 0;

        let n = self.points.len();
        for cur_index in 0..n {
            let next_index : usize = (cur_index+1)%n;
            let (cur_point, next_point) = (&self.points[cur_index], &self.points[next_index]);

            // ищем пересечение ребра s c положительным Ox лучем начинающимся в pm
            let dir = next_point - cur_point;

            if dir.y == zero {
                // ребро параллельно прямой, пересечений нет.
                continue;
            }

            let t = (&pm.y - &cur_point.y) / &dir.y;

            if t >= zero && t <= one {
                let pi = cur_point + &(&dir*t.clone());
                if pi.x < pm.x {
                    continue;
                }

                num_of_intersections += 1;
            }
        }

        return (num_of_intersections % 2) == 1;

    }

    pub fn signed_area(&self) -> Number {
        let n = self.points.len();
        let mut signed_area = Number::new(0.);

        for cur_index in 0..n {
            let next_index = (cur_index+1)%n;
            let (cur_point, next_point) = (&self.points[cur_index], &self.points[next_index]);
            signed_area = signed_area + &cur_point.x*&next_point.y - &next_point.x*&cur_point.y;
        }

        return signed_area / Number::new(2.);
    }

    pub fn reverse_order(&mut self) {
        self.points.reverse();
    }
}


#[derive(Clone)]
#[derive(Debug)]
pub struct PolygonTreeNode {
    children: Vec<PolygonTreeNode>,
    polygon: Polygon,
    index: Vec<usize>
}

impl PolygonTreeNode {
    pub fn new(p : Polygon) -> PolygonTreeNode {
        PolygonTreeNode {
            children: Vec::new(),
            polygon: p,
            index: Vec::new()
        }
    }

    pub fn add_child(&mut self, new_child: PolygonTreeNode) {
        self.children.push(new_child);
    }

    pub fn add_children(&mut self, new_children: Vec<PolygonTreeNode>) {
        self.children.extend(new_children);
    }

    pub fn insert_polygon(&mut self, p : &Polygon) -> bool {
        if self.polygon.contains(&p) {
            let mut inserted = false;
            for child_tree in self.children.iter_mut() {
                inserted = inserted || child_tree.insert_polygon(p);
                if inserted {
                    break;
                }
            }

            if !inserted {
                self.add_child(PolygonTreeNode::new(p.clone()));
            }
            return true;
        } else {
            return false;
        }
    }

    #[allow(dead_code)]
    pub fn set_polygon(&mut self, polygon: Polygon) {
        self.polygon = polygon;
    }

    #[allow(dead_code)]
    pub fn get_children_ref(&self) -> &Vec<PolygonTreeNode> {
        return &self.children;
    }

    pub fn get_cur_polygon(&self) -> Polygon {
        return self.polygon.clone();
    }

    pub fn get_child_polygons(&self) -> Vec<Polygon> {
        let mut vec = Vec::new();
        vec.extend(self.children.iter().map(|ref node|-> Polygon {node.polygon.clone()}));
        return vec;
    }

    pub fn split_tree(self) -> Vec<PolygonTreeNode> {
        return self.children;
    }

    #[allow(dead_code)]
    pub fn map(&mut self, f: &Fn(&mut Point)) {
        for p in self.polygon.points.iter_mut() {
            f(p);
        }

        for subtree in self.children.iter_mut() {
            subtree.map(f);
        }
    }

    // строим дерево вложенных полигонов для передачи в триангуляцию
    pub fn new_trees(t : Triangle, segments : Vec<Segment>) -> Vec<PolygonTreeNode> {
        let mut tc = TriangleContainer::new(t, segments);
        let mut boundary_polygons : Vec<Polygon> = Vec::new();

        loop {
            // bs -> boundary segment
            let o_index_of_bs = tc.pop_boundary();
            if o_index_of_bs.is_none() {
                break;
            }
            let index_of_bs = o_index_of_bs.unwrap();
            let o_polygon = tc.get_polygon(index_of_bs, true);
            if o_polygon.is_some() {
                boundary_polygons.push(o_polygon.unwrap());
            }
        }

        let mut internal_polygons : Vec<Polygon> = Vec::new();

        loop {
            // is -> internal segment
            let o_index_of_is = tc.pop_internal();
            if o_index_of_is.is_none() {
                break;
            }
            let index_of_is = o_index_of_is.unwrap();
            let o_polygon = tc.get_polygon(index_of_is, false);
            if o_polygon.is_some() {
                internal_polygons.push(o_polygon.unwrap());
            }
        }

        let mut vec_of_trees: Vec<PolygonTreeNode> = Vec::new();
        for bp in boundary_polygons {
            vec_of_trees.push(PolygonTreeNode::new(bp));
        }


        for ip in internal_polygons {
            for tree in vec_of_trees.iter_mut() {
                if tree.insert_polygon(&ip) {
                    break;
                }
            }
        }

        return vec_of_trees;
    }

}

struct TriangleContainer {
    segments: Vec<Segment>,
    boundary: Vec<usize>,
    internal: Vec<usize>,
    normal: Vector,
    point_to_ns: HashMap<Point, Vec<usize>>
}

impl TriangleContainer {

    /*
    Ищем точки на сторонах треугольника и формируем дополнительные граничные сегменты.
    Формируем промежуточное представление.
    */
    pub fn new(t: Triangle, ss: Vec<Segment>) -> TriangleContainer {
        let mut tc = TriangleContainer {
            segments: Vec::new(),
            boundary: Vec::new(),
            internal: Vec::new(),
            normal: t.get_normal(),
            point_to_ns: HashMap::new()
        };


        let mut ab_set : BTreeSet<PointWrapper> =  BTreeSet::new();
        let ab : Segment = Segment::new(t.get(0), t.get(1));
        ab_set.insert(PointWrapper::new(t.get(0), &ab));
        ab_set.insert(PointWrapper::new(t.get(1), &ab));

        let mut bc_set : BTreeSet<PointWrapper> =  BTreeSet::new();
        let bc : Segment = Segment::new(t.get(1), t.get(2));
        bc_set.insert(PointWrapper::new(t.get(1), &bc));
        bc_set.insert(PointWrapper::new(t.get(2), &bc));

        let mut ca_set : BTreeSet<PointWrapper> = BTreeSet::new();
        let ca : Segment = Segment::new(t.get(2), t.get(0));
        ca_set.insert(PointWrapper::new(t.get(2), &ca));
        ca_set.insert(PointWrapper::new(t.get(0), &ca));

        for s in ss.iter() {
            TriangleContainer::update_set(s, &ab, &mut ab_set);
            TriangleContainer::update_set(s, &bc, &mut bc_set);
            TriangleContainer::update_set(s, &ca, &mut ca_set);
        }

        /*
        Лемма 1:
        Отрезки-сечения не могут иметь более одной общей точки.
        В противном случае присутствует самопересечение поверхности по общему фрагменту.

        Лемма 2:
        Если отрезки-сечения имеют общую точку, то это их граничная точка.
        В противном случае имеется самопересечение.
        */

        tc.add_all(TriangleContainer::extract_new_ss(&ab_set), true);
        tc.add_all(TriangleContainer::extract_new_ss(&bc_set), true);
        tc.add_all(TriangleContainer::extract_new_ss(&ca_set), true);

        // let mut plane: Plane = Plane::new(t.get_normal(), t.get(0));
        // println!("plane = {:?}", plane);
        // check_segments(&plane, &ss);


        tc.add_all(ss, false);

        // check_segments(&plane, &tc.segments);

        /*
        println!("Segments!");
        for (i, s) in tc.segments.iter().enumerate() {
            println!("{:?}) {:?}", i, s);
        }
        println!("boundary.len() = {0}, internal.len() = {1}", tc.boundary.len(), tc.internal.len());
        */

        let mut map: HashMap<Point, Vec<usize>> = HashMap::new();
        fn add_to_hash_map(p: &Point, index: usize, map: &mut HashMap<Point, Vec<usize>>) {
            if map.contains_key(p) {
                map.get_mut(p).unwrap().push(index);
            } else {
                map.insert(p.clone(), vec![index]);
            }
        }

        for (index, s) in tc.segments.iter().enumerate() {
            add_to_hash_map(&s.org, index.clone(), &mut map);
            add_to_hash_map(&s.dest, index.clone(), &mut map);
        }

        tc.point_to_ns.extend(map);

        return tc;
    }

    fn update_set(s : &Segment, side : &Segment, set : &mut BTreeSet<PointWrapper>) {
        if s.org.classify(&side.org, &side.dest) == EPointPosition::Between {
            set.insert(PointWrapper::new(s.org.clone(), side));
        }

        if s.dest.classify(&side.org, &side.dest) == EPointPosition::Between {
            set.insert(PointWrapper::new(s.dest.clone(), side));
        }
    }

    pub fn pop_boundary(&mut self) -> Option<usize> {
        self.boundary.pop()
    }

    pub fn pop_internal(&mut self) -> Option<usize> {
        self.internal.pop()
    }

    // возвращает полигон, который можно построить начиная с отрезка, заданного индексом.
    pub fn get_polygon(&mut self, index: usize, boundary: bool) -> Option<Polygon> {

        /*
        if boundary {
            println!("Getting boundary polygon, starting at {0}!", index);
        } else {
            println!("Getting internal polygon, starting at {0}!", index);
        }
        */

        let mut cur_s = self.segments[index].clone();
        let mut points : Vec<Point> = Vec::new();

        loop  {
            points.push(cur_s.org.clone());

            let neighbours: Vec<usize> = self.point_to_ns.get(&cur_s.dest).unwrap().clone();
            if neighbours.len() < 2 {
                return None;
            }

            let index_of_next: usize = self.find_best(&cur_s, &neighbours);
            self.mark_as_performed(index_of_next);

            if self.segments[index_of_next].org != cur_s.dest {
                // println!("flipped {0}", cur_s);
                self.segments[index_of_next].flip();
            }
            cur_s = self.segments[index_of_next].clone();

            if index_of_next == index {
                break;
            }
        }

        // println!("polygon: {:?}", points);
        let mut p = Polygon::new(points, self.normal.clone());
        if !boundary && p.signed_area().is_it_negative() {
            p.reverse_order();
        }

        return Some(p);
    }

    fn add(&mut self, new_s: Segment, is_it_boundary: bool) {
        for s in self.segments.iter() {
            if s == &new_s {
                return;
            }
        }

        self.segments.push(new_s);
        if is_it_boundary {
            self.boundary.push(self.segments.len() - 1);
        } else {
            self.internal.push(self.segments.len() - 1);
        }
    }

    fn add_all(&mut self, v: Vec<Segment>, is_it_boundary: bool) {
        for s in v {
            self.add(s, is_it_boundary);
        }
    }

    fn extract_new_ss(set : &BTreeSet<PointWrapper>) -> Vec<Segment> {
        let mut vec: Vec<&PointWrapper> = Vec::new();
        vec.extend(set);
        let mut vec_of_ss : Vec<Segment> = Vec::new();

        for i in 0..vec.len()-1 {
            vec_of_ss.push(
                Segment::new(
                    vec[i].clone().extract_point(),
                    vec[i+1].clone().extract_point()
                )
            );
        }

        return vec_of_ss;
    }


    // каждый раз выбираем набиолее "левую" точку.
    fn is_first_better(e1: &EPointPosition, sc1: &Number, e2: &EPointPosition, sc2: &Number) -> bool {

        if e2 == &EPointPosition::Right && e1 == &EPointPosition::Left {
            return true;
        }

        if e2 == &EPointPosition::Left && e1 == &EPointPosition::Right {
            return false;
        }

        if e2 == &EPointPosition::Beyond && e1 == &EPointPosition::Right {
            return false;
        }

        if e2 == &EPointPosition::Beyond && e1 == &EPointPosition::Left {
            return true;
        }

        if e2 == &EPointPosition::Right && e1 == &EPointPosition::Beyond {
            return true;
        }

        if e2 == &EPointPosition::Left && e1 == &EPointPosition::Beyond {
            return false;
        }

        //assert_ne!(sc1, sc2);

        if e2 == &EPointPosition::Left && e1 == &EPointPosition::Left {
            return sc1 < sc2;
        }

        if e2 == &EPointPosition::Right && e1 == &EPointPosition::Right {
            return sc1 > sc2;
        }
        panic!("ERROR: e1 = {:?}, e2 = {:?}", e1, e2);
    }

    fn find_best(&self, cur_segment: &Segment, ns: &Vec<usize>) -> usize {
        /*
        println!();
        println!("cur_segment = {0}", cur_segment);
        println!("ns = {:?}", ns);
        */

        if ns.len() < 2 {
            panic!("Something goes wrong!");
        }


        let mut best_index = 0;
        let mut e = EPointPosition::Right;
        let mut sc = Number::new(-2.);

        for i in 0..ns.len() {
            let next_segment = self.segments[ns[i]].clone();
            if &next_segment == cur_segment {
                continue;
            }

            let end_point = if  cur_segment.dest == next_segment.org {
                next_segment.dest.clone()
            } else {
                next_segment.org.clone()
            };

            let cur_e: EPointPosition = end_point.classify(&cur_segment.org, &cur_segment.dest);
            let cur_vec : Vector = &cur_segment.dest - &cur_segment.org;
            let next_vec : Vector = if  cur_segment.dest == next_segment.org {
                &next_segment.dest - &next_segment.org
            } else {
                &next_segment.org - &next_segment.dest
            };

            /*
            println!("cur_s = {:?}", cur_segment);
            println!("saved_s = {:?}", self.segments.get(best_index));
            println!("next_s = {:?}", next_segment);
            println!("end_point = {:?}", end_point);
            */

            let cur_sc = cur_vec.get_signed_cos2(&next_vec);

            if TriangleContainer::is_first_better(&cur_e, &cur_sc, &e, &sc) &&
                self.point_to_ns.get(&end_point).unwrap().len() >= 2
            {
                e = cur_e;
                sc = cur_sc;
                best_index = ns[i];
            }
        }

        assert_ne!(sc, Number::new(-2.));

        //println!("best index = {0}", best_index);
        return best_index;
    }

    fn mark_as_performed(&mut self, index: usize) {
        for i in 0..self.boundary.len() {
            if self.boundary[i] == index {
                self.boundary.remove(i);
                break;
            }
        }

        for i in 0..self.internal.len() {
            if self.internal[i] == index {
                self.internal.remove(i);
                break;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use primitives::*;
    use triangulation::*;
    // use env_logger::init  as env_logger_init;

    #[test]
    fn loop_builder_test1() {
        let p1 = Point::new_from_f64(10.0, 10.0, 0.0);
        let p2 = Point::new_from_f64(0.0, 10.0, 0.0);
        let p3 = Point::new_from_f64(10.0, 0.0, 0.0);

        let p4 = Point::new_from_f64(8.0, 6.0, 0.0);
        let p5 = Point::new_from_f64(8.0, 8.0, 0.0);
        let p6 = Point::new_from_f64(10.0, 6.0, 0.0);
        let p7 = Point::new_from_f64(10.0, 4.0, 0.0);

        let p8 = Point::new_from_f64(6.0, 6.0, 0.0);
        let p9 = Point::new_from_f64(6.0, 8.0, 0.0);
        let p10 = Point::new_from_f64(6.0, 9.0, 0.0);
        let p11 = Point::new_from_f64(7.0, 8.0, 0.0);

        let p12 = Point::new_from_f64(4.0, 10.0, 0.0);
        let p13 = Point::new_from_f64(4.0, 8.0, 0.0);


        let t = Triangle::new(vec![p1.clone(), p2.clone(), p3.clone()]);
        let ss : Vec<Segment> = vec![
            Segment::new(p4.clone(), p5.clone()),
            Segment::new(p6.clone(), p5.clone()),
            Segment::new(p6.clone(), p7.clone()),
            Segment::new(p4.clone(), p7.clone()),
            Segment::new(p2.clone(), p8.clone()),
            Segment::new(p8.clone(), p3.clone()),
            Segment::new(p9.clone(), p10.clone()),
            Segment::new(p11.clone(), p10.clone()),
            Segment::new(p9.clone(), p11.clone()),
            Segment::new(p12.clone(), p13.clone()),
        ];


        let ts : Vec<Triangle> = triangulate_ptree3d(t, ss);
        assert_eq!(ts.len(), 16);

        let mut mesh = Mesh::new();
        mesh.add_triangles(ts);

        let mut f = File::create("res_of_tests/ear_cl_tr/loop_builder_test1.stl").unwrap();


        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };

    }
}