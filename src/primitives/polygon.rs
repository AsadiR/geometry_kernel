use primitives::point::Point;
use primitives::vector::Vector;
// use primitives::segment::Segment;

#[derive(Clone)]
#[derive(Debug, Hash)]
pub struct Polygon {
    pub points: Vec<Point>,
    pub normal: Vector
}

impl Polygon {
    pub fn new(points : Vec<Point>, normal : Vector) -> Polygon {
        Polygon {
            points: points,
            normal: normal
        }
    }

    pub fn get_points(self) -> Vec<Point> {
        return self.points;
    }

    pub fn get_points_ref(&self) -> &Vec<Point> {
        return &self.points;
    }


    pub fn add_point(&mut self, p : Point) {
        self.points.push(p);
    }
}