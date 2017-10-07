use primitives::*;
use intersect::mesh_x_mesh;
use intersect::triangle_x_triangle::InfoTxT;
use triangulation::incremental_triangulation::triangulate;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::collections::hash_set;
use std::vec;

use log::LogLevel;
use time::PreciseTime;
use std::result::Result;

#[derive(Clone)]
#[derive(PartialEq, Eq)]
#[derive(Debug)]
enum Marker {
    Unclassified,
    Inner,
    Outer,
    PlanarPlus,
    PlanarMinus
}

impl Marker {
    pub fn is_it_planar(&self) -> bool {
        return (*self == Marker::PlanarMinus) || (*self == Marker::PlanarPlus);
    }
}

/// This structure keeps intermediate information, necessary for performing of boolean operations and have methods to perform such operations.
/// It's still in a process of being created, so it is unreliable.
/// It's recommended to perform boolean operations only for simple objects.
pub struct BoolOpPerformer {
    mesh_a : Mesh,
    mesh_b : Mesh,
    a_it_to_marker : HashMap<usize, Marker>,
    b_it_to_marker : HashMap<usize, Marker>,
}

fn find_conjugated_point(tr: &Triangle, org: &Point, dest: &Point) -> Option<Point> {
    for i in 0..3 {
        let p_cur = tr.get_ref(i);
        let p_next = tr.get_ref((i+1)%3);
        if (p_cur == org) && (p_next == dest) || (p_cur == dest) && (p_next == org) {
            return Some(tr.get_ref((i+2)%3).clone())
        }
    }
    return None;
}

fn choose_correct_dp(ns: &Vec<Vector>, v: &Vector) -> Number {
    /*
    Утверждение:
    Если наблюдается случай касания ребром одной поверхности другой, то
    тогда для этого ребра возникнут две разные нормали. При этом они могут давать
    разный результат при классификации. В этом случае нужно брать нормаль, которая имеет больший угол
    с вектором, проведенным  из середины этого ребра в третью точку треугольника (v).
    */


    let mut odp: Option<Number> = None;
    let mut on: Option<Vector> = None;

    let len_ns = ns.len();

    let mut o_cos2 : Option<Number> = None;
    let len2_v = v.length2();

    for n in ns.iter() {
        let cur_dp = v.dot_product(n);

        if cur_dp.is_it_zero() {
            // игнорируем случаи наложения.
            continue
        }

        let cur_cos2 = &cur_dp / (&len2_v * n.length2());

        /*
        if len_ns >= 2 {
            println!("normal {:?}", n);
            println!("cur_dp {0}", cur_dp);
            println!("cur_cos2 {0}", cur_cos2);
            println!();
        }
        */

        if o_cos2.is_none() || (o_cos2.clone().unwrap() < cur_cos2) {
            odp = Some(cur_dp);
            o_cos2 = Some(cur_cos2);
        }
    }

    if odp.is_some() {
        return odp.unwrap();
    } else {
        return Number::zero();
    }

}

fn first_classification(t: &Triangle, tdesc: &mut TDesc) -> Marker {
    debug!("First classification for {:?} was started", t);
    for (s, ref ns) in tdesc.get_s_to_ns_ref().iter() {
        debug!("s: {:?}", s);

        let (org, dest) = s.clone().get_org_dest();
        let ocp = find_conjugated_point(t, &org, &dest);
        match ocp {
            Some(p) => {
                let p_proj = s.get_point_projection(&p);
                debug!("\tp_proj {:?}", p_proj);
                debug!("\tp {:?}", p);
                let v = p - p_proj;

                //println!("\ns: {:?}", s);
                let dot_vn = choose_correct_dp(ns, &v);
                if dot_vn.is_it_negative() {
                    debug!("\tIt's an inner triangle dot_vn={0}\n", dot_vn);
                    return  Marker::Inner;
                } else if dot_vn.is_it_positive() {
                    debug!("\tIt's an outer triangle dot_vn={0}\n", dot_vn);
                    return Marker::Outer;
                }
            },
            _ => {}
        }
    }
    debug!("It's an unclassified triangle\n");
    return Marker::Unclassified;
}


