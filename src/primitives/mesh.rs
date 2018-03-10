// #![feature(test)]
// use test::Bencher;

use primitives::point;
use primitives::vector;
use primitives::number::*;
use primitives::Plane;

// use bidir_map::BidirMap;
// use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::{HashMap, HashSet};


// use std::collections::HashSet;

use std::io::{Result, ErrorKind, Error};
use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};
use std::fmt;
use std::io::Cursor;
use time::PreciseTime;

use primitives::triangle::Triangle;


#[derive(Hash)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct MeshTriangle {
    pub normal: vector::Vector,
    // Indexes of PointS in this triangle
    pub ips : Vec<usize>,
    pub attr_byte_count: u16,
    // Indexes of NeighborS for this triangle
    pub ins: Vec<usize>
}


impl MeshTriangle {
    fn new(/*ps : &Vec<point::Point>,*/ normal: vector::Vector) -> MeshTriangle {
        MeshTriangle {
            normal: normal,
            ips: Vec::new(),
            attr_byte_count: 0,
            ins: Vec::new()
        }
    }

    pub fn get_normal(&self) -> vector::Vector {
        self.normal.clone()
    }

    pub(crate) fn add_neighbour(&mut self, it : usize) {
        if !self.conatin_neighbour(it) {
            self.ins.push(it);
        }
    }

    pub(crate) fn conatin_neighbour(&self, it : usize) -> bool{
        for i in self.ins.iter() {
            if it == *i {
                return true;
            }
        }
        return false;
    }
}


impl PartialEq for MeshTriangle {
    fn eq(&self, rhs: &MeshTriangle) -> bool {
        for i in 0..2 {
            if self.ips[i] != rhs.ips[i] {
                return false;
            }
        }

        self.normal == rhs.normal //&& self.attr_byte_count == rhs.attr_byte_count
    }
}

impl Eq for MeshTriangle {}


/// It is an alias for BinaryStlFile
pub type Mesh = BinaryStlFile;

struct BinaryStlHeader {
    pub header: [u8; 80],
    pub num_triangles: u32
}

impl Clone for BinaryStlHeader {
    fn clone(&self) -> BinaryStlHeader {
        let header = self.header;
        let num_triangles = self.num_triangles;
        BinaryStlHeader {
            header: header,
            num_triangles: num_triangles
        }
    }
}

impl fmt::Debug for BinaryStlHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BinaryStlHeader[{:?}]]", self.num_triangles)
    }
}


/// This class represents a geometry topology of polygonal mesh.
#[derive(Clone)]
#[derive(Debug)]
pub struct BinaryStlFile {
    header: BinaryStlHeader,
    index_to_triangle: HashMap<usize, MeshTriangle>,
    ip_to_p: HashMap<usize, point::Point>,
    p_to_ip: HashMap<point::Point, usize>,
    ip_to_its: HashMap<usize, Vec<usize>>,
    max_tr_index: usize
}


impl BinaryStlFile {


    /// This method returns a new empty BinaryStlFile
    pub fn new() -> BinaryStlFile {
        BinaryStlFile {
            header: BinaryStlHeader { header: [0u8; 80], num_triangles: 0 },
            index_to_triangle: HashMap::new(),
            ip_to_p: HashMap::new(),
            p_to_ip: HashMap::new(),
            ip_to_its: HashMap::new(),
            max_tr_index: 0
        }
    }

    /// This method returns a number of unique points, used in the file.
    pub fn num_of_points(&self) -> usize {
        return self.ip_to_p.keys().len();
    }

    /// This method returns a number of triangles.
    pub fn num_of_triangles(&self) -> usize {
        return self.index_to_triangle.len();
    }

