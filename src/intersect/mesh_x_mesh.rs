use intersect::triangle_x_triangle;
use primitives::*;
use intersect::tuple_iter::{TupleIter, enumerate_simple};

pub struct IntersectionResult {
    pub res_mxm_list : Vec<(usize, usize, triangle_x_triangle::ResTxT)>
}

impl IntersectionResult {
    pub fn new(res_mxm_list : Vec<(usize, usize, triangle_x_triangle::ResTxT)>) -> IntersectionResult {
        IntersectionResult {
            res_mxm_list : Vec::new()
        }
    }

    pub fn get_res_list(self) -> Vec<(usize, usize, triangle_x_triangle::ResTxT)> {
        self.res_mxm_list
    }
}


pub fn intersect(a : &Mesh, b : &Mesh) -> IntersectionResult {
    let triangles_enum = enumerate_simple(a, b);
    let mut res_mxm_list : Vec<(usize, usize, triangle_x_triangle::ResTxT)> = Vec::new();
    for &(index_a, index_b) in triangles_enum.iter() {
        let tr_a = a.get_triangle(index_a);
        let tr_b = b.get_triangle(index_b);
        let res_txt = triangle_x_triangle::intersect(&tr_a, &tr_b);
        if res_txt.get_info().does_it_intersecting() {
            res_mxm_list.push((index_a, index_b, res_txt));
        }
    }
    return IntersectionResult::new(res_mxm_list)
}