fn second_classification(t: &Triangle, tdesc: &mut TDesc) -> Marker {
    for polygon in tdesc.get_polygons_ref() {
        let pn = polygon.normal.clone();
        let ps = polygon.get_points_ref();
        let ps_len = ps.len();
        let tn = t.get_normal();

        let mut num_of_common_ps = 0;
        for i in 0..ps.len() {
            for j in 0..3 {
                if &ps[i] == t.get_ref(j) {
                    num_of_common_ps += 1;
                }
            }
        }

        // полигоны выпуклые, и как следствие все их внутренние треугольники содержат три точки из полигона
        if num_of_common_ps != 3 {
            continue;
        }

        for i in 0..ps.len() {
            let ref p_cur = ps[i];
            let ref p_next = ps[(i+1)%ps_len];
            let ocp = find_conjugated_point(t, p_cur, p_next);
            match ocp {
                Some(p) => {
                    let s = Segment::new(p_cur.clone(), p_next.clone());
                    let p_proj = s.get_point_projection(&p);
                    let v = p - p_proj;
                    let dot_vn = v.dot_product(&pn);
                    if dot_vn.is_it_zero() {
                        let dot_pn_x_tn = tn.dot_product(&pn);
                        if dot_pn_x_tn.is_it_positive() {
                            return Marker::PlanarPlus;
                        } else {
                            return Marker::PlanarMinus;
                        }
                    } else {
                        return  Marker::Unclassified;
                    }
                },
                _ => {}
            }
        }
    }
    return Marker::Unclassified;
}

fn triangulate_all(it_to_desc: HashMap<usize, TDesc>) -> HashMap<Triangle, Marker> {
    let mut new_ts : HashMap<Triangle, Marker>  = HashMap::new();
    for (it, mut tdesc) in it_to_desc.into_iter() {
        let mut points : Vec<Point> = Vec::new();
        for p in tdesc.get_points_drain_iter() {
            points.push(p);
        }

        let ts = triangulate(points, tdesc.plane.clone());

        for mut t in ts {
            let m1 = first_classification(&t, &mut tdesc);
            let m2 = second_classification(&t, &mut tdesc);
            let res_marker = if m2.is_it_planar() {
                // TODO исправить баг с параллельным продолжением
                // TODO исправить баг касание ребром и отсутствие некоторых двойных нормалей
                m2
            } else {
                m1
            };
            new_ts.insert(t, res_marker);
        }
    }
    return new_ts;
}

// Decorator for "old" triangle
struct TDesc {
    it : usize,
    points : HashSet<Point>,
    polygons : Vec<Polygon>,
    s_to_ns: BTreeMap<Segment, Vec<Vector>>,
    plane: Plane
}

impl TDesc {
    pub fn new(it: usize, plane: Plane) -> TDesc {
        TDesc {
            it: it,
            points: HashSet::new(),
            polygons: Vec::new(),
            s_to_ns: BTreeMap::new(),
            plane: plane
        }
    }

    pub fn len_of_points(&self) -> usize {
        return self.points.len();
    }

    pub fn add_point(&mut self, p: Point) {
        self.points.insert(p);
    }

    pub fn add_polygon(&mut self, polygon: Polygon) {
        self.polygons.push(polygon.clone());

        let points = polygon.get_points();
        for p in points {
            self.points.insert(p);
        }
    }


    /*
    Так как модели правильные, т.е. без самопересечения.
    То один треугольник по одному отрезку секут ровно два треугольника.
    */
    pub fn add_s_and_n(&mut self, s: Segment, n: Vector) {
        if self.s_to_ns.contains_key(&s) {
            self.s_to_ns.get_mut(&s).unwrap().push(n);
        } else {
            self.s_to_ns.insert(s.clone(), vec![n]);
            let (org, dest) = s.get_org_dest();
            self.points.insert(org);
            self.points.insert(dest);
        }
    }

    pub fn get_points_drain_iter(&mut self) -> hash_set::Drain<Point> {
        return self.points.drain();
    }

    pub fn get_points_ref(&self) -> &HashSet<Point> {
        return &self.points;
    }



    pub fn get_s_to_ns_ref(&self) -> &BTreeMap<Segment, Vec<Vector>> {
        return &self.s_to_ns;
    }

    pub fn get_polygons_drain_iter(&mut self) -> vec::Drain<Polygon> {
        let len = self.polygons.len();
        return self.polygons.drain(0..len);
    }

    pub fn get_polygons_ref(&self) -> &Vec<Polygon> {
        return &self.polygons;
    }

}