    /// This method adds a triangle to the topology. It does not check if this triangle was added before or wasn't.
    /// # Arguments
    ///
    /// * `tr` - A triangle to add
    pub fn add_triangle(&mut self, tr : Triangle) -> Result<usize> {
        if tr.degradation_level() != 0
        {
            // panic!("ERROR: useless triangle {:?}", tr);
            return Err(
                Error::new(
                    ErrorKind::Other,
                    format!("ERROR: useless triangle {:?}", tr)
                )
            );
        }


        let normal = tr.get_normal();
        let ps = tr.get_points();

        debug!("add_triangle: {:?}", ps);

        let mut m_tr = MeshTriangle::new(/*&ps,*/ normal);

        let it : usize = self.max_tr_index.clone();
        self.max_tr_index += 1;


        for p in ps {
            if self.p_to_ip.contains_key(&p) {
                let ip : usize = self.p_to_ip.get(&p).unwrap().clone();
                for int in &self.ip_to_its[&ip] {
                    // to ignore removed triangles
                    if self.index_to_triangle.contains_key(&int) {
                        m_tr.add_neighbour(*int);
                        self.index_to_triangle.get_mut(int).unwrap().add_neighbour(it);
                    }
                }
                self.ip_to_its.get_mut(&ip).unwrap().push(it);
                m_tr.ips.push(ip);
            } else {

                let ip : usize = self.ip_to_p.len();
                self.ip_to_its.insert(ip.clone(), Vec::new());
                self.ip_to_its.get_mut(&ip).unwrap().push(it);
                self.p_to_ip.insert(p.clone(), ip.clone());
                self.ip_to_p.insert(ip, p);
                m_tr.ips.push(ip);
            }
        }

        self.index_to_triangle.insert(it, m_tr.clone());
        self.header.num_triangles += 1;

        // TODO убрать возможность вставлять одинаоквые треугольники
        return Ok(it);
    }

    /// This method adds each triangle from a vector of triangles to the topology.
    /// # Arguments
    ///
    /// * `ts` - A vector of triangles
    pub fn add_triangles(&mut self, ts : Vec<Triangle>) {
        for t in ts {
            self.add_triangle(t).ok();
        }
    }

    fn read_point<T: ReadBytesExt>(input: &mut T) -> Result<point::Point> {

        let x1 = input.read_f32::<LittleEndian>()?;

        let x2 = input.read_f32::<LittleEndian>()?;
        let x3 = input.read_f32::<LittleEndian>()?;

        Ok(point::Point {x: Number::new_from_f32(x1), y: Number::new_from_f32(x2), z: Number::new_from_f32(x3)})

        //Ok(point::Point {x: number::new_from_f32(0.), y: number::new_from_f32(0.), z: number::new_from_f32(0.)})
        // return Err(Error::new(ErrorKind::Other, "bbb"));
    }

    fn read_triangle<T: ReadBytesExt>(&mut self, input: &mut T) -> Result<()> {
        /*let normal : vector::Vector =*/ Mesh::read_point(input)?.convert_to_vector();

        let v1 = Mesh::read_point(input)?;
        let v2 = Mesh::read_point(input)?;
        let v3 = Mesh::read_point(input)?;

        /*let attr_count =*/ input.read_u16::<LittleEndian>()?;


        //self.add_triangle(Triangle::new_with_normal(vec![v1, v2, v3], normal));
        self.add_triangle(Triangle::new(vec![v1, v2, v3])).ok();

        Ok(())
    }

    pub(crate) fn get_normal_by_index(&self, index: usize) -> vector::Vector {
        return self.index_to_triangle[&index].get_normal()
    }

    pub(crate) fn get_plane_by_index(&self, index: usize) -> Plane {
        let p0_index = self.index_to_triangle[&index].ips[0];
        Plane::new(
            self.get_normal_by_index(index),
            self.ip_to_p[&p0_index].clone()
        )
    }

    pub(crate) fn remove_triangle(&mut self, index: &usize) {
        let opt_removed_t = self.index_to_triangle.remove(index);

        if opt_removed_t.is_none() {
            return;
        }

        let removed_t = opt_removed_t.unwrap();

        for index_of_n in removed_t.ins.iter() {
            let nt = self.index_to_triangle.get_mut(index_of_n).unwrap();
            nt.ins.retain(|&x| &x != index);
        }

        for ip in removed_t.ips.iter() {
            self.ip_to_its.get_mut(ip).unwrap().retain(|&x| &x != index);
        }

        self.header.num_triangles -= 1;

    }

