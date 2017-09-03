
pub mod number;
pub mod point;
pub mod vector;
pub mod mesh;
pub mod line;
pub mod segment;
pub mod plane;
pub mod triangle;
pub mod polygon;

pub use self::point::Point;
pub use self::number::Number;
pub use self::number::NumberTrait;
pub use self::vector::Vector;
pub use self::segment::Segment;
pub use self::line::Line;
pub use self::plane::Plane;
pub use self::triangle::Triangle;
pub use self::polygon::Polygon;
pub use self::mesh::Mesh;




pub mod zero_trait;
pub use self::zero_trait::Zero;

pub mod signed_trait;
pub use self::signed_trait::Signed;



/*

mod segment;
mod line;
mod curve;
mod plane;
pub mod mesh;
mod base_object;
mod triangle;
mod polygon;

pub use self::base_object::BaseObject;
pub use self::point::Point;
pub use self::point::EClassify;
pub use self::vector::Vector;
pub use self::line::Line;
pub use self::curve::Curve;
pub use self::mesh::{Mesh, read_stl};
pub use self::segment::Segment;
pub use self::plane::Plane;
pub use self::triangle::Triangle;
pub use self::polygon::Polygon;


pub const EPS : f64 = 0.0005;

pub fn eq_f64(a : f64, b : f64) -> bool {
    return (a <= b + EPS) & (a >= b - EPS)
}

*/