fn add_triangles(ts_and_ms: HashMap<Triangle, Marker>, mesh: &mut Mesh, it_to_marker: &mut HashMap<usize, Marker>) {
    for (t, m) in ts_and_ms.into_iter() {
        let it = mesh.add_triangle(t);
        it_to_marker.insert(it, m);
    }
}

fn dfs(mesh: &mut Mesh, it_to_marker: &mut HashMap<usize, Marker>) {
    let mut ts_to_visit: Vec<(usize, Marker)> = Vec::new();
    for (it, m) in it_to_marker.iter() {
        if m != &Marker::Unclassified {
            ts_to_visit.push((it.clone(), m.clone()));
        }
    }

    while ts_to_visit.len() != 0 {
        let (it, m) = ts_to_visit.pop().unwrap();

        let conjugated_ts = mesh.find_segment_conjugated_triangles(it);
        for ict in conjugated_ts {
            if !it_to_marker.contains_key(&ict) || (it_to_marker[&ict] == Marker::Unclassified) {
                it_to_marker.insert(ict, m.clone());
                ts_to_visit.push((ict, m.clone()));
            }
        }
    }

}


impl BoolOpPerformer {

    /// This method prepares intermediate structures for performing of boolean operations and saves it in the instance of `BoolOpPerformer` structure.
    /// If meshes don't intersect each other the `Err` will be returned.
    /// # Arguments
    ///
    /// * `mesh_a_ref` - A reference to the first mesh.
    /// * `mesh_a_ref` - A reference to the second mesh.
    pub fn new(mesh_a_ref: &Mesh, mesh_b_ref: &Mesh) -> Result<BoolOpPerformer, &'static str> {
        let start = PreciseTime::now();

        if log_enabled!(LogLevel::Info) {
            info!("----------------------------------------");
            info!("<BoolOpPerformer::new> is performing ...\n");
        }

        let mut mesh_a : Mesh = mesh_a_ref.clone();
        let mut mesh_b : Mesh = mesh_b_ref.clone();

        let m_x_m_start = PreciseTime::now();
        info!("Intersection of meshes is performing ...");
        let mxm_res = mesh_x_mesh::intersect(&mesh_a, &mesh_b);
        info!("<mesh_x_mesh::intersect> is finished in {0} seconds.", m_x_m_start.to(PreciseTime::now()));

        let mut a_it_to_tdec : HashMap<usize, TDesc> = HashMap::new();
        let mut b_it_to_tdec : HashMap<usize, TDesc> = HashMap::new();

        let mxm_res_lst = mxm_res.get_res_list();
        info!("There are {0} pairs of intersecting triangles.\n", mxm_res_lst.len());
        if mxm_res_lst.len() == 0 {
            return Err("Meshes don't intersect each other!");
        }


        let td_start = PreciseTime::now();
        info!("The triangle descriptors are being created ...");
        for (index_a, index_b, res) in  mxm_res_lst{
            debug!("\t({0}, {1}) is performing", index_a, index_b);
            mesh_a.remove_triangle(&index_a);
            mesh_b.remove_triangle(&index_b);

            if !a_it_to_tdec.contains_key(&index_a) {
                a_it_to_tdec.insert(index_a, TDesc::new(index_a, mesh_a_ref.get_plane_by_index(index_a)));
            }

            if !b_it_to_tdec.contains_key(&index_b) {
                b_it_to_tdec.insert(index_b, TDesc::new(index_b, mesh_b_ref.get_plane_by_index(index_b)));
            }

            let mut_ref_tdec_a = a_it_to_tdec.get_mut(&index_a).unwrap();
            let mut_ref_tdec_b = b_it_to_tdec.get_mut(&index_b).unwrap();

            // первичное добавление точек треугольника
            if mut_ref_tdec_a.len_of_points() == 0 {
                for p in mesh_a_ref.get_triangle(index_a).get_points_ref().iter() {
                    mut_ref_tdec_a.add_point(p.clone());
                }
            }

            if mut_ref_tdec_b.len_of_points() == 0 {
                for p in mesh_b_ref.get_triangle(index_b).get_points_ref().iter() {
                    mut_ref_tdec_b.add_point(p.clone());
                }
            }

            let na = mesh_a_ref.get_normal_by_index(index_a);
            let nb = mesh_b_ref.get_normal_by_index(index_b);

            // вторичное добавление точек фрагментов пересечения
            match res.get_info() {

                /*
                InfoTxT::IntersectingInAPoint => {
                    let point = res.get_point();
                    mut_ref_tdec_a.add_point(point.clone());
                    mut_ref_tdec_b.add_point(point);
                },
                */

                InfoTxT::Intersecting => {
                    let segment = res.get_segment();

                    //println!("segment : {:?}", segment);

                    mut_ref_tdec_a.add_s_and_n(segment.clone(), nb);
                    mut_ref_tdec_b.add_s_and_n(segment, na);
                }

                InfoTxT::CoplanarIntersecting => {
                    let mut polygon = res.get_polygon();
                    mut_ref_tdec_b.add_polygon(polygon.clone());

                    polygon.normal = nb;
                    mut_ref_tdec_a.add_polygon(polygon);
                }

                _ => {}
            }
        }
        info!("The triangle descriptors have been created in {0} seconds.\n", td_start.to(PreciseTime::now()));

