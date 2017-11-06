use intersect::triangle_x_triangle;
use primitives::*;
use intersect::tuple_iter::{/*TupleIter,*/ enumerate_simple, TreeAABT};
// use log::LogLevel;
use time::PreciseTime;

pub struct IntersectionResult {
    pub res_mxm_list : Vec<(usize, usize, triangle_x_triangle::ResTxT)>
}

impl IntersectionResult {
    pub fn new(res_mxm_list : Vec<(usize, usize, triangle_x_triangle::ResTxT)>) -> IntersectionResult {
        IntersectionResult {
            res_mxm_list : res_mxm_list
        }
    }

    pub fn get_res_list(self) -> Vec<(usize, usize, triangle_x_triangle::ResTxT)> {
        self.res_mxm_list
    }
}


pub fn intersect(a : &Mesh, b : &Mesh, use_tree: usize) -> IntersectionResult {
    info!("<mesh_x_mesh::intersect> was started!");
    let start = PreciseTime::now();

    info!("The enumerating of indexes is performing ...");
    let triangles_enum = if use_tree == 0 {
        enumerate_simple(a, b)
    } else {
        let tree_a = TreeAABT::new(a);
        let tree_b = TreeAABT::new(b);
        TreeAABT::intersect_trees(&tree_a, &tree_b)
    };

    debug!("Number of pairs: {0}", triangles_enum.v.len());


    let mut res_mxm_list : Vec<(usize, usize, triangle_x_triangle::ResTxT)> = Vec::new();


    info!("The triangles are intersecting ...");
    let mut counter = 0;
    for &(index_a, index_b) in triangles_enum.iter() {
        if counter%10000 == 0 && counter != 0 {
            info!("The {0}-th triangle pair is performing. Mesh intersection lasts {1} seconds!", counter, start.to(PreciseTime::now()));
        }

        let tr_a = a.get_triangle(index_a);
        let tr_b = b.get_triangle(index_b);

        let res_txt = triangle_x_triangle::intersect(&tr_a, &tr_b);



        if res_txt.get_info().does_it_intersecting() {
            debug!("-----------");
            debug!("{:?}", tr_a);
            debug!("{:?}", tr_b);
            debug!("info {:?}", res_txt.get_info());
            res_mxm_list.push((index_a, index_b, res_txt));
        }

        counter += 1;
    }
    info!("<mesh_x_mesh::intersect> finished in {0} seconds!", start.to(PreciseTime::now()));
    return IntersectionResult::new(res_mxm_list)
}
