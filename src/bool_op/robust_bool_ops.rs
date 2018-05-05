use primitives::*;
use intersect::mesh_x_mesh;
use intersect::triangle_x_triangle::InfoTxT;
use triangulation::*;
use std::collections::{HashMap, BTreeSet};

use log::LogLevel;
use time::PreciseTime;
use std::result::Result;
use std::mem::swap;

use std::fs::File;
use std::path::Path;
use std::fs;



pub struct BoolOpResult {
    re_triangulated_mesh_a : Mesh,
    re_triangulated_mesh_b : Mesh,
    blocks: Blocks
}

#[derive(Clone)]
struct CurveSegment {
    s: Segment,
    // apt = plus triangle of mesh_a
    index_of_apt: usize,
    // amt = minus triangle of mesh_a
    index_of_amt: usize,
    // bpt = plus triangle of mesh_b
    index_of_bpt: usize,
    // bmt = minus triangle of mesh_b
    index_of_bmt: usize
}


impl CurveSegment {
    pub fn new(
        s: Segment,
        index_of_apt: usize, index_of_amt: usize,
        index_of_bpt: usize, index_of_bmt: usize
    ) -> CurveSegment {
        return CurveSegment {
            s,
            index_of_apt, index_of_amt,
            index_of_bpt, index_of_bmt,
        };
    }

    pub fn flip(&mut self) {
        self.s.flip();
        swap(&mut self.index_of_apt, &mut self.index_of_amt);
        swap(&mut self.index_of_bpt, &mut self.index_of_bmt)
    }
}

#[derive(Clone)]
struct Curve {
    css: Vec<CurveSegment>,
    indexes_of_positive_sub_surfaces: BTreeSet<usize>,
    indexes_of_negative_sub_surfaces: BTreeSet<usize>
}

impl PartialEq for Curve {
    fn eq(&self, rhs: &Curve) -> bool {
        return self.get_a_its() == rhs.get_a_its() && self.get_b_its() == rhs.get_b_its();
    }
}

impl Eq for Curve {}

impl Curve {
    // WARN после ретриангуляции индексы поменялись!!!! их использовать нельзя!!!
    pub fn new(css: Vec<CurveSegment>) -> Curve {
        return Curve {
            css,
            indexes_of_positive_sub_surfaces: BTreeSet::new(),
            indexes_of_negative_sub_surfaces: BTreeSet::new()
        };
    }

    pub fn get_a_its(&self) -> BTreeSet<usize> {
        let mut a_its: BTreeSet<usize> = BTreeSet::new();
        for cs in self.css.iter() {
            a_its.insert(cs.index_of_amt);
            a_its.insert(cs.index_of_apt);
        }
        return a_its;
    }

    pub fn get_b_its(&self) -> BTreeSet<usize> {
        let mut b_its: BTreeSet<usize> = BTreeSet::new();
        for cs in self.css.iter() {
            b_its.insert(cs.index_of_bmt);
            b_its.insert(cs.index_of_bpt);
        }
        return b_its;
    }


    // алгоритм работает корректно только если кривые из сегментов можно построить единственным образом!!!
    pub fn new_curves(
        it_to_ss: HashMap<usize, Vec<Segment>>, mesh_a: &Mesh, mesh_b: &Mesh
    ) -> Vec<Curve> {

        let mut res: Vec<Curve> = Vec::new();
        let mut all_css: Vec<CurveSegment> = Vec::new();

        for (_, ss) in it_to_ss {
            for s in ss {
                // println!("s = {:?}", s);
                let (index_of_apt, index_of_amt) = mesh_a.get_indexes_of_triangles_by_two_points(&s.org, &s.dest).unwrap();
                let (index_of_bpt, index_of_bmt) = mesh_b.get_indexes_of_triangles_by_two_points(&s.org, &s.dest).unwrap();
                let new_cs = CurveSegment::new(s, index_of_apt, index_of_amt, index_of_bpt, index_of_bmt);
                // println!("new_cs {:?}", new_cs.s);
                all_css.push(new_cs);
            }
        }

        // println!("all_css.len() = {0}", all_css.len());
        // генерирую кривые с определенными направлениями!!!

        fn find_next(all_css: &mut Vec<CurveSegment>, cur_cs: &CurveSegment) -> Option<CurveSegment> {
            for i in 0..all_css.len() {
                let mut cs : CurveSegment = all_css[i].clone();
                if cs.s.org == cur_cs.s.dest {
                    all_css.remove(i);
                    return Some(cs);
                } else if cs.s.dest == cur_cs.s.dest {
                    all_css.remove(i);
                    cs.flip();
                    return Some(cs);
                }
            }
            return None;
        }

        while all_css.len() != 0 {
            let mut cur_cs = all_css.pop().unwrap();
            let mut css: Vec<CurveSegment> = Vec::new();
            loop {
                // println!("cur_cs {:?}", cur_cs.s);
                css.push(cur_cs.clone());
                let opt_next_cs = find_next(&mut all_css, &cur_cs);
                let css_len = css.len();
                if opt_next_cs.is_none() {
                    // println!("first = {:?}", css[0].s);
                    // println!("last = {:?}", css[css_len-1].s);
                    if css[0].s.org != css[css_len-1].s.dest {
                        // println!("Opened loop was found!\n");
                    } else {
                        // println!("Closed loop was found!\n");
                        res.push(Curve::new(css));
                    }
                    break;
                }
                // println!("css.len() = {0}", css.len());

                let next_cs = opt_next_cs.unwrap();
                cur_cs = next_cs;

            }
        }

        assert_ne!(res.len(), 0);
        return res;
    }