        let triangulation_start = PreciseTime::now();
        info!("<triangulate_all> has been started ...");
        let mut new_ts_a : HashMap<Triangle, Marker> = triangulate_all(a_it_to_tdec);
        let mut new_ts_b : HashMap<Triangle, Marker> = triangulate_all(b_it_to_tdec);
        info!("Triangulated intersection area, for model A, contains {0} triangles.", new_ts_a.len());
        info!("Triangulated intersection area, for model B, contains {0} triangles.", new_ts_b.len());
        info!("<triangulate_all> has been performed in {0} seconds.\n", triangulation_start.to(PreciseTime::now()));

        let mut a_it_to_marker : HashMap<usize, Marker> = HashMap::new();
        let mut b_it_to_marker : HashMap<usize, Marker> = HashMap::new();

        let dfs_start = PreciseTime::now();
        info!("<dfs> has been started ...");
        add_triangles(new_ts_a, &mut mesh_a, &mut a_it_to_marker);
        add_triangles(new_ts_b, &mut mesh_b, &mut b_it_to_marker);
        dfs(&mut mesh_a, &mut a_it_to_marker);
        dfs(&mut mesh_b, &mut b_it_to_marker);
        info!("<dfs> has been performed in {0} seconds.\n", dfs_start.to(PreciseTime::now()));

        if log_enabled!(LogLevel::Info) {
            fn print_it_to_marker_num_info(it_to_marker: HashMap<usize, Marker>) {
                let mut num_of_inner = 0;
                let mut num_of_outer = 0;
                let mut num_of_unclassified = 0;
                let mut num_of_planar_plus = 0;
                let mut num_of_planar_minus = 0;

                for (it, m) in it_to_marker.into_iter() {
                    match m {
                        Marker::Inner => num_of_inner += 1,
                        Marker::Outer => num_of_outer += 1,
                        Marker::PlanarMinus => num_of_planar_minus += 1,
                        Marker::PlanarPlus => num_of_planar_plus += 1,
                        Marker::Unclassified => num_of_unclassified += 1
                    }
                }

                info!("\t\t{0} inner triangles", num_of_inner);
                info!("\t\t{0} outer triangles", num_of_outer);
                info!("\t\t{0} unclassified triangles", num_of_unclassified);
                info!("\t\t{0} planar-plus triangles", num_of_planar_plus);
                info!("\t\t{0} planar-minus triangles", num_of_planar_minus);
            }

            info!("In model A intermediate mesh were found:");
            print_it_to_marker_num_info(a_it_to_marker.clone());

            info!("In model B intermediate mesh were found:");
            print_it_to_marker_num_info(b_it_to_marker.clone());


            info!("{0} triangles for mesh A, were classified.", a_it_to_marker.len());
            info!("Resulting mesh A has {0} triangles", mesh_a.num_of_triangles());

            info!("{0} triangles for mesh B, were classified.", b_it_to_marker.len());
            info!("Resulting mesh B has {0} triangles", mesh_b.num_of_triangles());

            info!("<BoolOpPerformer::new> is finished in {0} seconds.\n", start.to(PreciseTime::now()));
            info!("----------------------------------------");
        }

