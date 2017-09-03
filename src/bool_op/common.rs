use primitives::*;
use intersect::mesh_x_mesh;
use intersect::triangle_x_triangle::InfoTxT;
use triangulation::incremental_triangulation::triangulate;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::hash_set;
use std::vec;

#[derive(Clone)]
#[derive(PartialEq, Eq)]
#[derive(Debug)]
enum Marker {
    Unclassified,
    Inner,
    Outer,
    Planar
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
    for (s, n) in tdesc.get_s_and_n_drain_iter() {
        let (org, dest) = s.get_org_dest();
        let ocp = find_conjugated_point(t, &org, &dest);
        let s = Segment::new(org, dest);
        match ocp {
            Some(p) => {
                let p_proj = s.get_point_projection(&p);
                let v = p - p_proj;
                let dot_vn = v.dot_product(&n);
                if dot_vn.is_it_negative() {
                    return Marker::Outer;
                } else {
                    return  Marker::Inner;
                }
            },
            _ => {}
        }
    }
    return Marker::Unclassified;
}

fn second_classification(t: &Triangle, tdesc: &mut TDesc) -> Marker {
    for polygon in tdesc.get_polygons_drain_iter() {
        let pn = polygon.normal.clone();
        let ps = polygon.get_points();
        let ps_len = ps.len();
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
                        return Marker::Planar;
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
        let ts = triangulate(v);
        for t in ts {
            let m1 = first_classification(&t, &mut tdesc);
            let m2 = second_classification(&t, &mut tdesc);
            let res_marker = if m2 == Marker::Planar {
                m2
            }else {
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
    s_and_n : Vec<(Segment, Vector)>
}

impl TDesc {
    pub fn new(it: usize) -> TDesc {
        TDesc {
            it: it,
            points: HashSet::new(),
            polygons: Vec::new(),
            s_and_n: Vec::new()
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
        let mut mesh_a : Mesh = mesh_a_ref.clone();
        let mut mesh_b : Mesh = mesh_b_ref.clone();

        let mxm_res = mesh_x_mesh::intersect(&mesh_a, &mesh_b);

        let mut a_it_to_tdec : HashMap<usize, TDesc> = HashMap::new();
        let mut b_it_to_tdec : HashMap<usize, TDesc> = HashMap::new();


        for (index_a, index_b, res) in mxm_res.get_res_list() {
            mesh_a.remove_triangle(&index_a);
            mesh_b.remove_triangle(&index_b);

            if !a_it_to_tdec.contains_key(&index_a) {
                a_it_to_tdec.insert(index_a, TDesc::new(index_a));
            }

            if !b_it_to_tdec.contains_key(&index_b) {
                b_it_to_tdec.insert(index_b, TDesc::new(index_b));
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

        //TODO two types of planar areas
        //TODO reverse normals for different bool ops

        BoolOpPerformer {
            mesh_a : mesh_a,
            mesh_b : mesh_b,
            a_it_to_marker : a_it_to_marker,
            b_it_to_marker : b_it_to_marker
        }
    }


    pub fn union(&self) -> Mesh {
        self.mesh_a.clone()
    }

    pub fn intersection(&self) -> Mesh {
        self.mesh_a.clone()
    }

    pub fn difference(&self) -> Mesh {
        self.mesh_a.clone()
    }
}