    pub fn new_curves_from_polygons(
        polygons: Vec<Polygon>,
        mesh_a: &Mesh, mesh_b: &Mesh
    ) -> Vec<Curve> {

        let mut ss: Vec<Segment> = Vec::new();
        fn find(ss: &Vec<Segment>, needed_s: &Segment) -> Option<usize> {
            for (index, s) in ss.iter().enumerate() {
                if s == needed_s {
                    return Some(index);
                }
            }
            return None;
        }

        for cur_p in polygons.iter() {
            for s in cur_p.get_segments() {
                let opt_index = find(&ss, &s);
                if opt_index.is_none() {
                    ss.push(s);
                } else {
                    ss.remove(opt_index.unwrap());
                }
            }
        }

        let mut adapter: HashMap<usize, Vec<Segment>> = HashMap::new();
        adapter.insert(0, ss);

        return Curve::new_curves(adapter, mesh_a, mesh_b);

    }

    // проверяет являются ли треугольники смежными по отрезку, принадлежащему кривой.
    pub(crate) fn are_they_twins(&self, it1: &usize, it2: &usize, from_what_mesh_is_it: &EMesh) -> bool {
        for cs in self.css.iter() {
            if *from_what_mesh_is_it == EMesh::MeshA {
                if &cs.index_of_apt == it1 && &cs.index_of_amt == it2 || &cs.index_of_apt == it2 && &cs.index_of_amt == it1 {
                   return true;
                }
            }

            if *from_what_mesh_is_it == EMesh::MeshB {
                if &cs.index_of_bpt == it1 && &cs.index_of_bmt == it2 || &cs.index_of_bpt == it2 && &cs.index_of_bmt == it1 {
                    return true;
                }
            }
        }

        return false;
    }