    pub(crate) fn get_indexes_of_triangles_by_two_points(&self, p1: &point::Point, p2: &point::Point) -> Option<(usize, usize)> {
        let opt_ip1 = self.p_to_ip.get(p1);
        let opt_ip2 = self.p_to_ip.get(p2);
        if opt_ip1.is_none() || opt_ip2.is_none() {
            return None;
        }

        let ip1 = opt_ip1.unwrap().clone();
        let ip2 = opt_ip2.unwrap().clone();

        let its1 = self.ip_to_its.get(&ip1).unwrap().clone();
        let its2 = self.ip_to_its.get(&ip2).unwrap().clone();

        let mut set1 : BTreeSet<usize> = BTreeSet::new();
        let mut set2 : BTreeSet<usize> = BTreeSet::new();

        set1.extend(its1);
        set2.extend(its2);

        let intersection = set1.intersection(&set2);

        let mut res = Vec::new();
        res.extend(intersection);

        assert_eq!(res.len(), 2);

        // нужно обеспечить, что треугольник у которого есть ребро с направлением, совпадающим с p1p2 будет первым!
        let mut is_t0_plus = false;
        let ips_for_t0: Vec<usize> = self.index_to_triangle.get(res[0]).unwrap().ips.clone();
        for cur_index in 0..ips_for_t0.len() {
            let next_index = (cur_index + 1) % ips_for_t0.len();
            let (cur_ip, next_ip) = (ips_for_t0[cur_index], ips_for_t0[next_index]);
            let (cur_p, next_p) = (&self.ip_to_p[&cur_ip], &self.ip_to_p[&next_ip]);
            if cur_p == p1 && next_p == p2 {
                is_t0_plus = true;
                break;
            }
        }

        if is_t0_plus {
            return Some((*res[0], *res[1]));
        } else {
            return Some((*res[1], *res[0]));
        }
    }

    fn read_header<T: ReadBytesExt>(input: &mut T) -> Result<BinaryStlHeader> {
        let mut header = [0u8; 80];

        match input.read(&mut header) {
            Ok(n) => if n == header.len() {
                ()
            }
                else {
                    return Err(Error::new(ErrorKind::Other,
                                          "Couldn't read STL header"));
                },
            Err(e) => return Err(e)
        };

        let num_triangles = input.read_u32::<LittleEndian>()?;

        Ok(BinaryStlHeader{ header: header, num_triangles: num_triangles })
    }

    /// This static method reads a data from the `input` in STL format and creates a new topology.
    /// # Arguments
    ///
    /// * `input` - A type, implementing ReadBytesExt.
    pub fn read_stl<T: ReadBytesExt>(input: &mut T) -> Result<BinaryStlFile> {
        // read the header
        let start = PreciseTime::now();
        info!("Reading model ...");

        let header = Mesh::read_header(input)?;
        let mut mesh = Mesh::new();

        // read the whole file
        let mut buffer = Vec::new();
        input.read_to_end(&mut buffer)?;
        let mut cursor = Cursor::new(buffer);


        info!("Number of triangles is {:?}", header.num_triangles);
        for i in 0 .. header.num_triangles {
            //if i > 10000 {
            //    break;
            //}
            debug!("Iterration number {:?}", i);

            mesh.read_triangle(&mut cursor)?;
        }

        let end = PreciseTime::now();
        info!("<read_stl> is finished in {0} seconds\n", start.to(end));
        Ok(mesh)
    }

    fn write_point<T: WriteBytesExt>(out: &mut T, p: &point::Point) -> Result<()> {

        out.write_f32::<LittleEndian>(p.x.clone().convert_to_f32())?;
        out.write_f32::<LittleEndian>(p.y.clone().convert_to_f32())?;
        out.write_f32::<LittleEndian>(p.z.clone().convert_to_f32())?;

        Ok(())
    }

