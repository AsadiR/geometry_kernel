use primitives::*;
use std::slice::Iter;

pub struct TupleIter {
    v : Vec<(usize, usize)>,
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
    TupleIter::new(pairs)
}
