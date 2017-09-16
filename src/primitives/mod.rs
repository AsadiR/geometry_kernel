
pub mod number;
pub mod point;
pub mod vector;
pub mod mesh;
pub(crate) mod line;
pub(crate) mod segment;
pub(crate) mod plane;
pub(crate) mod triangle;
pub(crate) mod polygon;

pub(crate) use self::point::Point;
pub(crate) use self::number::Number;
pub(crate) use self::number::NumberTrait;
pub(crate) use self::vector::Vector;
pub(crate) use self::segment::Segment;
pub(crate) use self::line::Line;
pub(crate) use self::plane::Plane;
pub(crate) use self::triangle::Triangle;
pub(crate) use self::polygon::Polygon;
pub(crate) use self::mesh::Mesh;

pub mod zero_trait;
pub(crate) use self::zero_trait::Zero;

pub mod signed_trait;
pub(crate) use self::signed_trait::Signed;




