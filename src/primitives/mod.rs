
pub(crate) mod number_trait;
pub(crate) mod number_impl_big_rational;
pub(crate) mod number_impl_gmp;

pub mod number;
pub mod point;
pub mod vector;
pub mod mesh;
pub mod triangle;
pub(crate) mod line;
pub(crate) mod segment;
pub(crate) mod plane;
pub(crate) mod polygon;

pub(crate) use self::point::Point;
pub(crate) use self::number::*;
pub(crate) use self::vector::Vector;
pub(crate) use self::segment::Segment;
pub(crate) use self::line::Line;
pub(crate) use self::plane::Plane;
pub(crate) use self::triangle::Triangle;
pub(crate) use self::polygon::Polygon;
pub(crate) use self::mesh::Mesh;

pub(crate) mod zero_trait;
pub(crate) use self::zero_trait::Zero;

pub(crate) mod signed_trait;
pub(crate) use self::signed_trait::Signed;