    pub(crate) fn is_it_positive(&self, it: usize, from_what_mesh_is_it: &EMesh) -> bool {
        for cs in self.css.iter() {
            if *from_what_mesh_is_it == EMesh::MeshA {
                if cs.index_of_apt == it {
                    return true;
                }
                if cs.index_of_amt == it {
                    return false;
                }
            }

            if *from_what_mesh_is_it == EMesh::MeshB {
                if cs.index_of_bpt == it {
                    return true;
                }
                if cs.index_of_bmt == it {
                    return false;
                }
            }
        }

        panic!("The triangle with such index does not exist in the curve!");
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum EMesh {
    MeshA,
    MeshB
}

#[derive(Clone)]
struct SubSurface {
    indexes_of_ts: BTreeSet<usize>,
    indexes_of_positive_curves: BTreeSet<usize>,
    indexes_of_negative_curves: BTreeSet<usize>,
    from_what_mesh_is_it: EMesh
}



impl SubSurface {
    pub fn new(from_what_mesh_is_it: EMesh) -> SubSurface {
        return SubSurface {
            indexes_of_ts: BTreeSet::new(),
            indexes_of_positive_curves: BTreeSet::new(),
            indexes_of_negative_curves: BTreeSet::new(),
            from_what_mesh_is_it
        }
    }

    pub fn add_to_mesh(&self, resulting_mesh: &mut Mesh, mesh: &Mesh, reversed: bool) {
        for it in self.indexes_of_ts.iter() {
            let t = if reversed {
                mesh.get_reversed_triangle(it.clone())
            } else {
                mesh.get_triangle(it.clone())
            };

            resulting_mesh.add_triangle(t).is_ok();
        }
    }

    pub fn add_sub_surfaces(
        curves: &mut Vec<Curve>,
        sub_surfaces: &mut Vec<SubSurface>,
        mesh_a: &Mesh, mesh_b: &Mesh
    ) {
        let mut it_to_ic_for_a: HashMap<usize, BTreeSet<usize>> = HashMap::new();
        let mut it_to_is_for_a: HashMap<usize, usize> = HashMap::new();

        let mut it_to_ic_for_b: HashMap<usize, BTreeSet<usize>> = HashMap::new();
        let mut it_to_is_for_b: HashMap<usize, usize> = HashMap::new();

        fn insert(it_to_ic: &mut HashMap<usize, BTreeSet<usize>>, it: &usize, ic: &usize) {
            if !it_to_ic.contains_key(it) {
                it_to_ic.insert(it.clone(), BTreeSet::new());
            }
            it_to_ic.get_mut(it).unwrap().insert(ic.clone());
        }

        for (index, curve) in curves.iter().enumerate() {
            for cs in curve.css.iter() {
                insert(&mut it_to_ic_for_a, &cs.index_of_apt, &index);
                insert(&mut it_to_ic_for_a,&cs.index_of_amt, &index);

                insert(&mut it_to_ic_for_b,&cs.index_of_bpt, &index);
                insert(&mut it_to_ic_for_b,&cs.index_of_bmt, &index);
            }
        }

        for curve in curves.clone().iter() {
            for cs in curve.css.iter() {
                SubSurface::add_sub_surface(
                    &cs.index_of_apt,
                    EMesh::MeshA,
                    &it_to_ic_for_a,
                    &mut it_to_is_for_a,
                    curves, sub_surfaces, mesh_a);

                SubSurface::add_sub_surface(
                    &cs.index_of_amt,
                    EMesh::MeshA,
                    &it_to_ic_for_a,
                    &mut it_to_is_for_a,
                    curves, sub_surfaces, mesh_a);

                SubSurface::add_sub_surface(
                    &cs.index_of_bpt,
                    EMesh::MeshB,
                    &it_to_ic_for_b,
                    &mut it_to_is_for_b,
                    curves, sub_surfaces, mesh_b);

                SubSurface::add_sub_surface(
                    &cs.index_of_bmt,
                    EMesh::MeshB,
                    &it_to_ic_for_b,
                    &mut it_to_is_for_b,
                    curves, sub_surfaces, mesh_b);
            }
        }
    }

    fn add_sub_surface(
        start_index: &usize,
        from_what_mesh_is_it: EMesh,
        it_to_ic: &HashMap<usize, BTreeSet<usize>>,
        it_to_is: &mut HashMap<usize,usize>,
        curves: &mut Vec<Curve>,
        sub_surfaces: &mut Vec<SubSurface>,
        mesh: &Mesh
    ) {
        /*
        При добавлении sub-surface нужно:
        добавить его в <sub_surfaces>,
        внутри него нужно задать ссылки на ориентированные кривые,
        кривые должны ссылаться на subsurface,
        нужно знать к какой поверхности относится subsurface
        */

        let mut sub_surface = SubSurface::new(from_what_mesh_is_it);
        let mut stack: Vec<usize> = vec![start_index.clone()];
        let is = sub_surfaces.len();

        while !stack.is_empty() {
            let cur_it = stack.pop().unwrap();

            // Текущий треугольник уже размечен индексом некоторого subsurface
            if it_to_is.contains_key(&cur_it) {
                continue
            }

            let ins = mesh.find_segment_conjugated_triangles(cur_it);
            for int in ins {

                // Текущий трегольник и его сосед являются граничными и индексы кривых этих треугольников совпадают.
                if it_to_ic.contains_key(&int) && it_to_ic.contains_key(&cur_it)
                   // it_to_ic.get(&int).unwrap() == it_to_ic.get(&cur_it).unwrap()
                {
                    let indexes_of_curves = it_to_ic.get(&int).unwrap();
                    assert!(indexes_of_curves.len() <= 2);

                    let mut continue_flag = false;
                    for ic in indexes_of_curves.iter() {
                        if curves[*ic].are_they_twins(&int, &cur_it, &sub_surface.from_what_mesh_is_it) {
                            continue_flag = true;
                        }
                    }

                    if continue_flag {
                        // println!("continue");
                        continue;
                    }
                }

                // Сосед уже размечен индексом некоторого subsurface
                if it_to_is.contains_key(&int) {
                    continue
                }

                // Сосед является граничным, а текущий нет. При этом они оба не размеченные.
                if it_to_ic.contains_key(&int) {
                    sub_surface.update_curves_and_subsurface(int.clone(), &it_to_ic[&int], is.clone(), curves);
                }

                // Текущий - граничный, а сосед - обычный. При этом они оба не размеченные.
                if it_to_ic.contains_key(&cur_it) {
                    sub_surface.update_curves_and_subsurface(cur_it.clone(), &it_to_ic[&cur_it], is.clone(), curves);
                }

                // Оба обычные и неразмеченные.
                // println!("push");
                stack.push(int);
            }

            // размечаем текущий треугольник
            it_to_is.insert(cur_it.clone(), is.clone());
            sub_surface.indexes_of_ts.insert(cur_it);
        }

        // println!("sub_surface.indexes_of_ts.len() = {0}", sub_surface.indexes_of_ts.len());

        if !sub_surface.indexes_of_ts.is_empty() {
            sub_surfaces.push(sub_surface);
        }
    }

    fn update_curves_and_subsurface(
        &mut self, it: usize, indexes_of_curves: &BTreeSet<usize>,
        is: usize, curves: &mut Vec<Curve>
    ) {
        if indexes_of_curves.len() != 1 {
            return;
        }

        for ic in indexes_of_curves {
            if curves[*ic].is_it_positive(it, &self.from_what_mesh_is_it) {
                self.indexes_of_positive_curves.insert(*ic);
                curves[*ic].indexes_of_positive_sub_surfaces.insert(is);
            } else {
                self.indexes_of_negative_curves.insert(*ic);
                curves[*ic].indexes_of_negative_sub_surfaces.insert(is);
            }
        }
    }
}


/*
так как на вход поступают связные поверхности, то:
результат пересечения - 1..N поверхностей
результат объединения - 1 поверхность.
результат дополнения - 1..N повехностей
*/

struct Blocks {
    union: Mesh,
    intersections: Vec<Mesh>,
    difs_ab: Vec<Mesh>,
    difs_ba: Vec<Mesh>,
}

impl Blocks {
    pub fn new(
        it_to_ss_for_mesh_a: HashMap<usize, Vec<Segment>>,
        // polygons: Vec<Polygon>,
        mesh_a: &Mesh,
        mesh_b: &Mesh,
    ) -> Blocks {
        /*

        Нужно завести класс, в котором будут хранитсят замкнутые кривые.
        Каждый сегмент кривой ссылается на два треугольника.
        Иначе самопересечение.
        У каждого mesh-а будет свой набор кривых.
        У каждой кривой должен быть метод grow, возвращающий 2 subsurface-а.
        Если такого subsurface-а нет, то он добавляется. Если есть, то второй, найденный на предыдущем шаге связывается с текущем.
        В итоге имеем граф subsurface-ов. Однако он неразмеченный.

        Исходя из принципов distinguishing-а нужно задать метки ребрам между subsurface-ами
        По меткам собираем результирующие поверхности.
        */
        let log_meshes = true;

        if log_enabled!(LogLevel::Info) && log_meshes {
            let dir_path = "res_of_tests/robust_bool_op/dbg";
            if Path::new(&dir_path).exists() {
                fs::remove_dir_all(&dir_path).ok();
            }
            fs::create_dir_all(&dir_path).ok();

            Blocks::write_mesh(mesh_a.clone(), "retr_a.stl");
            Blocks::write_mesh(mesh_b.clone(), "retr_b.stl");

            assert!(mesh_a.geometry_check());
            assert!(mesh_b.geometry_check());
        }

        let mut curves: Vec<Curve> = Curve::new_curves(it_to_ss_for_mesh_a, mesh_a, mesh_b);
        info!("There were constructed {0} curves.", curves.len());

        let mut sub_surfaces: Vec<SubSurface> = Vec::new();
        SubSurface::add_sub_surfaces(&mut curves, &mut sub_surfaces, mesh_a, mesh_b);

        info!("There were constructed {0} sub-surfaces.", sub_surfaces.len());
        // println!("mesh_a.len() = {0}", mesh_a.num_of_triangles());
        // println!("mesh_b.len() = {0}", mesh_b.num_of_triangles());

        if log_enabled!(LogLevel::Info) && log_meshes {
            Blocks::write_sub_surfaces(&sub_surfaces, mesh_a, mesh_b);
        }

        let blocks_ui = Blocks::dfs(&sub_surfaces, &curves, false);
        let blocks_dif = Blocks::dfs(&sub_surfaces, &curves, true);

        let (union, block_union, intersections) =
            Blocks::distinguish_u_and_i(blocks_ui, &sub_surfaces, mesh_a, mesh_b);

        if log_enabled!(LogLevel::Info) && log_meshes {
            Blocks::write_mesh(union.clone(), "union");

            for (i, mesh) in intersections.iter().enumerate() {
                Blocks::write_mesh(mesh.clone(), &format!("intersection_{0}", i));
            }
        }

        let (difs_ab, difs_ba) = Blocks::distinguish_difs(
            blocks_dif, &sub_surfaces,
            mesh_a, mesh_b, block_union,
        );

        if log_enabled!(LogLevel::Info) && log_meshes {
            for (i, mesh) in difs_ab.iter().enumerate() {
                Blocks::write_mesh(mesh.clone(), &format!("dif_ab_{0}", i));
            }

            for (i, mesh) in difs_ba.iter().enumerate() {
                Blocks::write_mesh(mesh.clone(), &format!("dif_ba_{0}", i));
            }
        }

        return Blocks {
            union,
            intersections,
            difs_ab,
            difs_ba
        };
    }

    fn get_mesh(
        sub_surfaces: &Vec<SubSurface>,
        mesh_a: &Mesh, mesh_b: &Mesh,
        reversed_mesh_a: bool, reversed_mesh_b: bool
    ) -> Mesh {
        let mut res = Mesh::new();
        for sub_surface in sub_surfaces.iter() {
            if sub_surface.from_what_mesh_is_it == EMesh::MeshA {
                //println!("from mesh a");
                sub_surface.add_to_mesh(&mut res, mesh_a, reversed_mesh_a);
            } else {
                //println!("from mesh b");
                sub_surface.add_to_mesh(&mut res, mesh_b, reversed_mesh_b);
            }
        }
        return res;
    }

    fn get_mesh_from_block(
        sub_surfaces: &Vec<SubSurface>, block: &BTreeSet<usize>,
        mesh_a: &Mesh, mesh_b: &Mesh,
        reversed_mesh_a: bool, reversed_mesh_b: bool
    ) -> Mesh {
        let mut block_sub_surfaces : Vec<SubSurface> = Vec::new();
        for i in block.iter() {
            block_sub_surfaces.push(sub_surfaces[*i].clone());
        }
        return Blocks::get_mesh(&block_sub_surfaces, mesh_a, mesh_b, reversed_mesh_a, reversed_mesh_b);
    }

    fn write_mesh(mesh: Mesh, file_name: &str) {
        let output_file_name = format!("res_of_tests/robust_bool_op/dbg/{0}.stl", file_name);
        let mut f = File::create(output_file_name).unwrap();
        //println!("cur_mesh.len()= {0}", cur_mesh.num_of_triangles());
        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!("Can not write into file!")
        };
    }

    fn write_sub_surfaces(sub_surfaces: &Vec<SubSurface>, mesh_a: &Mesh, mesh_b: &Mesh) {
        for (index, s) in sub_surfaces.iter().enumerate() {
            //println!("posc = {:?} negc = {:?}", s.indexes_of_positive_curves, s.indexes_of_negative_curves);

            let cur_mesh = Blocks::get_mesh(&vec![s.clone()], mesh_a, mesh_b, false, false);
            let output_file_name = format!("res_of_tests/robust_bool_op/dbg/{0}.stl", index);

            let mut f = File::create(output_file_name).unwrap();
            //println!("cur_mesh.len()= {0}", cur_mesh.num_of_triangles());
            match cur_mesh.write_stl(&mut f) {
                Ok(_) => (),
                Err(_) => panic!("Can not write into file!")
            };
        }
    }

    fn dfs(
        sub_surfaces: &Vec<SubSurface>, curves: &Vec<Curve>,
        are_we_looking_for_difs: bool
    ) -> Vec<BTreeSet<usize>> {
        /*
        Выбираю произвольный subsurface. Далее из него перехожу по кривым со сменой знака обхода.
        Помечаю пройденные. По завершению, беру следующий subsurface и повторяю процедуру.
        В результает имею два списка индексов subsurface-ов. Один - это пересечение, второй - это объединение.
        */
        let mut vec_of_blocks: Vec<BTreeSet<usize>> = Vec::new();

        let mut visited: BTreeSet<usize> = BTreeSet::new();

        // println!("\ndfs");

        for is in 0..sub_surfaces.len() {
            if visited.contains(&is) {
                continue;
            }

            let mut stack: BTreeSet<usize> = BTreeSet::new();
            stack.insert(is);
            let mut block: BTreeSet<usize> = BTreeSet::new();

            while !stack.is_empty() {
                let cur_is: usize = stack.iter().next().unwrap().clone();
                stack.remove(&cur_is);
                if visited.contains(&cur_is) {
                    continue;
                }
                // println!("cur_is = {0}", cur_is);

                block.insert(cur_is.clone());
                visited.insert(cur_is);

                let cur_ss_mesh_marker: EMesh;
                let mut indexes_of_nearest_ss: Vec<usize> = Vec::new();
                {
                    let cur_ss: &SubSurface = &sub_surfaces[cur_is];
                    cur_ss_mesh_marker = cur_ss.from_what_mesh_is_it.clone();

                    if !are_we_looking_for_difs {
                        for ic in cur_ss.indexes_of_positive_curves.clone() {
                            indexes_of_nearest_ss.extend(curves[ic].indexes_of_negative_sub_surfaces.clone());
                        }

                        for ic in cur_ss.indexes_of_negative_curves.clone() {
                            indexes_of_nearest_ss.extend(curves[ic].indexes_of_positive_sub_surfaces.clone());
                        }
                    } else {
                        for ic in cur_ss.indexes_of_positive_curves.clone() {
                            indexes_of_nearest_ss.extend(curves[ic].indexes_of_positive_sub_surfaces.clone());
                        }

                        for ic in cur_ss.indexes_of_negative_curves.clone() {
                            indexes_of_nearest_ss.extend(curves[ic].indexes_of_negative_sub_surfaces.clone());
                        }
                    }
                }
                indexes_of_nearest_ss.retain(
                    |&ins| sub_surfaces.get(ins).unwrap().from_what_mesh_is_it != cur_ss_mesh_marker && ins != cur_is
                );

                stack.extend(indexes_of_nearest_ss);
                // println!("stack = {:?}", stack);
            }

            vec_of_blocks.push(block);
        }

        // println!("block1 {:?}", vec_of_blocks[0]);
        // println!("block2 {:?}", vec_of_blocks[1]);

        // println!("vec_of_blocks_len = {0}", vec_of_blocks.len());
        assert!(vec_of_blocks.len() >= 2);

        return vec_of_blocks;
    }


    fn distinguish_u_and_i(
        mut blocks_ui: Vec<BTreeSet<usize>>,
        sub_surfaces: &Vec<SubSurface>,
        mesh_a: &Mesh, mesh_b: &Mesh
    ) -> (Mesh, BTreeSet<usize>, Vec<Mesh>) {
        let mut meshes: Vec<Mesh> = Vec::new();

        for block in blocks_ui.iter(){
            let mesh = Blocks::get_mesh_from_block(sub_surfaces, block, mesh_a, mesh_b, false, false);
            meshes.push(mesh);
        }

        let mut index_of_the_best: usize = 0;
        let (mut x_min, mut x_max, mut y_min, mut y_max, mut z_min, mut z_max) = meshes[0].find_xyz_ranges();

        for i in 0..meshes.len() {
            let (cur_x_min, cur_x_max, cur_y_min, cur_y_max, cur_z_min, cur_z_max) = meshes[i].find_xyz_ranges();
            if cur_x_min <= x_min && cur_x_max >= x_max ||
               cur_y_min <= y_min && cur_y_max >= y_max ||
               cur_z_min <= z_min && cur_z_max >= z_max {
                x_min = cur_x_min;
                x_max = cur_x_max;
                y_min = cur_y_min;
                y_max = cur_y_max;
                z_min = cur_z_min;
                z_max = cur_z_max;

                index_of_the_best = i;
            }
        }
        let union = meshes.remove(index_of_the_best);
        let union_block = blocks_ui.remove(index_of_the_best);

        meshes.retain(|m| m.geometry_check());

        return (union, union_block, meshes);
    }

    fn distinguish_difs(
        blocks_difs: Vec<BTreeSet<usize>>,
        sub_surfaces: &Vec<SubSurface>,
        mesh_a: &Mesh, mesh_b: &Mesh,
        block_union: BTreeSet<usize>
    ) -> (Vec<Mesh>, Vec<Mesh>) {
        let mut difs_ab: Vec<Mesh> = Vec::new();
        let mut difs_ba: Vec<Mesh> = Vec::new();

        for block in blocks_difs {
            let outer_part = block.intersection(&block_union);
            let opt_index = outer_part.into_iter().next();
            if opt_index.is_none() {
                continue;
            }
            let index_of_ss = opt_index.unwrap();
            if sub_surfaces[*index_of_ss].from_what_mesh_is_it == EMesh::MeshA {

                let cur_mesh = Blocks::get_mesh_from_block(
                    sub_surfaces,
                    &block, mesh_a, mesh_b,
                    false, true
                );
                let mut meshes = cur_mesh.split_into_connectivity_components();
                meshes.retain(|m| m.geometry_check());
                difs_ab.extend(meshes);
                // difs_ab.push(cur_mesh);
            } else {
                let cur_mesh = Blocks::get_mesh_from_block(
                    sub_surfaces, &block, mesh_a, mesh_b,
                    true, false
                );
                let mut meshes = cur_mesh.split_into_connectivity_components();
                meshes.retain(|m| m.geometry_check());
                difs_ba.extend(meshes);
                // difs_ba.push(cur_mesh);
            }
        }

        return (difs_ab, difs_ba);
    }

    pub fn get_intersection(&self) -> &Vec<Mesh> {
        return &self.intersections;
    }

    pub fn get_union(&self) -> &Mesh {
        return &self.union;
    }

    pub fn get_difference_ab(&self) -> &Vec<Mesh> {
        return &self.difs_ab;
    }

    pub fn get_difference_ba(&self) -> &Vec<Mesh> {
        return &self.difs_ba;
    }
}


impl BoolOpResult {

