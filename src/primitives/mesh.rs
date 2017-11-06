// #![feature(test)]
// use test::Bencher;

use primitives::point;
use primitives::vector;
use primitives::number::*;
use primitives::Plane;

// use bidir_map::BidirMap;
// use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

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
    // Indexes of NeighborS in this triangle
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
    pub fn add_triangle(&mut self, tr : Triangle) -> usize {
        if tr.degradation_level() != 0
        {
            panic!("ERROR: useless triangle {:?}", tr);
            //return
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
        return it;
    }

    /// This method adds each triangle from a vector of triangles to the topology.
    /// # Arguments
    ///
    /// * `ts` - A vector of triangles
    pub fn add_triangles(&mut self, ts : Vec<Triangle>) {
        for t in ts {
            self.add_triangle(t);
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
        self.add_triangle(Triangle::new(vec![v1, v2, v3]));

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
        /*
        использовать осторожно:
        поля ins в MeshTriangle по прежнему могут содеражть индексы удаленных треугольников
        ip_to_its - тоже может содержать индексы удаленных треугольников в векторах.
        */
        let res = self.index_to_triangle.remove(index);
        if res.is_some() {
            self.header.num_triangles -= 1;
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

    #[allow(dead_code)]
    pub(crate) fn geometry_check(&self) -> bool {
        debug!("geometry check is performing ...");
        for index in 0..self.index_to_triangle.len() {
            let sc_ts = self.find_segment_conjugated_triangles(index);

            if sc_ts.len() != 3 {
                debug!("\t{:?}", sc_ts);

                debug!("\tnum of nts is {0}", self.index_to_triangle[&index].ins.len());
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

}

#[cfg(test)]
mod test {
    // use std;
    use std::io::Cursor;
    use std::fs::File;

    use primitives::point;
    use primitives::mesh;
    use primitives::triangle::Triangle;

    #[test]
    fn write_read() {
        // Make sure we can write and read a simple file.
        let mut mesh = mesh::Mesh::new();
        let t = Triangle::new(
            vec![point::Point::new_from_f64(0f64, 0f64, 0f64),
                 point::Point::new_from_f64(0f64, 0f64, 1f64),
                 point::Point::new_from_f64(1f64, 0f64, 1f64),]
        );

        mesh.add_triangle(t);



        let mut buffer = Vec::new();

        match mesh.write_stl(&mut buffer) {
            Ok(_) => (),
            Err(_) => panic!()
        }

        match mesh::Mesh::read_stl(&mut Cursor::new(buffer)) {
            Ok(stl) => {
                assert!(stl.header.num_triangles == mesh.header.num_triangles);
                assert!(stl.index_to_triangle.len() == 1);
                assert!(stl.index_to_triangle[&0] == mesh.index_to_triangle[&0])
            },
            Err(_) => panic!()
        }
    }

    fn rw_test(file_name : &str)
    {
        let mut mesh = mesh::Mesh::new();
        let t = Triangle::new(
            vec![point::Point::new_from_f64(0f64, 0f64, 0f64),
                 point::Point::new_from_f64(0f64, 0f64, 1f64),
                 point::Point::new_from_f64(1f64, 0f64, 1f64),]
        );

        mesh.add_triangle(t);

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
        let mesh = mesh::Mesh::read_stl(&mut f).unwrap();

        let mut f = File::create("res_of_tests/import_export/skull_new.stl").unwrap();
        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };
    }
}




