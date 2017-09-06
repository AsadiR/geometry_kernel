use primitives::*;
use intersect::mesh_x_mesh;
use intersect::triangle_x_triangle::InfoTxT;
use triangulation::incremental_triangulation::triangulate;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::hash_set;
use std::vec;

use log::LogLevel;
use time::PreciseTime;

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

pub struct BoolOpPerformer {
    mesh_a : Mesh,
    mesh_b : Mesh,
    a_it_to_marker : HashMap<usize, Marker>,
    b_it_to_marker : HashMap<usize, Marker>
}

fn find_conjugated_point(tr: &Triangle, org: &Point, dest: &Point) -> Option<Point> {
    for i in 0..3 {
        let ref p_cur = tr.points[i];
        let ref p_next = tr.points[(i+1)%3];
        if (p_cur == org) && (p_next == dest) || (p_cur == dest) && (p_next == org) {
            return Some(tr.points[(i+2)%3].clone())
        }
    }
    return None;
}

fn first_classification(t: &Triangle, tdesc: &mut TDesc) -> Marker {
    println!("First classification for {:?} was started", t);
    for (s, n) in tdesc.get_s_and_n_drain_iter() {
        println!("s: {:?}", s);

        let (org, dest) = s.get_org_dest();
        let ocp = find_conjugated_point(t, &org, &dest);
        let s = Segment::new(org, dest);
        match ocp {
            Some(p) => {
                let p_proj = s.get_point_projection(&p);
                let v = p - p_proj;
                let dot_vn = v.dot_product(&n);
                if dot_vn.is_it_negative() {
                    println!("It's an outer triangle\n");
                    return Marker::Outer;
                } else {
                    println!("It's an inner triangle\n");
                    return  Marker::Inner;
                }
            },
            _ => {}
        }
    }
    println!("It's an unclassified triangle\n");
    return Marker::Unclassified;
}