    /// This method writes a data from the topology to the `out` in STL format.
    /// # Arguments
    ///
    /// * `out` - A type, implementing WryteBytesExt.
    pub fn write_stl<T: WriteBytesExt>(
        &self,
        out: &mut T,
    ) -> Result<()> {
        let start = PreciseTime::now();
        info!("Writing model ...");

        //info!("dbg: {:?} : {:?}", self.header.num_triangles as usize, self.triangles.len());
        assert!(self.header.num_triangles as usize == self.index_to_triangle.len());

        //write the header.
        out.write(&self.header.header)?;
        out.write_u32::<LittleEndian>(self.header.num_triangles)?;

        // write all the triangles
        for (_, t) in self.index_to_triangle.iter() {
            // write the normal
            Mesh::write_point(out, &t.normal.get_point())?;

            // write the points

            for ip in &t.ips {
                Mesh::write_point(out, self.ip_to_p.get(ip).unwrap())?;
            }

            out.write_u16::<LittleEndian>(t.attr_byte_count)?;
        }

        let end = PreciseTime::now();
        info!("<write_stl> is finished in {0} seconds\n", start.to(end));

        Ok(())
    }

    pub(crate) fn get_triangles_and_neighbours(&self) -> (HashMap<usize, Triangle>, HashMap<usize, BTreeSet<usize>>) {
        let mut ts: HashMap<usize, Triangle> = HashMap::new();
        let mut ns: HashMap<usize, BTreeSet<usize>> = HashMap::new();

        for (k, v) in self.index_to_triangle.iter() {
            ts.insert(*k, self.get_triangle(*k));

            let mut set : BTreeSet<usize> = BTreeSet::new();
            set.extend(v.ins.clone());
            ns.insert(*k, set);
        }

        return (ts, ns);
    }

    /// This method returns a reference to HashMap, containing pairs `(id, point)`.
    /// `id` - unique identifier of a point.
    /// `point` - a point in mesh.
    pub fn get_points(&self) -> &HashMap<usize, point::Point> {
        return &self.ip_to_p;
    }

    /// This method returns `Vec`, containing indexes of triangles.
    /// Use `get_triangle(&self, index : usize)` to get `Triangle`.
    pub fn get_it_iterator(&self) -> Vec<usize>
    {
        let mut res : Vec<usize> = Vec::new();
        for k in self.index_to_triangle.keys() {
            res.push(k.clone());
        }
        return res;
    }

    /// This method returns a copy of triangle by `index`.
    /// # Arguments
    ///
    /// * `index` - An index of triangle to get
    pub fn get_triangle(&self, index : usize) -> Triangle {
        let mt : &MeshTriangle = &self.index_to_triangle[&index];
        let p1 = self.ip_to_p[&mt.ips[0]].clone();
        let p2 = self.ip_to_p[&mt.ips[1]].clone();
        let p3 = self.ip_to_p[&mt.ips[2]].clone();

        return Triangle::new_with_normal(vec![p1,p2,p3], mt.normal.clone());
    }

    /// This method returns a copy of triangle by `index` with a reversed normal.
    /// # Arguments
    ///
    /// * `index` - An index of triangle to get
    pub fn get_reversed_triangle(&self, index : usize) -> Triangle {
        let mt : &MeshTriangle = &self.index_to_triangle[&index];
        let p1 = self.ip_to_p[&mt.ips[0]].clone();
        let p2 = self.ip_to_p[&mt.ips[1]].clone();
        let p3 = self.ip_to_p[&mt.ips[2]].clone();

        return Triangle::new_with_normal(vec![p2,p1,p3], mt.normal.clone());
    }


    /// This method returns a `Vec`, containing indexes of adjacent triangles for the triangle specified by `index`.
    /// # Arguments
    ///
    /// * `index` - An index of triangle.
    pub fn get_indexes_of_neighbours(&self, index : usize) -> Vec<usize> {
        return self.index_to_triangle.get(&index).unwrap().ins.clone();
    }