    /// This method prepares intermediate structures for performing of boolean operations and saves it in the instance of `BoolOpResult` structure.
    /// If meshes don't intersect each other the `Err` will be returned.
    /// # Arguments
    ///
    /// * `mesh_a_ref` - A reference to the first mesh.
    /// * `mesh_a_ref` - A reference to the second mesh.
    pub fn new(mesh_a_ref: &Mesh, mesh_b_ref: &Mesh) -> Result<BoolOpResult, &'static str> {
        let start = PreciseTime::now();

        if !mesh_a_ref.geometry_check()  {
            return Err("Geometry check failed for first mesh! Each triangle must have three adjacent triangles!");
        } else if !mesh_b_ref.geometry_check() {
            return Err("Geometry check failed for second mesh! Each triangle must have three adjacent triangles!");
        }

        let mut connectivity_components_for_a = mesh_a_ref.clone().split_into_connectivity_components();
        if connectivity_components_for_a.len() != 1 {
            return Err("The first mesh should have only one connectivity component!");
        }

        let mut connectivity_components_for_b = mesh_b_ref.clone().split_into_connectivity_components();
        if connectivity_components_for_b.len() != 1 {
            return Err("The second mesh should have only one connectivity component!");
        }

        if log_enabled!(LogLevel::Info) {
            info!("----------------------------------------");
            info!("<BoolOpResult::new> is performing ...\n");
        }