fn second_classification(t: &Triangle, tdesc: &mut TDesc) -> Marker {
    for polygon in tdesc.get_polygons_drain_iter() {
        let pn = polygon.normal.clone();
        let ps = polygon.get_points();
        let ps_len = ps.len();
        let tn = t.get_normal();

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

fn triangulate_all(it_to_desc: HashMap<usize, TDesc>) -> Vec<(Triangle, Marker)> {
    let mut new_ts : Vec<(Triangle, Marker)>  = Vec::new();
    for (it, mut tdesc) in it_to_desc.into_iter() {
        let mut v : Vec<Point> = Vec::new();
        for p in tdesc.get_points_drain_iter() {
            v.push(p);
        }
        let ts = triangulate(v, tdesc.plane.clone());
        for mut t in ts {
            let m1 = first_classification(&t, &mut tdesc);
            let m2 = second_classification(&t, &mut tdesc);
            let res_marker = if m2.is_it_planar() {
                m2
            } else {
                m1
            };
            new_ts.push((t, res_marker));
        }
    }
    return new_ts;
}

// Decorator for "old" triangle
struct TDesc {
    it : usize,
    points : HashSet<Point>,
    polygons : Vec<Polygon>,
    s_and_n : Vec<(Segment, Vector)>,
    plane: Plane
}

impl TDesc {
    pub fn new(it: usize, plane: Plane) -> TDesc {
        TDesc {
            it: it,
            points: HashSet::new(),
            polygons: Vec::new(),
            s_and_n: Vec::new(),
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

    pub fn add_s_and_n(&mut self, s: Segment, n: Vector) {
        self.s_and_n.push((s.clone(), n));
        let (org, dest) = s.get_org_dest();
        self.points.insert(org);
        self.points.insert(dest);
    }

    pub fn get_points_drain_iter(&mut self) -> hash_set::Drain<Point> {
        return self.points.drain();
    }

    pub fn get_s_and_n_drain_iter(&mut self) -> vec::Drain<(Segment, Vector)> {
        let len = self.s_and_n.len();
        return self.s_and_n.drain(0..len);
    }

    pub fn get_polygons_drain_iter(&mut self) -> vec::Drain<Polygon> {
        let len = self.polygons.len();
        return self.polygons.drain(0..len);
    }

}


impl BoolOpPerformer {
    pub fn new(mesh_a_ref: &Mesh, mesh_b_ref: &Mesh) -> BoolOpPerformer {
        let start = PreciseTime::now();

        if log_enabled!(LogLevel::Info) {
            info!("<BoolOpPerformer::new> is performing ...");
        }

        let mut mesh_a : Mesh = mesh_a_ref.clone();
        let mut mesh_b : Mesh = mesh_b_ref.clone();

        let mxm_res = mesh_x_mesh::intersect(&mesh_a, &mesh_b);

        let mut a_it_to_tdec : HashMap<usize, TDesc> = HashMap::new();
        let mut b_it_to_tdec : HashMap<usize, TDesc> = HashMap::new();

        let mxm_res_lst = mxm_res.get_res_list();
        info!("There are {0} pairs of intersecting triangles.", mxm_res_lst.len());

        for (index_a, index_b, res) in  mxm_res_lst{
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
                for p in mesh_a_ref.get_triangle(index_a).points.iter() {
                    mut_ref_tdec_a.add_point(p.clone());
                }
            }

            if mut_ref_tdec_b.len_of_points() == 0 {
                for p in mesh_b_ref.get_triangle(index_b).points.iter() {
                    mut_ref_tdec_b.add_point(p.clone());
                }
            }

            // вторичное добавление точек фрагментов пересечения
            match res.get_info() {
                InfoTxT::IntersectingInAPoint => {
                    let point = res.get_point();
                    mut_ref_tdec_a.add_point(point.clone());
                    mut_ref_tdec_b.add_point(point);
                },

                InfoTxT::Intersecting => {
                    let segment = res.get_segment();
                    let na = mesh_a_ref.get_normal_by_index(index_a);
                    let nb = mesh_b_ref.get_normal_by_index(index_b);

                    mut_ref_tdec_a.add_s_and_n(segment.clone(), nb);
                    mut_ref_tdec_b.add_s_and_n(segment, na);
                }

                InfoTxT::CoplanarIntersecting => {
                    let polygon = res.get_polygon();
                    mut_ref_tdec_a.add_polygon(polygon.clone());
                    mut_ref_tdec_b.add_polygon(polygon);
                }

                _ => {}
            }
        }

        let mut new_ts_a : Vec<(Triangle, Marker)> = triangulate_all(a_it_to_tdec);
        let mut new_ts_b : Vec<(Triangle, Marker)> = triangulate_all(b_it_to_tdec);

        info!("Triangulated intersection area, for model A, contains {0} triangles.", new_ts_a.len());
        info!("Triangulated intersection area, for model B, contains {0} triangles.", new_ts_b.len());

        let mut a_it_to_marker : HashMap<usize, Marker> = HashMap::new();
        let mut b_it_to_marker : HashMap<usize, Marker> = HashMap::new();

        fn add_triangles(ts_and_ms: Vec<(Triangle, Marker)>, mesh: &mut Mesh, it_to_marker: &mut HashMap<usize, Marker>) {
            for (t, m) in ts_and_ms.into_iter() {
                let it = mesh.add_triangle(t);
                it_to_marker.insert(it, m);
            }
        };

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

        };

        add_triangles(new_ts_a, &mut mesh_a, &mut a_it_to_marker);
        add_triangles(new_ts_b, &mut mesh_b, &mut b_it_to_marker);
        dfs(&mut mesh_a, &mut a_it_to_marker);
        dfs(&mut mesh_b, &mut b_it_to_marker);

        //TODO reverse normals for different bool ops



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
        }

        BoolOpPerformer {
            mesh_a : mesh_a,
            mesh_b : mesh_b,
            a_it_to_marker : a_it_to_marker,
            b_it_to_marker : b_it_to_marker
        }
    }


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
            if (*m == Marker::Outer) || (*m == Marker::PlanarPlus) {
                res_mesh.add_triangle(self.mesh_b.get_triangle(*it));
            }
        }

        if log_enabled!(LogLevel::Info) {
            info!("Resulting mesh contains {0} point and {1} triangles", res_mesh.num_of_points(), res_mesh.num_of_triangles());
            info!("Mesh union is finished in {0} seconds.\n", start.to(PreciseTime::now()));
        }

        return res_mesh;
    }

    pub fn intersection(&self) -> Mesh {
        self.mesh_a.clone()
    }

    pub fn difference(&self) -> Mesh {
        self.mesh_a.clone()
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

    fn bool_op_test(input_file_name_a: &str,
                    input_file_name_b: &str,
                    inter_res_a_file_name: &str,
                    inter_res_b_file_name: &str,
                    output_file_name: &str,
                    op_type: BoolOpType) {
        env_logger_init().unwrap_or_else(|x| ->  () {});

        let mut fa = File::open(input_file_name_a).unwrap();
        let mut fb = File::open(input_file_name_b).unwrap();
        let ma : Mesh = Mesh::read_stl(& mut fa).unwrap();
        let mb : Mesh = Mesh::read_stl(& mut fb).unwrap();


        let bool_op_performer : BoolOpPerformer = BoolOpPerformer::new(&ma, &mb);

        let mut ia_f = File::create(inter_res_a_file_name).unwrap();
        match bool_op_performer.mesh_a.write_stl(&mut ia_f) {
            Ok(_) => (),
            Err(_) => panic!("Can not write into file!")
        };

        let mut ib_f = File::create(inter_res_b_file_name).unwrap();
        match bool_op_performer.mesh_b.write_stl(&mut ib_f) {
            Ok(_) => (),
            Err(_) => panic!("Can not write into file!")
        };

        let mut m = match op_type {
            BoolOpType::Union => bool_op_performer.union(),
            BoolOpType::Difference => bool_op_performer.difference(),
            BoolOpType::Intersection => bool_op_performer.intersection()
        };

        let mut f = File::create(output_file_name).unwrap();
        match m.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!("Can not write into file!")
        };
    }

    #[test]
    fn zero_union_test() {
        bool_op_test("input_for_tests/plane1.stl",
                     "input_for_tests/plane2.stl",
                     "res_of_tests/simple_bool_op/union_test0_inter_a.stl",
                     "res_of_tests/simple_bool_op/union_test0_inter_b.stl",
                     "res_of_tests/simple_bool_op/union_test0.stl",
                     BoolOpType::Union);
    }

    #[test]
    fn first_union_test() {
        bool_op_test("input_for_tests/cube_in_origin.stl",
                   "input_for_tests/scaled_shifted_cube.stl",
                   "res_of_tests/simple_bool_op/union_test1_inter_a.stl",
                   "res_of_tests/simple_bool_op/union_test1_inter_b.stl",
                   "res_of_tests/simple_bool_op/union_test1.stl",
                    BoolOpType::Union);
    }

}