        Ok(BoolOpPerformer {
            mesh_a : mesh_a,
            mesh_b : mesh_b,
            a_it_to_marker : a_it_to_marker,
            b_it_to_marker : b_it_to_marker,
        })
    }

    /// This method perform union-operation and return a resulting mesh.
    pub fn union(&self) -> Mesh {
        let start = PreciseTime::now();

        if log_enabled!(LogLevel::Info) {
            info!("Mesh union is performing ...");
        }

        let mut res_mesh = Mesh::new();

        for (it, m) in self.a_it_to_marker.iter() {
            if (*m == Marker::Outer) || (*m == Marker::PlanarPlus) {
                res_mesh.add_triangle(self.mesh_a.get_triangle(*it));
            }
        }


        for (it, m) in self.b_it_to_marker.iter() {
            if *m == Marker::Outer {
                res_mesh.add_triangle(self.mesh_b.get_triangle(*it));
            }
        }


        if log_enabled!(LogLevel::Info) {
            info!("Resulting mesh contains {0} point and {1} triangles", res_mesh.num_of_points(), res_mesh.num_of_triangles());
            info!("Mesh union is finished in {0} seconds.\n", start.to(PreciseTime::now()));
        }

        return res_mesh;
    }

    /// This method perform intersection-operation and return a resulting mesh.
    pub fn intersection(&self) -> Mesh {
        let start = PreciseTime::now();

        if log_enabled!(LogLevel::Info) {
            info!("Mesh intersection is performing ...");
        }

        let mut res_mesh = Mesh::new();
        for (it, m) in self.a_it_to_marker.iter() {
            if (*m == Marker::Inner) || (*m == Marker::PlanarPlus) {
                res_mesh.add_triangle(self.mesh_a.get_triangle(*it));
            }
        }

        for (it, m) in self.b_it_to_marker.iter() {
            if *m == Marker::Inner {
                res_mesh.add_triangle(self.mesh_b.get_triangle(*it));
            }
        }

        if log_enabled!(LogLevel::Info) {
            info!("Resulting mesh contains {0} point and {1} triangles", res_mesh.num_of_points(), res_mesh.num_of_triangles());
            info!("Mesh intersection is finished in {0} seconds.\n", start.to(PreciseTime::now()));
        }

        return res_mesh;
    }

    /// This method perform difference-operation and return a resulting mesh.
    pub fn difference(&self) -> Mesh {
        let start = PreciseTime::now();

        if log_enabled!(LogLevel::Info) {
            info!("Mesh difference is performing ...");
        }

        let mut res_mesh = Mesh::new();
        for (it, m) in self.a_it_to_marker.iter() {
            if (*m == Marker::Outer) || (*m == Marker::PlanarMinus) {
                res_mesh.add_triangle(self.mesh_a.get_triangle(*it));
            }
        }

        for (it, m) in self.b_it_to_marker.iter() {
            if *m == Marker::Inner {
                res_mesh.add_triangle(self.mesh_b.get_reversed_triangle(*it));
            }
        }

        if log_enabled!(LogLevel::Info) {
            info!("Resulting mesh contains {0} point and {1} triangles", res_mesh.num_of_points(), res_mesh.num_of_triangles());
            info!("Mesh difference is finished in {0} seconds.\n", start.to(PreciseTime::now()));
        }

        return res_mesh;
    }
}

#[cfg(test)]
mod tests {
    use primitives::*;
    use bool_op::BoolOpPerformer;
    use std::fs::File;
    use env_logger::init  as env_logger_init;

    enum BoolOpType {
        Union,
        Intersection,
        Difference
    }

    macro_rules! pattern_str {
        () =>  ("res_of_tests/simple_bool_op/{0}/{1}")
    }

