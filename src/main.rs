extern crate geometry_kernel;
use geometry_kernel::primitives::mesh::Mesh;
use geometry_kernel::bool_op::BoolOpResult;
use std::fs::File;

extern crate gmp;

fn main() {
    let mut f_a = File::open("input_for_tests/cube_in_origin.stl").unwrap();
    let mesh_a = Mesh::read_stl(&mut f_a).unwrap();

    let mut f_b = File::open("input_for_tests/long_scaled_shifted_cube.stl").unwrap();
    let mesh_b = Mesh::read_stl(&mut f_b).unwrap();

    let result = BoolOpResult::new(&mesh_a, &mesh_b).expect("The error was raised in a constructor of <BoolOpPerformer>!");
    let union_res = result.union();

    let mut f_res= File::create("res_of_tests/robust_bool_op/main/union_res.stl").unwrap();
    match union_res.write_stl(&mut f_res) {
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