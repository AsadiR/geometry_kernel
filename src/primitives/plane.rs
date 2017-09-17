use primitives::vector::Vector;
use primitives::point::Point;
use primitives::number::Number;
use primitives::zero_trait::Zero;
use std::mem::swap;

// n*(p-p0) = 0

#[derive(Clone, Debug)]
pub struct Plane {
    pub normal: Vector,
    pub point: Point,

    // n*x + d = 0
    pub d: Number
}

impl Plane {
    pub fn new(normal: Vector, point: Point) -> Plane {
        let d = -normal.dot_product(&point.get_vector());
        Plane {
            normal: normal,
            point: point,
            d: d
        }
    }

    pub fn new_3p(p0: &Point, p1: &Point, p2: &Point) -> Plane {
        let v1 = p0 - p1;
        let v2 = p1 - p2;
        let n = v1.cross_product(&v2);
        Plane::new(n, p0.clone())
    }

    pub fn swap_yz(& mut self) {
        self.normal.swap_yz();
        self.point.swap_yz();
    }

    pub fn swap_xy(& mut self) {
        self.normal.swap_xy();
        self.point.swap_xy();
    }

    pub fn swap_xz(& mut self) {
        self.normal.swap_xz();
        self.point.swap_xz();
    }

    pub fn does_it_contain_point(&self, point : &Point) -> bool {
        let dp = self.normal.dot_product(&(point - &self.point));
        //println!("dp: {0}", dp);
        return dp.is_it_zero();
    }


    pub fn get_ref_d(&self) -> &Number{
        &self.d
    }

    pub fn get_ref_normal(&self) -> &Vector {
        &self.normal
    }
}