        let mut mesh_a : Mesh = connectivity_components_for_a.remove(0);
        let mut mesh_b : Mesh = connectivity_components_for_b.remove(0);

        fn add_segment_to_map(it: &usize, s: Segment, t_to_ss: &mut HashMap<usize, Vec<Segment>>) {
            if t_to_ss.contains_key(it) {
                // повторяться отрезки не могут, так как иначе присутствует самопересечение
                // однако если мы имеем дело с плоскостным пересечением то могут
                let vec: &mut Vec<Segment> = t_to_ss.get_mut(it).unwrap();
                if !vec.contains(&s) {
                    vec.push(s);
                }
            } else {
                t_to_ss.insert(it.clone(), vec![s]);
            }
        }

        let mut it_to_ss_for_mesh_a: HashMap<usize, Vec<Segment>> = HashMap::new();
        let mut it_to_ss_for_mesh_b: HashMap<usize, Vec<Segment>> = HashMap::new();

        let m_x_m_start = PreciseTime::now();
        info!("Intersection of meshes is performing ...");
        let mxm_res = mesh_x_mesh::intersect(&mesh_a, &mesh_b, 1);
        info!("<mesh_x_mesh::intersect> is finished in {0} seconds.", m_x_m_start.to(PreciseTime::now()));


        let mxm_res_lst = mxm_res.get_res_list();
        info!("There are {0} pairs of intersecting triangles.", mxm_res_lst.len());

