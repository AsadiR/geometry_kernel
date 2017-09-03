use primitives::*;
use std::collections::BTreeSet;
use std::collections::btree_set::Iter;

pub struct TupleIter {
    v : BTreeSet<(usize, usize)>,
    index : usize,
}

impl TupleIter {
    pub fn new(v : BTreeSet<(usize, usize)>) -> TupleIter {
        TupleIter {v: v, index: 0}
    }

    pub fn iter(&self) -> Iter<(usize, usize)>
    {
        return self.v.iter();
    }
}


pub fn enumerate_simple(a : &Mesh, b : &Mesh) -> TupleIter {
    let mut pairs : BTreeSet<(usize, usize)> = BTreeSet::new();

    for index_a in 0..a.triangles.len() {
        for index_b in 0..b.triangles.len() {
            if !pairs.contains(&(index_b, index_a)) {
                pairs.insert((index_a, index_b));
            }
        }
    }
    TupleIter::new(pairs)
}
