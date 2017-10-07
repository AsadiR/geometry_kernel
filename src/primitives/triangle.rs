use primitives::point::Point;
use primitives::plane::Plane;
use primitives::vector::Vector;
use primitives::number::*;
use primitives::zero_trait::Zero;
use primitives::signed_trait::Signed;
use primitives::segment::Segment;
use std::collections::BTreeSet;

/// This structure represents a triangle in 3D space.
#[derive(Debug, Hash)]
pub struct Triangle {
    points : Vec<Point>,
    normal : Option<Vector>
}

impl PartialEq for Triangle {
    fn eq(&self, other: &Triangle) -> bool {
        (self.points[0] == other.points[0]) & (self.points[1] == other.points[1]) & (self.points[2] == other.points[2])
    }
}

impl Eq for Triangle {}

impl Triangle {
    /// This method creates `Triangle` from a `Vec` of points and calculates a normal using `points`.
    /// # Arguments
    ///
    /// * `points` - A `Vec<Point>` to create the triangle.
    pub fn new(points : Vec<Point>) -> Triangle {
        let mut t  = Triangle {
            points : points,
            normal : None
        };
        let v1 = t.get_ref(0) - t.get_ref(1);
        let v2 = t.get_ref(1) - t.get_ref(2);
        let n = v1.cross_product(&v2);
        t.normal = Some(n);
        return t;
    }

    /// This method creates `Triangle` from a `Vec` of points.
    /// # Arguments
    ///
    /// * `points` - A `Vec<Point>' to create the triangle.
    /// * `normal` - A normal vector.
    pub fn new_with_normal(points : Vec<Point>, normal : Vector) -> Triangle {
        let mut t  = Triangle {
            points : points,
            normal : Some(normal)
        };
        return t;
    }

    /// This method returns the reference to the `Vec<Point>`, containing the triangle points.
    pub fn get_points_ref(&self) -> &Vec<Point> {
        return &self.points;
    }

    /// This method returns the reference to the `Vec<Point>`, containing the triangle points.
    pub fn get_points(self) -> Vec<Point> {
        return self.points;
    }

    /// This method returns the copy of the point, specified by `index`.
    /// # Arguments
    ///
    /// * `index` - An index of the point. It has to be less than 3.
    pub fn get(&self, index : usize) -> Point {
        self.points[index].clone()
    }

    /// This method returns the reference to the point, specified by `index`.
    /// # Arguments
    ///
    /// * `index` - An index of the point. It has to be less than 3.
    pub fn get_ref(&self, index : usize) -> &Point {
        &self.points[index]
    }

    pub(crate) fn gen_plane(&self) -> Plane {
        let mut p = Plane::new(
            // check it!
            (self.get_ref(0) - self.get_ref(1)).cross_product(&(self.get_ref(1) - self.get_ref(2))),
            self.get_ref(0).clone()
        );
        // p.normal.normalize();
        return p;
    }

    /// This method returns a copy of the normal `Vector`.
    pub fn get_normal(&self) -> Vector {
        if self.normal.is_some() {
            return self.normal.clone().unwrap();
        } else {
            panic!("Something goes wrong!");
        }
    }

    pub(crate) fn reverse(&mut self) {
        self.points.swap(0, 1);
        self.normal = Some(self.normal.clone().unwrap() * Number::new(-1.));
    }

    pub(crate) fn does_triangle_contain_point(&self, p : &Point) -> bool {
        // http://blackpawn.com/texts/pointinpoly/

        let v0 = self.get_ref(2) - self.get_ref(0);
        let v1 = self.get_ref(1) - self.get_ref(0);
        let v2 = p - self.get_ref(0);

        /*
        println!("p = {:?}", p);
        println!("p0 = {:?}", self.get_ref(0));
        println!("p1 = {:?}", self.get_ref(1));
        println!("p2 = {:?}", self.get_ref(2));
        println!("v0 = {:?}", v0);
        println!("v1 = {:?}", v1);
        println!("v2 = {:?}", v2);
        */

        // Compute dot products
        let dot00 = v0.dot_product(&v0);
        let dot01 = v0.dot_product(&v1);
        let dot02 = v0.dot_product(&v2);
        let dot11 = v1.dot_product(&v1);
        let dot12 = v1.dot_product(&v2);

        // Compute barycentric coordinates
        let inv_denom = Number::new(1.) / (&dot00 * &dot11 - &dot01 * &dot01);
        let u = (&dot11 * &dot02 - &dot01 * &dot12) * &inv_denom;
        let v = (&dot00 * &dot12 - &dot01 * &dot02) * &inv_denom;

        //println!("u = {0}", u);
        //println!("v = {0}", v);

        // Check if point is in triangle
        return (u.is_it_positive() || u.is_it_zero()) &&
            (v.is_it_positive() || v.is_it_zero()) &&
            (u + v <= Number::new(1.))
    }

    /// This method checks the triangle and returns:
    /// 0 - if it's common triangle (there are zero coincident points).
    /// 1 - if it's a segment (there are two coincident points).
    /// 2 - is it's a point (there are three coincident points).
    pub fn degradation_level(&self) -> u64 {
        if self.points[0] == self.points[1] && self.points[1] == self.points[2] {
            return 2;
        }

        let cp = (&self.points[0] - &self.points[1]).cross_product(&(&self.points[2] - &self.points[1]));
        if cp.is_zero() {
            return 1;
        }

        return 0;
    }


    pub(crate) fn get_sides(&self) -> Vec<Segment> {
        let s1 = Segment::new(self.get(0), self.get(1));
        let s2 = Segment::new(self.get(1), self.get(2));
        let s3 = Segment::new(self.get(2), self.get(0));
        let tr1_segments : Vec<Segment> = vec![s1, s2, s3];
        return tr1_segments;
    }
}


#[cfg(test)]
mod tests {
    use primitives::*;
    use intersect::*;

    #[test]
    fn triangle_contains_point() {
        let p1 = Point::new_from_f64(1., 0., 0.);
        let p2 = Point::new_from_f64(0., 1., 0.);
        let p3 = Point::new_from_f64(1., 1., 0.);
        let tr1 = Triangle::new(vec![p1, p2, p3]);

        let p = Point::new_from_f64(1. / 2., 1. / 2., 0.);
        assert!(tr1.does_triangle_contain_point(&p));
    }

    #[test]
    fn triangle_does_not_contain_point() {
        let p1 = Point::new_from_f64(1., 0., 0.);
        let p2 = Point::new_from_f64(0., 1., 0.);
        let p3 = Point::new_from_f64(1., 1., 0.);
        let tr1 = Triangle::new(vec![p1, p2, p3]);

        let p = Point::new_from_f64(1. / 2., 3. , 0.);
        assert!(!tr1.does_triangle_contain_point(&p));
    }

    #[test]
    fn triangle_contains_test() {
        let p1 = Point::new_from_f64(0., 0., 0.);
        let p2 = Point::new_from_f64(1., 0., 0.);
        let p3 = Point::new_from_f64(0., 1., 0.);
        let tr1 = Triangle::new(vec![p1,p2,p3]);

        let p1 = Point::new_from_f64(0., -1., 0.);
        let p2 = Point::new_from_f64(1., -1., 0.);
        let p3 = Point::new_from_f64(1., 1., 0.);
        let tr2 = Triangle::new(vec![p1,p2,p3]);

        assert!(tr2.does_triangle_contain_point(tr1.get_ref(1)));
    }
}


