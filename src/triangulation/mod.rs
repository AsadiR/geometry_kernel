
pub mod incremental_triangulation;
pub mod ear_clipping_triangulation;
pub mod triangulation3d;

pub use self::triangulation3d::{triangulate3d, triangulate_ptree3d};
pub use self::triangulation3d::TriangulationAlgorithm;