        for (index_a, index_b, res) in mxm_res_lst {
            match res.get_info() {
                InfoTxT::Intersecting => {
                    let segment = res.get_segment();
                    add_segment_to_map(&index_a, segment.clone(), &mut it_to_ss_for_mesh_a);
                    add_segment_to_map(&index_b, segment.clone(), &mut it_to_ss_for_mesh_b);
                }

                InfoTxT::CoplanarIntersecting => {
                    return Err("Meshes should not have planar intersections!");
                }

                _ => {}
            }
        }

        if it_to_ss_for_mesh_a.is_empty() && it_to_ss_for_mesh_b.is_empty() {
            return Err("Meshes doesn't intersect each other!");
        }

        let retr_start = PreciseTime::now();
        info!("Retriangulation is performing ...");

        let re_triangulated_mesh_a = BoolOpResult::re_triangulate_mesh(
            it_to_ss_for_mesh_a.clone(),
            &mesh_a
        );

        let re_triangulated_mesh_b = BoolOpResult::re_triangulate_mesh(
            it_to_ss_for_mesh_b.clone(),
            &mesh_b
        );
        info!("Retriangulation is finished in {0} seconds.", retr_start.to(PreciseTime::now()));

        let build_blocks_start = PreciseTime::now();
        info!("Building blocks ...");
        let blocks = Blocks::new(
            it_to_ss_for_mesh_a,
            &re_triangulated_mesh_a, &re_triangulated_mesh_b,
        );
        info!("Blocks were built in {0} seconds.", build_blocks_start.to(PreciseTime::now()));

        let bool_op_res = BoolOpResult {
            re_triangulated_mesh_a: re_triangulated_mesh_a,
            re_triangulated_mesh_b: re_triangulated_mesh_b,
            blocks: blocks
        };

        info!("<BoolOpResult::new> is finished in {0} seconds.\n", start.to(PreciseTime::now()));
        info!("----------------------------------------");
        return Ok(bool_op_res);
    }

