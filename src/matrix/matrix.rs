use std::ops::{Add, Sub, Mul, Div, Neg};
use primitives::zero_trait::Zero;

use std::clone::Clone;
// use std::mem;
use std::fmt::Display;
// use std::iter::Rev;


pub struct Row<T>
    where T:
        Add<Output = T> +
        Sub<Output = T> +
        Mul<Output = T> +
        Div<Output = T> +
        Zero<T> + Clone + Neg<Output = T> +
        PartialOrd + PartialEq +
        Display
{
    values : Vec<T>
}

pub struct Matrix<T>
    where T:
        Add<Output = T> +
        Sub<Output = T> +
        Mul<Output = T> +
        Div<Output = T> +
        Zero<T> + Clone + Neg<Output = T> +
        PartialOrd + PartialEq +
        Display
{
    rows : Vec<Row<T>>
}

impl<T> Row<T>
    where T:
        Add<Output = T> +
        Sub<Output = T> +
        Mul<Output = T> +
        Div<Output = T> +
        Zero<T> + Clone + Neg<Output = T> +
        PartialOrd + PartialEq +
        Display
{
    pub fn new(n : usize) -> Row<T> {
        let mut r = Row {values : Vec::new()};
        for _ in 0..n {
            r.values.push(T::zero());
        }
        return r
    }

    pub fn new_from_vector(v : Vec<T>) -> Row<T> {
        Row {values : v}
    }


    pub fn length(&self) -> usize {
        self.values.len()
    }

    pub fn get(&self, index : &usize) -> T {
        self.values[*index].clone()
    }

    pub fn convert_to_vec(self) -> Vec<T> {
        self.values
    }

    pub fn set(&mut self, i : &usize, value : T) {
        self.values[*i] = value;
    }

    pub fn print(&self) {
        for j in 0..self.length() {
            print!("{0}\t", self.get(&j));
        }
        println!();
    }
}

#[allow(dead_code)]
impl<T> Matrix<T>
    where T:
        Add<Output = T> +
        Sub<Output = T> +
        Mul<Output = T> +
        Div<Output = T> +
        Zero<T> + Clone + Neg<Output = T> +
        PartialOrd + PartialEq +
        Display
{
    pub fn new(n : usize) -> Matrix<T> {
        let mut m = Matrix {rows : Vec::new()};
        for _ in 0..n {
            m.rows.push(Row::new(n));
        }
        return m
    }

    pub fn new_from_vector(rows : Vec<Row<T>>) -> Matrix<T> {
        Matrix {
            rows : rows
        }
    }

    pub fn get(&self, i : &usize, j : &usize) -> T {
        self.rows[*i].values[*j].clone()
    }

    pub fn set(&mut self, i : &usize, j : &usize, value : T) {
        self.rows[*i].values[*j] = value;
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        for i in 0..self.number_of_rows() {
            let ref row = self.rows[i];
            row.print();
        }
        println!();
    }

    pub fn number_of_rows(&self) -> usize {
        self.rows.len()
    }

    pub fn solve(&mut self, mut b : Row<T>) -> Row<T> {
        let n = self.number_of_rows();
        let mut x = Row::new(n);

        //b.print();
        //self.print();


        for main_index in 0..n-1 {
            //println!("Iter: {0}", main_index);
            //self.print();

            let mut main_value = self.get(&main_index, &main_index);
            if main_value.is_it_zero() {
                let mut new_main_index = main_index.clone();
                for i in main_index..n {
                    main_value = self.get(&i, &main_index);
                    //println!("main_value = {0}", main_value);
                    if !main_value.is_it_zero() {
                        new_main_index = i;
                    }
                }

                if new_main_index == main_index {
                    //println!("main_index = {0}", main_index);
                    panic!("Singular matrix!\n");
                } else {
                    self.rows.swap(main_index, new_main_index);
                    b.values.swap(main_index, new_main_index);
                }
            }
            main_value = self.get(&main_index, &main_index);

            //self.print();
            //println!("main_index = {0}, n = {1}", main_index, n);

            for i in main_index+1..n {
                let value_under_main = self.get(&i, &main_index);
                //println!("value_under_main = {0}", value_under_main);
                if value_under_main.is_it_zero() {
                    continue;
                }

                let factor : T = -(value_under_main/main_value.clone());
                //println!("main_value = {0}", main_value);
                //println!("factor = {0}", factor);


                for j in main_index..n {
                    let cur_value_in_main = self.get(&main_index, &j);
                    let cur_value_under_main = self.get(&i, &j);
                    let new_value = cur_value_under_main + cur_value_in_main*factor.clone();
                    self.set(&i, &j, new_value)
                }

                b.values[i] = b.values[i].clone() + b.values[main_index].clone()*factor;

            }

            //self.print();
        }

        //b.print();
        //self.print();


        for i in (0..n).rev() {
            x.set(&i, b.values[i].clone());

            for j in i+1..n {
                let xi = x.get(&i);
                let xj = x.get(&j);
                x.set(&i,  xi - self.get(&i, &j)*xj);
            }
            let xi = x.get(&i)/self.get(&i, &i);
            x.set(&i, xi);
        }

        return x
    }
}


