
pub mod point_wrapper;

pub mod line_x_line;

pub mod line_x_plane;

pub mod line_x_segment;

pub mod plane_x_plane;

pub mod segment_x_segment;

pub mod triangle_x_triangle;

pub mod mesh_x_mesh;

pub mod tuple_iter;

/*
pub mod segment_x_segment;


pub mod plane_x_plane;
pub mod triangle_x_triangle;
pub mod line_x_segment;
pub mod mesh_x_mesh;

/*
pub import Af (Abstract function), Raf (Realization of abstract function), Info
*/
pub use self::line_x_line::{AfLxL, InfoLxL, RafSimpleLxL};
pub use self::segment_x_segment::{AfSxS, InfoSxS, RafSimpleSxS};
pub use self::line_x_plane::{AfLxP, InfoLxP, RafSimpleLxP};
pub use self::plane_x_plane::{AfPxP, InfoPxP, RafSimplePxP};
pub use self::triangle_x_triangle::{AfTxT, InfoTxT, RafSimpleTxT};
pub use self::line_x_segment::{AfLxS, InfoLxS, RafSimpleLxS};
pub use self::mesh_x_mesh::{AfMxM, MeshIArea, RafSimpleMxM};

*/