    fn re_triangulate_mesh(
        it_to_ss: HashMap<usize, Vec<Segment>>,
        mesh: &Mesh
    ) -> Mesh {
        let mut new_mesh = mesh.clone();

        for (it, ss) in  it_to_ss {
            let t: Triangle = mesh.get_triangle(it.clone());
            let ts : Vec<Triangle> = triangulate_ptree3d(t, ss);
            new_mesh.remove_triangle(&it);
            new_mesh.add_triangles(ts);
        }

        return new_mesh;
    }

    pub(crate) fn get_intermidiate_meshes(&self) -> (Mesh, Mesh) {
        return (self.re_triangulated_mesh_a.clone(), self.re_triangulated_mesh_b.clone());
    }

    /// This method returns a reference to Vec of meshes, containing (A/B).
    pub fn difference_ab(&self) -> &Vec<Mesh> {
        return self.blocks.get_difference_ab();
    }

    /// This method returns a reference to Vec of meshes, containing (B/A).
    pub fn difference_ba(&self) -> &Vec<Mesh> {
        return self.blocks.get_difference_ba()
    }

    /// This method returns a reference to the mesh, containing (A U B).
    pub fn union(&self) -> &Mesh {
        return self.blocks.get_union();
    }

    /// This method returns a reference to Vec of Meshes, containing (A intersect B).
    pub fn intersection(&self) -> &Vec<Mesh> {
        return self.blocks.get_intersection();
    }
}


#[cfg(test)]
mod tests {
    use primitives::*;
    use bool_op::BoolOpResult;
    use std::fs::File;
    use env_logger::init  as env_logger_init;
    use std::path::Path;
    use std::fs;

    #[derive(Clone)]
    enum BoolOpType {
        Union,
        Intersection,
        DifferenceAB,
        DifferenceBA
    }

    macro_rules! pattern_str {
        (full) =>  ("res_of_tests/robust_bool_op/{0}/{1}.stl");
        (dir) =>  ("res_of_tests/robust_bool_op/{0}");
        (rfull) =>  ("res_of_tests/robust_bool_op/{0}/{1}/{2}.stl");
        (rdir) =>  ("res_of_tests/robust_bool_op/{0}/{1}");
    }

