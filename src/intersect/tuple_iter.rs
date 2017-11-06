use primitives::*;
use std::slice::Iter;
use std::collections::HashMap;
use std::cmp::{max, min};
use std::collections::BTreeSet;
use std::fmt;

#[allow(dead_code)]
pub struct TupleIter {
    pub(crate) v : Vec<(usize, usize)>,
    index : usize,
}

impl TupleIter {
    pub fn new(v : Vec<(usize, usize)>) -> TupleIter {
        TupleIter {v: v, index: 0}
    }

    pub fn iter(&self) -> Iter<(usize, usize)>
    {
        return self.v.iter();
    }
}


pub fn enumerate_simple(a : &Mesh, b : &Mesh) -> TupleIter {
    let mut pairs : Vec<(usize, usize)> = Vec::new();

    for index_a in 0..a.num_of_triangles() {
        for index_b in 0..b.num_of_triangles() {
            pairs.push((index_a, index_b));
        }
    }


    return TupleIter::new(pairs);
}

// оболочка
#[derive(Clone)]
#[derive(Debug)]
struct AAB {
    x_min : Number,
    x_max : Number,
    y_min : Number,
    y_max : Number,
    z_min : Number,
    z_max : Number
}

impl fmt::Display for AAB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x_min: {}, x_max: {}, y_min: {}, y_max: {}, z_min: {}, z_max: {}",
               self.x_min.clone().convert_to_f32(),
               self.x_max.clone().convert_to_f32(),
               self.y_min.clone().convert_to_f32(),
               self.y_max.clone().convert_to_f32(),
               self.z_min.clone().convert_to_f32(),
               self.z_max.clone().convert_to_f32())
    }
}

impl AAB {
    pub fn new(
        x_min : Number,
        x_max : Number,
        y_min : Number,
        y_max : Number,
        z_min : Number,
        z_max : Number
    ) -> AAB {
        AAB {
            x_min,
            x_max,
            y_min,
            y_max,
            z_min,
            z_max
        }
    }

    pub fn union_boxes(box1: &AAB, box2: &AAB) -> AAB {
        AAB {
            x_min: min(box1.x_min.clone(), box2.x_min.clone()),
            x_max: max(box1.x_max.clone(), box2.x_max.clone()),
            y_min: min(box1.y_min.clone(), box2.y_min.clone()),
            y_max: max(box1.y_max.clone(), box2.y_max.clone()),
            z_min: min(box1.z_min.clone(), box2.z_min.clone()),
            z_max: max(box1.z_max.clone(), box2.z_max.clone())
        }
    }

    fn overlay(min1: &Number, max1: &Number, min2: &Number, max2: &Number) -> bool {
        return (min1 >= min2) && (min1 <= max2) || (max1 >= min2) && (max1 <= max2) ||
            (min2 >= min1) && (min2 <= max1) || (max2 >= min1) && (max2 <= max1);
    }

    pub fn do_boxes_intersect(box1: &AAB, box2: &AAB) -> bool {
        return AAB::overlay(&box1.x_min, &box1.x_max, &box2.x_min, &box2.x_max) &&
               AAB::overlay(&box1.y_min, &box1.y_max, &box2.y_min, &box2.y_max) &&
               AAB::overlay(&box1.z_min, &box1.z_max, &box2.z_min, &box2.z_max);
    }

    pub fn wrap_triangle(t : Triangle) -> AAB {
        let mut ps = t.get_points();
        let first_p = ps.pop().unwrap();

        let mut x_min : Number = first_p.x.clone();
        let mut x_max : Number = first_p.x.clone();
        let mut y_min : Number = first_p.y.clone();
        let mut y_max : Number = first_p.y.clone();
        let mut z_min : Number = first_p.z.clone();
        let mut z_max : Number = first_p.z.clone();
        for t in ps {
            if t.x < x_min {
                x_min = t.x.clone();
            }

            if t.x > x_max {
                x_max = t.x;
            }

            if t.y < y_min {
                y_min = t.y.clone();
            }

            if t.y > y_max {
                y_max = t.y;
            }

            if t.z < z_min {
                z_min = t.z.clone();
            }

            if t.z > z_max {
                z_max = t.z;
            }
        }

        return AAB::new(x_min, x_max, y_min, y_max, z_min, z_max);
    }
}


// структура для хранения топологии оболочек, используемая для построения AAT снизу-вверх
struct LayerAABT {
    // горизонтальные соседи
    neighbours: HashMap<usize, BTreeSet<usize>>,
    layer_level: usize
}

impl LayerAABT {
    pub fn new(mesh : &Mesh, tree: &mut TreeAABT) -> LayerAABT {
        let mut neighbours: HashMap<usize, BTreeSet<usize>> = HashMap::new();

        let (it_to_t, mut it_to_ns) = mesh.get_triangles_and_neighbours();
        for (index, t) in it_to_t.into_iter() {
            tree.boxes.insert(index, AAB::wrap_triangle(t));
            if index > tree.max_index {
                tree.max_index = index;
            }
            neighbours.insert(index, it_to_ns.remove(&index).unwrap());
        }

        debug!("Number of elements on lvl 0 is {}", neighbours.len());
        return LayerAABT {neighbours, layer_level: 0};
    }