    pub(crate) fn get_number_of_coincident_points(&self, it1: usize, it2: usize) -> usize {
        let mut t1_ipset: BTreeSet<usize> = BTreeSet::new();
        t1_ipset.extend(self.index_to_triangle[&it1].ips.clone());
        let mut res : usize = 0;
        for it in self.index_to_triangle[&it2].ips.iter() {
            if t1_ipset.contains(it) {
                res += 1;
            }
        }

        return res;
    }

    pub(crate) fn find_segment_conjugated_triangles(&self, it: usize) -> Vec<usize> {
        let mut res : Vec<usize> = Vec::new();

        for int in self.index_to_triangle[&it].ins.iter() {
            if self.index_to_triangle.contains_key(int) {
                let number = self.get_number_of_coincident_points(it, *int);
                if number == 2 {
                    res.push(int.clone());
                } else if number > 2 {
                    panic!("Something goes wrong!");
                }
            }

        }
        return res;
    }

    /// This method checks if each triangle has three adjacent triangles.
    #[allow(dead_code)]
    pub fn geometry_check(&self) -> bool {
        // Проверяем, что у каждого треугольника ровно три треугольника соседа.
        // Если это условие нарушается, то в геометрии присутствуют дыры или самопересечения.

        debug!("geometry check is performing ...");
        let index_to_triangle = self.index_to_triangle.clone();

        for (index, _) in index_to_triangle.iter() {
            let sc_ts = self.find_segment_conjugated_triangles(*index);

            if sc_ts.len() != 3 {
                debug!("cur_t {:?}", self.get_triangle(*index));
                debug!("\t{:?}", sc_ts);

                debug!("\tnum of nts is {0}", self.index_to_triangle[index].ins.len());
                debug!("\tnum of cts is {0}", sc_ts.len());
                for it in sc_ts {
                    debug!("\tt: {:?}", self.get_triangle(it));
                }
                return false;
            }

            if self.index_to_triangle[&index].ips.len() != 3 {
                return false;
            }
        }

        return true;
    }


    /// This method returns `Vec` of connectivity components.
    pub fn split_into_connectivity_components(self) -> Vec<Mesh> {
        let mut res : Vec<Mesh>  = Vec::new();
        let mut visited : HashSet<usize> = HashSet::new();
        for it in self.get_it_iterator() {
            if !visited.contains(&it) {
                let mut mesh = Mesh::new();
                let mut to_visit : Vec<usize> = vec![it];
                while !to_visit.is_empty() {
                    let extracted_index = to_visit.pop().unwrap();
                    if !visited.contains(&extracted_index) {
                        mesh.add_triangle(self.get_triangle(extracted_index)).ok();
                        visited.insert(extracted_index);
                        to_visit.extend(self.get_indexes_of_neighbours(extracted_index));
                    }
                }
                res.push(mesh);
            }
        }

        return res;
    }


    pub(crate) fn rotate_x(&mut self, angle: Number) {
        let start = PreciseTime::now();

        for (ip, p) in self.ip_to_p.iter_mut() {
            self.p_to_ip.remove(p);
            p.rotate_x(&angle);
            self.p_to_ip.insert(p.clone(), ip.clone());
        }

        let mut index_to_triangle = self.index_to_triangle.clone();

        for (it, mt) in index_to_triangle.iter_mut() {
            mt.normal = self.get_triangle(*it).calculate_normal();
        }

        self.index_to_triangle = index_to_triangle;

        info!("Rotation is finished in {0} seconds.\n", start.to(PreciseTime::now()));
    }

