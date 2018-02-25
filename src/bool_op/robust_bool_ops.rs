use primitives::*;
use intersect::mesh_x_mesh;
use intersect::triangle_x_triangle::InfoTxT;
use triangulation::*;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeMap;
use std::collections::hash_set;

use log::LogLevel;
use time::PreciseTime;
use std::result::Result;


pub struct BoolOpResult {
    re_triangulated_mesh_a : Mesh,
    re_triangulated_mesh_b : Mesh,
    subsurface_tree: SubsurfaceTree
}

pub struct SubsurfaceTree {}
impl SubsurfaceTree {
    pub fn new() -> SubsurfaceTree {
        // panic!("Not implemented!");
        return SubsurfaceTree {};
    }
}

impl BoolOpResult {
    pub fn new(mesh_a_ref: &Mesh, mesh_b_ref: &Mesh) -> Result<BoolOpResult, &'static str> {
        let start = PreciseTime::now();

        if !mesh_a_ref.geometry_check()  {
            return Err("Geometry check failed for first mesh! Each triangle must have three adjacent triangles!");
        } else if !mesh_b_ref.geometry_check() {
            return Err("Geometry check failed for second mesh! Each triangle must have three adjacent triangles!");
        }

        if log_enabled!(LogLevel::Info) {
            info!("----------------------------------------");
            info!("<BoolOpResult::calculate> is performing ...\n");
        }

        let mut mesh_a : Mesh = mesh_a_ref.clone();
        let mut mesh_b : Mesh = mesh_b_ref.clone();

        let m_x_m_start = PreciseTime::now();
        info!("Intersection of meshes is performing ...");
        let mxm_res = mesh_x_mesh::intersect(&mesh_a, &mesh_b, 1);
        info!("<mesh_x_mesh::intersect> is finished in {0} seconds.", m_x_m_start.to(PreciseTime::now()));

        let mxm_res_lst = mxm_res.get_res_list();
        let mut it_to_ss_for_mesh_a: HashMap<usize, Vec<Segment>> = HashMap::new();
        let mut it_to_ss_for_mesh_b: HashMap<usize, Vec<Segment>> = HashMap::new();

        fn add_segment_to_map(it: &usize, s: Segment, t_to_ss: &mut HashMap<usize, Vec<Segment>>) {
            if t_to_ss.contains_key(it) {
                // повторяться отрезки не могут, так как иначе присутствует самопересечение
                t_to_ss.get_mut(it).unwrap().push(s);
            } else {
                t_to_ss.insert(it.clone(), vec![s]);
            }
        }

        for (index_a, index_b, res) in  mxm_res_lst{
            match res.get_info() {
                InfoTxT::Intersecting => {
                    let segment = res.get_segment();
                    add_segment_to_map(&index_a, segment.clone(), &mut it_to_ss_for_mesh_a);
                    add_segment_to_map(&index_b, segment, &mut it_to_ss_for_mesh_b);
                }
                // TODO InfoTxT::CoplanarIntersecting => {}
                _ => {}

            }
        }

        let re_triangulated_mesh_a = BoolOpResult::re_triangulate_mesh(it_to_ss_for_mesh_a, &mesh_a);
        let re_triangulated_mesh_b = BoolOpResult::re_triangulate_mesh(it_to_ss_for_mesh_b, &mesh_b);

        // TODO build sub-surfaces
        // TODO build sub-blocks
        let subsurface_tree = SubsurfaceTree::new();
        let bool_op_res = BoolOpResult {
            re_triangulated_mesh_a: re_triangulated_mesh_a,
            re_triangulated_mesh_b: re_triangulated_mesh_b,
            subsurface_tree: subsurface_tree
        };

        info!("<BoolOpResult::calculate> is finished in {0} seconds.\n", start.to(PreciseTime::now()));
        info!("----------------------------------------");
        return Ok(bool_op_res);
    }

    fn re_triangulate_mesh(it_to_ss: HashMap<usize, Vec<Segment>>, mesh: &Mesh) -> Mesh {
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

    // TODO distinguishing

    pub fn difference_ab(&self) -> Mesh {
        panic!("Not implemented");
    }

    pub fn difference_ba(&self) -> Mesh {
        panic!("Not implemented");
    }

    pub fn union(&self) -> Mesh {
        panic!("Not implemented");
    }

    pub fn intersection(&self) -> Mesh {
        panic!("Not implemented");
    }
}