    pub fn create_next_layer(&self, tree: &mut TreeAABT) -> LayerAABT {
        //println!("Layer {0} is creating!", self.layer_level + 1);

        // соседи следующего уровня! не нужно путать с соседями текущего!
        let mut neighbours: HashMap<usize, BTreeSet<usize>> = HashMap::new();

        let mut stack: Vec<usize>  = Vec::new();

        // кладем индекс случайного бокса в стэк
        stack.push(self.neighbours.keys().next().unwrap().clone());

        let mut number_of_performed_elements = 0;

        while !stack.is_empty() /*|| number_of_performed_elements < self.neighbours.len()*/ {
            /*
            let cur_box_index = if !stack.is_empty() {
                stack.pop().unwrap()
            } else {
                // ищем необработанный элемент
                for index in self.neighbours.keys() {
                    if !tree.index_to_parent.contains_key(index) {
                        stack.push(*index);
                        break;
                    }
                }
                stack.pop().unwrap()
            };
            */
            let cur_box_index = stack.pop().unwrap();

            if tree.index_to_parent.contains_key(&cur_box_index) {
                continue;
            }


            let ns = self.neighbours.get(&cur_box_index).unwrap();
            let mut opt_first_nbox_index : Option<usize> = None;
            let mut opt_parent_index : Option<usize> = None;

            // ищем первого соседа без парента
            for nbox_index in ns.iter() {
                if !tree.index_to_parent.contains_key(nbox_index) {
                    opt_first_nbox_index = Some(*nbox_index);
                    tree.max_index += 1;
                    let parent_index = tree.max_index;
                    opt_parent_index = Some(parent_index);

                    let parent_box: AAB;
                    {
                        let first_nbox = tree.boxes.get(nbox_index).unwrap();
                        let cur_box = tree.boxes.get(&cur_box_index).unwrap();
                        parent_box = AAB::union_boxes(first_nbox, cur_box);
                    }

                    tree.boxes.insert(parent_index, parent_box);
                    neighbours.insert(parent_index, BTreeSet::new());

                    tree.successors.insert(parent_index, vec![*nbox_index, cur_box_index]);
                    tree.index_to_parent.insert(*nbox_index, parent_index);
                    tree.index_to_parent.insert(cur_box_index, parent_index);
                    break;
                }
            }

            let parent_index = if opt_parent_index.is_some() {
                number_of_performed_elements += 2;
                opt_parent_index.unwrap()
            } else {
                // в случае если соседа для объединения не было найдено выполняются следующие действия

                number_of_performed_elements += 1;

                tree.max_index += 1;
                let index = tree.max_index;
                let new_box = tree.boxes.get(&cur_box_index).unwrap().clone();
                tree.boxes.insert(index, new_box);
                tree.index_to_parent.insert(cur_box_index, index);
                neighbours.insert(index, BTreeSet::new());
                tree.successors.insert(index, vec![cur_box_index]);
                index
            };

            let mut ns_set: BTreeSet<usize> = BTreeSet::new();
            ns_set.extend(ns);
            if opt_first_nbox_index.is_some() {
                let first_nbox_index = opt_first_nbox_index.unwrap();
                let first_nbox_ns = self.neighbours.get(&first_nbox_index).unwrap();
                ns_set.extend(first_nbox_ns);
                ns_set.remove(&first_nbox_index);
                ns_set.remove(&cur_box_index);
            }

            for nbox_index in ns_set {
               if !tree.index_to_parent.contains_key(&nbox_index) {
                   stack.push(nbox_index);
               }  else {
                   let nbox_parent_index = tree.index_to_parent.get(&nbox_index).unwrap();
                   neighbours.get_mut(nbox_parent_index).unwrap().insert(parent_index);
                   neighbours.get_mut(&parent_index).unwrap().insert(*nbox_parent_index);
               }
            }

        }

        debug!("Number of performed elements is {}\n", number_of_performed_elements);
        debug!("Number of elements on lvl {} is {}", self.layer_level + 1, neighbours.len());
        return LayerAABT {neighbours, layer_level: self.layer_level + 1};
    }

    pub fn get_number_of_boxes_in_layer(&self) -> usize {
        return self.neighbours.len();
    }

    pub fn get_root_index(&self) -> usize {
        if self.neighbours.len() != 1 {
            panic!("Root cannot be extracted!");
        } else {
            return self.neighbours.keys().next().unwrap().clone();
        }
    }
}

// дерево выровненых по осям параллепипедов
pub(crate) struct TreeAABT {
    boxes: HashMap<usize, AAB>,
    // вертикальные соседи
    successors: HashMap<usize, Vec<usize>>,
    index_to_parent: HashMap<usize, usize>,
    max_index: usize,
    root_index: usize
}

impl TreeAABT {