#[cfg(test)]
mod tests {
    use primitives::*;
    use matrix::*;

    #[test]
    fn basic_test() {
        let m : Matrix<f32> = Matrix::new(3);
        assert!(m.number_of_rows() == 3);
    }

    #[test]
    fn solver_test1() {
        let n = 3;
        let mut m : Matrix<f32> = Matrix::new_from_vector(
            vec![
                Row::new_from_vector(vec![0., 1., 2.]),
                Row::new_from_vector(vec![1., 1., 0.]),
                Row::new_from_vector(vec![2., 0., 1.])
            ]
        );

        let b : Row<f32> = Row::new_from_vector(vec![3., 2., 3.]);

        let x  = m.solve(b);

        for i in 0..n {
            let v = x.get(&i);
            // println!("x[{0}] = {1}", i, v);
            assert!(v == 1.);

        }

    }

    #[test]
    fn solver_test2() {
        // let n = 3;
        let mut m : Matrix<f32> = Matrix::new_from_vector(
            vec![
                Row::new_from_vector(vec![0., 1., 2.]),
                Row::new_from_vector(vec![0., 0., 1.]),
                Row::new_from_vector(vec![2., 0., 1.])
            ]
        );

        let b : Row<f32> = Row::new_from_vector(vec![4., 1., 7.]);

        let x  = m.solve(b);


        /*
        for i in 0..n {
            println!("x[{0}] = {1}", i, x.get(&i));
        }
        */

        assert!(x.values == vec![3., 2., 1.]);

    }


    #[test]
    fn solver_test3() {
        // let n = 3;
        let mut m : Matrix<f32> = Matrix::new_from_vector(
            vec![
                Row::new_from_vector(vec![1., 2., 3.]),
                Row::new_from_vector(vec![1., 3., 3.]),
                Row::new_from_vector(vec![0., 1., 1.])
            ]
        );

        let b : Row<f32> = Row::new_from_vector(vec![9., 11., 3.]);

        let x  = m.solve(b);

        /*
        for i in 0..n {
            // println!("x[{0}] = {1}", i, x.get(&i));
        }
        */

        assert!(x.values == vec![2., 2., 1.]);

    }


    #[test]
    fn solver_test_rational1() {
        // let n = 3;
        let mut m : Matrix<Number> = Matrix::new_from_vector(
            vec![
                Row::new_from_vector(vec![Number::new_from_f32(1.), Number::new_from_f32(2.), Number::new_from_f32(3.)]),
                Row::new_from_vector(vec![Number::new_from_f32(1.), Number::new_from_f32(3.), Number::new_from_f32(3.)]),
                Row::new_from_vector(vec![Number::new_from_f32(0.), Number::new_from_f32(1.), Number::new_from_f32(1.)])
            ]
        );

        let b : Row<Number> = Row::new_from_vector(vec![Number::new_from_f32(9.), Number::new_from_f32(11.), Number::new_from_f32(3.)]);

        let x  = m.solve(b);

        /*
        for i in 0..n {
            // println!("x[{0}] = {1}", i, x.get(&i));
        }
        */

        assert!(x.values == vec![Number::new_from_f32(2.), Number::new_from_f32(2.), Number::new_from_f32(1.)]);

    }
}