/*
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


*/

/*
TODO плоскостные наложения в терминах subsurface-ов
*/


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
    ) -> Vec<(&'a str, Mesh)> {
        let bool_op_result : BoolOpResult = BoolOpResult::new(&ma, &mb).expect("The error was raised!");

        let mut results : Vec<(&'a str, Mesh)> = Vec::new();

        for op_type in operations {
            let (s, m) = match op_type {
                BoolOpType::Union => ("union", bool_op_result.union()),
                BoolOpType::DifferenceAB => ("diference_ab", bool_op_result.difference_ab()),
                BoolOpType::DifferenceBA => ("diference_ba", bool_op_result.difference_ba()),
                BoolOpType::Intersection => ("intersection", bool_op_result.intersection())
            };

            results.push((s, m));
        }

        let (mesh_a, mesh_b) = bool_op_result.get_intermidiate_meshes();
        assert!(mesh_a.geometry_check());
        assert!(mesh_b.geometry_check());


        results.push(("intermidiate_a", mesh_a));
        results.push(("intermidiate_b", mesh_b));
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

        let results : Vec<(&str, Mesh)> = perform_bool_ops(&ma, &mb, operations);
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


            let mut results : Vec<(&str, Mesh)> =
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
                     1, vec![],
                     true);
    }

    #[test]
    fn test2() {
        //cargo test first_union_test -- --nocapture
        bool_op_test("input_for_tests/cube_in_origin.stl",
                     "input_for_tests/long_scaled_shifted_cube.stl",
                     2, vec![],
                     true);
    }

    #[test]
    fn test4() {
        bool_op_test("input_for_tests/sphere_in_origin.stl",
                     "input_for_tests/cone_in_origin.stl",
                     4, vec![],
                     true);
    }

    #[test]
    fn test13() {
        bool_op_test("input_for_tests/screw.stl",
                     "input_for_tests/sphere_in_origin.stl",
                     13, vec![],
                     true);
    }
}

/*
TODO:
1. Если поверхность содержит самопересечения, то нужно возвращать ошибку. [1]
    1.1. Так как geometry check будет выполняться при создании BoolOpPerformer-а, то его нужно убрать из тестов.
    1.2. Убрать из unit-тестов все что противоречит данной концепции.
2. Если результирующая поверхность содержит несколько компонент связности, то нужно возвращать список mesh-ей [2]
    2.1. Нужно внутри BoolOpPerformer-а сделать метод для поиска компонент связности.
         Это делается с помощью нерекурсивного обхода графа.
    2.2. Написать тест с разбиением фигуры на три части.
3. Нужно подготовить тест с вращающимися цилиндрами.
    3.1. Создать в blender цилиндр с небольшим количеством полигонов.
    3.2. Написать метод выполняющий рациональное вращение с помощью ряда тейлора.
    3.3. Написать тест, который запускает булевы операции для оригинального и повернутого цилиндра в цикле.
    3.4. Нужно написать алгоритм, проверяющий точку на принадлежность внутренности поверхности.
         Это делается с помощью пересечения луча с поверхностью. Если найдено четное кошличество точек,
         то точка находится вне поверхности, иначе она находится внутри.
         Пересекать луч с поверхностью нужно, используя AABT.
         Луч - задется точкой и направляющим вектором. Нужно уметь пересекать луч с боксом.

    3.5. Нужно написать алгоритм, который методом Монте-Карло проверяет корректность результата.
    3.6. Во все тесты булевых операций нужно добавить проверка данным методом.
4. Исправить статью 2, и отправить ее дяде Виталию.
5. Подготовить план-содержание для записки дисертации.

TODO important:
1. Запретить касание ребром. Этот случай приводит к генерации поверхности с самопересечением и как следствие к поломке алгоритма.
2. Придумать алгоритм, который будет вместо триангуляции будет делать разбиение треугольников по ребрам.

*/