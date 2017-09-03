use primitives::vector::Vector;
use primitives::point::Point;
use primitives::Number;
use primitives::zero_trait::Zero;

// n*(p-p0) = 0
pub struct Plane {
    pub normal: Vector,
    pub point: Point,
}

impl Plane {
    pub fn does_plane_contain_point(&self, point : &Point) -> bool {
        self.normal.dot_product(&(point - &self.point)).is_it_zero()
    }

    // n*x + d = 0
    pub fn get_d(&self) -> Number{
        -self.normal.dot_product(&self.point.get_vector())
    }
}