    fn perform_bool_ops<'a>(
        ma: &Mesh, mb: &Mesh,
        operations: Vec<BoolOpType>,
    ) -> Vec<(String, Mesh)> {
        let bool_op_result : BoolOpResult = BoolOpResult::new(&ma, &mb).expect("The error was raised!");

        let mut results : Vec<(String, Mesh)> = Vec::new();

        for op_type in operations {
            match op_type {
                BoolOpType::Union => {
                    results.push(("union".to_string(), bool_op_result.union().clone()));
                },
                BoolOpType::Intersection => {
                    for (i, mesh) in bool_op_result.intersection().iter().enumerate() {
                        results.push((format!("intersection_{0}", i), mesh.clone()));
                    }
                }
                BoolOpType::DifferenceAB => {
                    for (i, mesh) in bool_op_result.difference_ab().iter().enumerate() {
                        results.push((format!("dif_ab_{0}", i), mesh.clone()));
                    }
                },
                BoolOpType::DifferenceBA => {
                    for (i, mesh) in bool_op_result.difference_ba().iter().enumerate() {
                        results.push((format!("dif_ba_{0}", i), mesh.clone()));
                    }
                },

            };
        }

        let (mesh_a, mesh_b) = bool_op_result.get_intermidiate_meshes();
        assert!(mesh_a.geometry_check());
        assert!(mesh_b.geometry_check());


        results.push(("intermidiate_a".to_string(), mesh_a));
        results.push(("intermidiate_b".to_string(), mesh_b));
        return results;
    }

    fn bool_op_test(
        input_file_name_a: &str, input_file_name_b: &str,
        test_index: usize,
        operations: Vec<BoolOpType>,
        do_geom_check_for_results: bool
    ) {
        env_logger_init().unwrap_or_else(|_| ->  () {});

        let mut fa = File::open(input_file_name_a).unwrap();
        let mut fb = File::open(input_file_name_b).unwrap();
        let ma : Mesh = Mesh::read_stl(& mut fa).unwrap();
        let mb : Mesh = Mesh::read_stl(& mut fb).unwrap();

        let results : Vec<(String, Mesh)> = perform_bool_ops(&ma, &mb, operations);
        let dir_path = format!(pattern_str!(dir), test_index);

        if Path::new(&dir_path).exists() {
            fs::remove_dir_all(&dir_path).ok();
        }
        fs::create_dir_all(&dir_path).ok();

        let mut errors = 0;

        for (s, m) in results {
            let output_file_name = format!(pattern_str!(full), test_index, s);
            let mut f = File::create(output_file_name).unwrap();
            match m.write_stl(&mut f) {
                Ok(_) => (),
                Err(_) => panic!("Can not write into file!")
            };



            if do_geom_check_for_results && !m.geometry_check() {
                errors += 1;
            }
        }

        if errors != 0 {
            panic!("Geometry check failed!");
        }

    }

    fn bool_op_test_with_rotation(
        input_file_name_a: &str, input_file_name_b: &str,
        test_index: usize,
        operations: Vec<BoolOpType>,
        do_geom_check_for_results: bool
    ) {
        env_logger_init().unwrap_or_else(|_| ->  () {});

        let mut fa = File::open(input_file_name_a).unwrap();
        let mut fb = File::open(input_file_name_b).unwrap();
        let ma : Mesh = Mesh::read_stl(& mut fa).unwrap();
        let mb : Mesh = Mesh::read_stl(& mut fb).unwrap();

        let main_dir_path = format!(pattern_str!(dir), test_index);
        if Path::new(&main_dir_path).exists() {
            fs::remove_dir_all(&main_dir_path).ok();
        }

        let step : usize = 10;

        for i in 0..10 {
            info!("Mesh b was turned by {0} degrees", i*&step);

            let mut cur_mb = mb.clone();
            cur_mb.rotate_x(Number::new((&step*i) as f64));
            let dir_path = format!(pattern_str!(rdir), test_index, i*&step);
            fs::create_dir_all(&dir_path).ok();

            let file_name = format!(pattern_str!(rfull), test_index, i*&step, "rotated");
            let mut f = File::create(file_name).unwrap();
            match cur_mb.write_stl(&mut f) {
                Ok(_) => (),
                Err(_) => panic!("Can not write into file!")
            };


            let mut results : Vec<(String, Mesh)> =
                perform_bool_ops(&ma, &cur_mb, operations.clone());

            let mut errors = 0;

            for (s, m) in results {
                let output_file_name = format!(pattern_str!(rfull), test_index, i*&step, s);
                let mut f = File::create(output_file_name).unwrap();
                match m.write_stl(&mut f) {
                    Ok(_) => (),
                    Err(_) => panic!("Can not write into file!")
                };

                if do_geom_check_for_results && !m.geometry_check() {
                    errors += 1;
                }
            }

            if errors != 0 {
                panic!("Geometry check failed!");
            }
        }
    }

    #[test]
    fn test1() {
        bool_op_test("input_for_tests/cube_in_origin.stl",
                     "input_for_tests/scaled_shifted_cube.stl",
                     1,
                     vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
                     true);
    }

    #[test]
    fn test2() {
        //cargo test first_union_test -- --nocapture
        bool_op_test("input_for_tests/cube_in_origin.stl",
                     "input_for_tests/long_scaled_shifted_cube.stl",
                     2,
                     vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
                     true);
    }

    #[test]
    fn test3() {
        bool_op_test("input_for_tests/sphere_in_origin.stl",
                     "input_for_tests/long_scaled_shifted_cube.stl",
                     3,
                     vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
                     true);
    }

    #[test]
    fn test4() {
        bool_op_test("input_for_tests/sphere_in_origin.stl",
                     "input_for_tests/cone_in_origin.stl",
                     4,
                     vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
                     true);
    }


    #[test]
    fn test5() {
        bool_op_test("input_for_tests/screw.stl",
                     "input_for_tests/sphere_in_origin.stl",
                     5,
                     vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
                     true);
    }

    #[ignore]
    #[test]
    fn test_rotation_cylinder() {
        bool_op_test_with_rotation(
            "input_for_tests/cylinder_8_a.stl",
            "input_for_tests/cylinder_8_c.stl",
            6,
            vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
            true
        );
    }

    #[ignore]
    #[test]
    fn test_rotation_tor() {
        bool_op_test_with_rotation(
            "input_for_tests/tor.stl",
            "input_for_tests/separating_plane.stl",
            7,
            vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
            true
        );
    }

    #[ignore]
    #[test]
    fn test_of_cylinders_imposition() {
        bool_op_test(
            //"input_for_tests/cylinder_8_a.stl",
            //"input_for_tests/cylinder_8_a_1.stl",
            "input_for_tests/cube_in_origin.stl",
            "input_for_tests/cube_in_origin_2.stl",
            8,
            vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
            true
        );
    }

    #[ignore]
    #[test]
    fn test_of_cube_union() {
        bool_op_test(
            "input_for_tests/cube1.stl",
            "input_for_tests/cube2.stl",
            9,
            vec![BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
            true
        );
    }

    #[test]
    fn test_diplom() {
        bool_op_test(
            // "input_for_tests/tor.stl",
            //"input_for_tests/Leg.stl",
            "input_for_tests/bone1_rot.stl",
            "input_for_tests/bone1.stl",
            10,
            vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
            true
        );
    }

    #[ignore]
    #[test]
    fn test_of_inner_cube() {
        bool_op_test(
            "input_for_tests/cube1.stl",
            "input_for_tests/cube3.stl",
            11,
            vec![BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA, BoolOpType::Intersection],
            true
        );
    }

    #[ignore]
    #[test]
    fn test_diplom2() {
        bool_op_test(
            //"input_for_tests/bone1_rot.stl",
            //"input_for_tests/bone_1_rot_with_hole.stl",
            //"input_for_tests/cube_in_origin.stl",
            //"input_for_tests/cube_with_hole.stl",
            "input_for_tests/cube_with_hole.stl",
            "input_for_tests/cube2.stl",
            12,
            vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
            true
        );
    }

    #[test]
    fn test_chew() {
        bool_op_test(
            "input_for_tests/челюсть.stl",
            //"input_for_tests/implant.stl",
            "input_for_tests/separator.stl",
            14,
            vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
            true
        );
    }

    #[ignore]
    #[test]
    fn test_bone1_bone2() {
        bool_op_test(
            "input_for_tests/bone1.stl",
            "input_for_tests/bone1_part.stl",
            15,
            vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
            true
        );
    }


    #[test]
    fn test_bone_screw() {
        bool_op_test(
            "input_for_tests/bone1.stl",
            "input_for_tests/little_screw.stl",
            16,
            vec![BoolOpType::Intersection, BoolOpType::Union, BoolOpType::DifferenceAB, BoolOpType::DifferenceBA],
            true
        );
    }
}

/*
TODO

Возвращать ошибку если тело не связное или не проходит geometry_check по определенному флагу
Написать алгоритм выполняющий сборку пересекающихся кривых

*/