    fn bool_op_test(input_file_name_a: &str, input_file_name_b: &str,
                    test_index: usize,
                    operations: Vec<BoolOpType>,
                    geometry_check: bool
    ) {
        env_logger_init().unwrap_or_else(|x| ->  () {});

        let mut fa = File::open(input_file_name_a).unwrap();
        let mut fb = File::open(input_file_name_b).unwrap();
        let ma : Mesh = Mesh::read_stl(& mut fa).unwrap();
        let mb : Mesh = Mesh::read_stl(& mut fb).unwrap();

        if geometry_check && (!ma.geometry_check() || !mb.geometry_check()) {
            panic!("Geometry check failed!")
        }

        let inter_res_a_file_name = format!(pattern_str!(), test_index, "inter_a.stl");
        let bool_op_performer : BoolOpPerformer = BoolOpPerformer::new(&ma, &mb).expect("The error was raised!");
        let mut ia_f = File::create(inter_res_a_file_name).unwrap();
        match bool_op_performer.mesh_a.write_stl(&mut ia_f) {
            Ok(_) => (),
            Err(_) => panic!("Can not write into file!")
        };

        let inter_res_b_file_name = format!(pattern_str!(), test_index, "inter_b.stl");
        let mut ib_f = File::create(inter_res_b_file_name).unwrap();
        match bool_op_performer.mesh_b.write_stl(&mut ib_f) {
            Ok(_) => (),
            Err(_) => panic!("Can not write into file!")
        };

        for op_type in operations {
            let mut m = match op_type {
                BoolOpType::Union => bool_op_performer.union(),
                BoolOpType::Difference => bool_op_performer.difference(),
                BoolOpType::Intersection => bool_op_performer.intersection()
            };

            let output_file_name = match op_type {
                BoolOpType::Union => format!(pattern_str!(), test_index, "union_res.stl"),
                BoolOpType::Difference => format!(pattern_str!(), test_index, "difference_res.stl"),
                BoolOpType::Intersection => format!(pattern_str!(), test_index, "intersection_res.stl")
            };

            let mut f = File::create(output_file_name).unwrap();
            match m.write_stl(&mut f) {
                Ok(_) => (),
                Err(_) => panic!("Can not write into file!")
            };

            if geometry_check && !m.geometry_check() {
                panic!("Geometry check failed!")
            }
        }
    }

    #[test]
    fn test0() {
        bool_op_test("input_for_tests/plane2.stl",
                     "input_for_tests/plane1.stl",
                     0, vec![BoolOpType::Union], false);
    }

    #[test]
    fn test1() {
        bool_op_test("input_for_tests/cube_in_origin.stl",
                   "input_for_tests/scaled_shifted_cube.stl",
                   1, vec![BoolOpType::Union, BoolOpType::Difference, BoolOpType::Intersection], true);
    }

    #[test]
    fn test2() {
        //cargo test first_union_test -- --nocapture
        bool_op_test("input_for_tests/cube_in_origin.stl",
                   "input_for_tests/long_scaled_shifted_cube.stl",
                   2, vec![BoolOpType::Union, BoolOpType::Difference, BoolOpType::Intersection], true);
    }

    #[test]
    fn test3() {
        bool_op_test("input_for_tests/sphere_in_origin.stl",
                     "input_for_tests/long_scaled_shifted_cube.stl",
                     3, vec![BoolOpType::Union, BoolOpType::Difference, BoolOpType::Intersection], true);
    }

    #[test]
    fn test4() {
        bool_op_test("input_for_tests/sphere_in_origin.stl",
                   "input_for_tests/cone_in_origin.stl",
                   4, vec![BoolOpType::Union, BoolOpType::Difference, BoolOpType::Intersection], true);
    }

    #[test]
    fn test5() {
        bool_op_test("input_for_tests/cube_in_origin.stl",
                     "input_for_tests/skew_cube_in_origin.stl",
                     5, vec![BoolOpType::Union, BoolOpType::Difference, BoolOpType::Intersection], true);
    }

    #[ignore]
    #[test]
    fn test6() {
        bool_op_test("input_for_tests/skull.stl",
                     "input_for_tests/sphere_in_origin.stl",
                     6, vec![BoolOpType::Union, BoolOpType::Difference, BoolOpType::Intersection], false);
    }


    #[test]
    fn test7() {
        // результат операции - поверхность с самопересечением.

        bool_op_test("input_for_tests/cube_in_origin.stl",
                     "input_for_tests/skew_cube_in_origin2.stl",
                     7, vec![BoolOpType::Union, BoolOpType::Difference, BoolOpType::Intersection], false);
    }

    #[test]
    fn test8() {
        // результат операции - поверхность с самопересечением.
        bool_op_test("input_for_tests/cube_in_origin.stl",
                     "input_for_tests/moved_cube.stl",
                     8, vec![BoolOpType::Union, BoolOpType::Difference, BoolOpType::Intersection], false);
    }

    #[test]
    fn test9() {
        bool_op_test("input_for_tests/cube_in_origin.stl",
                     "input_for_tests/moved_cube_not_skewed.stl",
                     9  , vec![BoolOpType::Union, BoolOpType::Difference, BoolOpType::Intersection], true);
    }

}