    pub(crate) fn find_xyz_ranges(&self) -> (Number, Number, Number, Number, Number, Number) {
        let mut x_min;
        let mut x_max;
        let mut y_min;
        let mut y_max;
        let mut z_min;
        let mut z_max;

        {
            let first_p = self.ip_to_p.get(&0).unwrap();
            x_min = first_p.x.clone();
            x_max = first_p.x.clone();
            y_min = first_p.y.clone();
            y_max = first_p.y.clone();
            z_min = first_p.z.clone();
            z_max = first_p.z.clone();
        }

        for p in self.ip_to_p.values() {
            if p.x < x_min {
                x_min = p.x.clone();
            }

            if p.x > x_max {
                x_max = p.x.clone();
            }

            if p.y < y_min {
                y_min = p.y.clone();
            }

            if p.y > y_max {
                y_max = p.y.clone();
            }

            if p.z < z_min {
                z_min = p.z.clone();
            }

            if p.z > z_max {
                z_max = p.z.clone();
            }
        }

        return (x_min, x_max, y_min, y_max, z_min, z_max);
    }
}

#[cfg(test)]
mod test {
    // use std;
    use std::io::Cursor;
    use std::fs::File;
    use primitives::*;

    #[test]
    fn write_read() {
        // Make sure we can write and read a simple file.
        let mut mesh = Mesh::new();
        let t = Triangle::new(
            vec![Point::new_from_f64(0f64, 0f64, 0f64),
                       Point::new_from_f64(0f64, 0f64, 1f64),
                       Point::new_from_f64(1f64, 0f64, 1f64),]
        );

        mesh.add_triangle(t).ok();

        let mut buffer = Vec::new();

        match mesh.write_stl(&mut buffer) {
            Ok(_) => (),
            Err(_) => panic!()
        }

        match mesh::Mesh::read_stl(&mut Cursor::new(buffer)) {
            Ok(stl) => {
                assert_eq!(stl.header.num_triangles, mesh.header.num_triangles);
                assert_eq!(stl.index_to_triangle.len(), 1);
                assert_eq!(stl.index_to_triangle[&0], mesh.index_to_triangle[&0])
            },
            Err(_) => panic!()
        }
    }

    fn rw_test(file_name : &str)
    {
        let mut mesh = mesh::Mesh::new();
        let t = Triangle::new(
            vec![Point::new_from_f64(0f64, 0f64, 0f64),
                       Point::new_from_f64(0f64, 0f64, 1f64),
                       Point::new_from_f64(1f64, 0f64, 1f64),]
        );

        mesh.add_triangle(t).ok();

        let mut f = File::create(file_name).unwrap();

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };

        let mut f = File::open(file_name).unwrap();

        match mesh::Mesh::read_stl(&mut f) {
            Ok(stl) => {
                assert_eq!(stl.header.num_triangles, mesh.header.num_triangles);
                assert_eq!(stl.index_to_triangle.len(), 1);
                assert_eq!(stl.index_to_triangle[&0], mesh.index_to_triangle[&0]);
                assert_eq!(stl.ip_to_p.get(&0), mesh.ip_to_p.get(&0));
                assert_eq!(stl.ip_to_p.get(&1), mesh.ip_to_p.get(&1));
                assert_eq!(stl.ip_to_p.get(&2), mesh.ip_to_p.get(&2));
            },
            Err(_) => {
                panic!();
            }
        }
    }


    #[test]
    fn file_write_read_simple() {
        rw_test("test.stl")
    }

    #[test]
    fn file_write_skull() {
        let mut f = File::open("input_for_tests/skull.stl").unwrap();
        let mesh = Mesh::read_stl(&mut f).unwrap();

        let mut f = File::create("res_of_tests/import_export/skull_new.stl").unwrap();
        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }

    #[test]
    fn read_multi_component_mesh() {
        let mut f = File::open("input_for_tests/figures_tor_cone_cube.stl").unwrap();
        let mesh = Mesh::read_stl(&mut f).unwrap();
        assert_eq!(mesh.split_into_connectivity_components().len(), 3);
    }

    #[test]
    fn x_rotation_test() {
        let mut f = File::open("input_for_tests/cylinder_8_a.stl").unwrap();
        let mut mesh = Mesh::read_stl(&mut f).unwrap();

        mesh.rotate_x(Number::new(70f64));

        let mut f = File::create("res_of_tests/import_export/rotated_cylinder.stl").unwrap();
        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };

    }
}