    // дерево строится так, что боксы имеют индексы такие же как и треугольники в полигональных сетках!
    pub fn new(mesh : &Mesh) -> TreeAABT {
        let boxes: HashMap<usize, AAB> = HashMap::new();
        let successors: HashMap<usize, Vec<usize>> = HashMap::new();
        let parents: HashMap<usize, usize> = HashMap::new();
        let mut tree = TreeAABT {
            boxes, successors,
            index_to_parent: parents,
            max_index: 0,
            root_index: 0
        };

        let mut layer = LayerAABT::new(mesh, &mut tree);

        while layer.get_number_of_boxes_in_layer() > 1 {
            layer = layer.create_next_layer(&mut tree);
        }
        tree.root_index = layer.get_root_index();

        debug!("Tree height: {0}\n", layer.layer_level);
        return tree;
    }

    pub fn intersect_trees(tree_a : &TreeAABT, tree_b : &TreeAABT) -> TupleIter {
        let mut pairs : Vec<(usize, usize)> = Vec::new();
        let mut stack: Vec<(usize, usize)> = Vec::new();
        stack.push((tree_a.root_index, tree_b.root_index));


        while !stack.is_empty() {
            let (index_a, index_b) = stack.pop().unwrap();
            let box1: &AAB = tree_a.boxes.get(&index_a).unwrap();
            let box2: &AAB = tree_b.boxes.get(&index_b).unwrap();

            //println!("box1 {}", box1);
            //println!("box2 {}", box2);

            if AAB::do_boxes_intersect(box1, box2) {
                if !tree_a.successors.contains_key(&index_a) && !tree_b.successors.contains_key(&index_b) {
                    pairs.push((index_a, index_b));
                } else if !tree_a.successors.contains_key(&index_a) {
                    for suc_index_b in tree_b.successors.get(&index_b).unwrap().iter() {
                        stack.push((index_a, *suc_index_b));
                    }
                } else if !tree_b.successors.contains_key(&index_b) {
                    for suc_index_a in tree_a.successors.get(&index_a).unwrap().iter() {
                        stack.push((*suc_index_a, index_b));
                    }
                } else {
                    for suc_index_a in tree_a.successors.get(&index_a).unwrap().iter() {
                        for suc_index_b in tree_b.successors.get(&index_b).unwrap().iter() {
                            stack.push((*suc_index_a, *suc_index_b));
                        }
                    }
                }
            } else {
                //println!("dont intersect");
            }
        }

        return TupleIter::new(pairs);
    }
}

#[cfg(test)]
mod test {
    // use std;
    // use std::io::Cursor;
    use std::fs::File;

    // use bidir_map::BidirMap;
    // use std::collections::BTreeMap;
    // use std::collections::BTreeSet;

    // use primitives::point;
    use primitives::mesh;
    // use primitives::triangle::Triangle;
    use intersect::tuple_iter::{TreeAABT, enumerate_simple};
    use intersect::mesh_x_mesh;

    use time::PreciseTime;

    #[ignore]
    #[test]
    fn first_tree_test() {
        let path_a = "input_for_tests/skull.stl";
        let path_b = "input_for_tests/sphere_in_origin.stl";

        let mut file_a = File::open(path_a).unwrap();
        let mesh_a = mesh::Mesh::read_stl(&mut file_a).unwrap();
        let tree_a = TreeAABT::new(&mesh_a);

        let mut file_b = File::open(path_b).unwrap();
        let mesh_b = mesh::Mesh::read_stl(&mut file_b).unwrap();
        let tree_b = TreeAABT::new(&mesh_b);

        let tuple_iter = TreeAABT::intersect_trees(&tree_a, &tree_b);
        debug!("optimized number of pairs: {0}", tuple_iter.v.len());

        let common_tupple_iter = enumerate_simple(&mesh_a, &mesh_b);
        debug!("common number of pairs: {0}", common_tupple_iter.v.len());
    }

    #[ignore]
    #[test]
    fn intersection_test() {
        //let path_a = "input_for_tests/skull.stl";

        let path_a = "input_for_tests/screw.stl";
        let path_b = "input_for_tests/sphere_in_origin.stl";

        //let path_a = "input_for_tests/sphere_in_origin.stl";
        //let path_b = "input_for_tests/cone_in_origin.stl";

        let mut file_a = File::open(path_a).unwrap();
        let mesh_a = mesh::Mesh::read_stl(&mut file_a).unwrap();

        let mut file_b = File::open(path_b).unwrap();
        let mesh_b = mesh::Mesh::read_stl(&mut file_b).unwrap();


        let start_tree = PreciseTime::now();
        debug!("Intersecting using tree ...");
        let tree_res = mesh_x_mesh::intersect(&mesh_a, &mesh_b, 1);
        debug!("Finished in {0}", start_tree.to(PreciseTime::now()));
        debug!("Res size: {0}", tree_res.res_mxm_list.len());

        debug!("");

        let start_common = PreciseTime::now();
        debug!("Intersecting using common mode ...");
        let common_res = mesh_x_mesh::intersect(&mesh_a, &mesh_b, 0);
        debug!("Finished in {0}", start_common.to(PreciseTime::now()));
        debug!("Res size: {0}", common_res.res_mxm_list.len());

        assert!(tree_res.res_mxm_list.len() == common_res.res_mxm_list.len());
    }
}