extern crate geometry_kernel;
extern crate num;
extern crate bidir_map;
extern crate test;

#[macro_use]
extern crate log;
extern crate env_logger;
use log::LogLevel;

use test::Bencher;


use geometry_kernel::primitives::{mesh, number};
use std::io::Cursor;
use std::fs::File;

use bidir_map::BidirMap;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

fn main() {
    env_logger::init().unwrap();

    // let num = number::new(0.333);
    // println!("value: {}", num);

    let mut f = File::open("input_for_tests/skull.stl").unwrap();
    let mesh = mesh::Mesh::read_stl(&mut f).unwrap();

    let mut f = File::create("res_of_tests/import_export/skull_new.stl").unwrap();
    match mesh.write_stl(&mut f) {
        Ok(_) => (),
        Err(_) => panic!()
    };
}


/*
valgrind --tool=callgrind target/debug/geometry_kernel_main


f64 read_stl skull => debug 266 s | release 12 s

после перехода на HashMap

BigRational read_stl skull => release 11.2 s
*/