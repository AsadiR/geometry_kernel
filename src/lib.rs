#![crate_name = "geometry_kernel"]


extern crate core;
extern crate time;
extern crate num;
extern crate gmp;
extern crate bidir_map;
extern crate byteorder;
extern crate test;
extern crate rulinalg;


#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate env_logger;

/// This module contains basic geometry classes.
pub mod primitives;
mod intersect;
mod matrix;
mod triangulation;


/// This module contains fucntions to perform boolean operations.
/// # Examples
///
/// ```
/// extern crate geometry_kernel;
/// use geometry_kernel::primitives::mesh::Mesh;
/// use geometry_kernel::bool_op::BoolOpPerformer;
/// use std::fs::File;
///
/// fn main() {
///   let mut f_a = File::open("input_for_tests/cube_in_origin.stl").unwrap();
///   let mesh_a = Mesh::read_stl(&mut f_a).unwrap();
///
///   let mut f_b = File::open("input_for_tests/long_scaled_shifted_cube.stl").unwrap();
///   let mesh_b = Mesh::read_stl(&mut f_b).unwrap();
///
///   let performer = BoolOpPerformer::new(&mesh_a, &mesh_b)
///     .expect("The error was raised in a constructor of <BoolOpPerformer>!");
///   let union_res = performer.union();
///
///   let mut f_res= File::create("res_of_tests/simple_bool_op/main/union_res.stl").unwrap();
///   match union_res.write_stl(&mut f_res) {
///     Ok(_) => (),
///     Err(_) => panic!()
///   };
/// }
/// ```
pub mod bool_op;
