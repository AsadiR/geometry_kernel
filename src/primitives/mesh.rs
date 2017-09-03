#![feature(test)]
use test::Bencher;

use primitives::point;
use primitives::vector;
use primitives::number;
use primitives::number::NumberTrait;

use bidir_map::BidirMap;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

use std::io::{Result, ErrorKind, Error};
use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};
use std::fmt;
use std::io::Cursor;
use time::PreciseTime;

use primitives::triangle::Triangle;


#[derive(Hash)]
#[derive(Clone)]
#[derive(Debug)]
pub struct MeshTriangle {
    pub normal: vector::Vector,
    // Indexes of PointS in this triangle
    pub ips : Vec<usize>,
    pub attr_byte_count: u16,
    // Indexes of NeighborS in this triangle
    pub ins: Vec<usize>
}


impl MeshTriangle {
    fn new(ps : &Vec<point::Point>) -> MeshTriangle {
        let v1 : vector::Vector = &ps[0] - &ps[1];
        let v2 : vector::Vector = &ps[1] - &ps[2];
        MeshTriangle {
            normal: v1.cross_product(&v2),
            ips: Vec::new(),
            attr_byte_count: 0,
            ins: Vec::new()
        }
    }

    pub fn get_normal(&self) -> vector::Vector {
        self.normal.clone()
    }

}

impl PartialEq for MeshTriangle {
    fn eq(&self, rhs: &MeshTriangle) -> bool {
        for i in 0..2 {
            if self.ips[i] != rhs.ips[i] {
                return false;
            }
        }

        self.normal == rhs.normal && self.attr_byte_count == rhs.attr_byte_count
    }
}

impl Eq for MeshTriangle {}

pub type Mesh = BinaryStlFile;

pub struct BinaryStlHeader {
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

#[derive(Clone)]
#[derive(Debug)]
pub struct BinaryStlFile {
    pub header: BinaryStlHeader,
    pub triangles: HashMap<usize, MeshTriangle>,
    pub ip_to_p: HashMap<usize, point::Point>,
    pub p_to_ip: HashMap<point::Point, usize>,
    // pub points: BidirMap<usize, point::Point>,
    ip_to_its: BTreeMap<usize, Vec<usize>>,
    max_tr_index: usize
}


impl BinaryStlFile {
    pub fn new() -> BinaryStlFile {
        BinaryStlFile {
            header: BinaryStlHeader { header: [0u8; 80], num_triangles: 0 },
            triangles: HashMap::new(),
            ip_to_p: HashMap::new(),
            p_to_ip: HashMap::new(),
            ip_to_its: BTreeMap::new(),
            max_tr_index: 0
        }
    }

    pub fn size(&self) -> usize {
        return self.ip_to_p.keys().len();
    }

    pub fn add_triangle(&mut self, tr : Triangle) -> usize {
        if tr.degradation_level() != 0
        {
            panic!("ERROR: useless triangle {:?}", tr);
            //return
        }

        let ps = tr.points;

        debug!("add_triangle: {:?}", ps);

        let mut m_tr = MeshTriangle::new(&ps);

        let it : usize = self.max_tr_index.clone();
        self.max_tr_index += 1;

        for p in ps {
            if self.p_to_ip.contains_key(&p) {
                let ip : usize = self.p_to_ip.get(&p).unwrap().clone();
                for int in &self.ip_to_its[&ip] {
                    // to ignore removed triangles
                    if self.triangles.contains_key(&int) {
                        m_tr.ins.push(*int);
                        self.triangles.get_mut(int).unwrap().ins.push(it);
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

        self.triangles.insert(it, m_tr);
        self.header.num_triangles += 1;
        return it;
    }

    pub fn add_triangles(&mut self, ts : Vec<Triangle>) {
        for t in ts {
            self.add_triangle(t);
        }
    }

    fn read_point<T: ReadBytesExt>(input: &mut T) -> Result<point::Point> {

        let x1 = input.read_f32::<LittleEndian>()?;

        let x2 = input.read_f32::<LittleEndian>()?;
        let x3 = input.read_f32::<LittleEndian>()?;

        Ok(point::Point {x: number::new_from_f32(x1), y: number::new_from_f32(x2), z: number::new_from_f32(x3)})

        //Ok(point::Point {x: number::new_from_f32(0.), y: number::new_from_f32(0.), z: number::new_from_f32(0.)})
        // return Err(Error::new(ErrorKind::Other, "bbb"));
    }

    fn read_triangle<T: ReadBytesExt>(&mut self, input: &mut T) -> Result<()> {
        let normal : vector::Vector = Mesh::read_point(input)?.convert_to_vector();

        let v1 = Mesh::read_point(input)?;
        let v2 = Mesh::read_point(input)?;
        let v3 = Mesh::read_point(input)?;
        let attr_count = input.read_u16::<LittleEndian>()?;


        self.add_triangle(Triangle::new(vec![v1, v2, v3]));

        Ok(())
    }

    pub fn get_normal_by_index(&self, index: usize) -> vector::Vector {
        return self.triangles[&index].get_normal()
    }

    pub fn remove_triangle(&mut self, index: &usize) {
        /*
        использовать осторожно:
        поля ins в MeshTriangle по прежнему могут содеражть индексы удаленных треугольников
        ip_to_its - тоже может содержать индексы удаленных треугольников в векторах.
        */
        self.triangles.remove(index);
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
            if i > 100000000 {
                break;
            }
            debug!("Iterration number {:?}", i);

            mesh.read_triangle(&mut cursor)?;
        }

        let end = PreciseTime::now();
        info!("{} seconds for <read_stl>", start.to(end));
        Ok(mesh)
    }

    fn write_point<T: WriteBytesExt>(out: &mut T, p: &point::Point) -> Result<()> {

        out.write_f32::<LittleEndian>(p.x.clone().to_f32())?;
        out.write_f32::<LittleEndian>(p.y.clone().to_f32())?;
        out.write_f32::<LittleEndian>(p.z.clone().to_f32())?;

        Ok(())
    }

    pub fn write_stl<T: WriteBytesExt>(
        &self,
        out: &mut T,
    ) -> Result<()> {
        debug!("dbg: {:?} : {:?}", self.header.num_triangles as usize, self.triangles.len());

        assert!(self.header.num_triangles as usize == self.triangles.len());

        //write the header.
        out.write(&self.header.header)?;
        out.write_u32::<LittleEndian>(self.header.num_triangles)?;

        // write all the triangles
        for (_, t) in self.triangles.iter() {
            // write the normal
            Mesh::write_point(out, &t.normal.gen_point())?;

            // write the points

            for ip in &t.ips {
                Mesh::write_point(out, self.ip_to_p.get(ip).unwrap())?;
            }

            out.write_u16::<LittleEndian>(t.attr_byte_count)?;
        }

        Ok(())
    }

    pub fn get_triangle(&self, index : usize) -> Triangle {
        let mt : &MeshTriangle = &self.triangles[&index];
        let p1 = self.ip_to_p[&mt.ips[0]].clone();
        let p2 = self.ip_to_p[&mt.ips[1]].clone();
        let p3 = self.ip_to_p[&mt.ips[2]].clone();

        Triangle::new(vec![p1,p2,p3])
    }

    pub fn get_number_of_coincident_points(&self, it1: usize, it2: usize) -> usize {
        let mut t1_ipset: BTreeSet<usize> = BTreeSet::new();
        t1_ipset.extend(self.triangles[&it1].ips.clone());
        let mut res : usize = 0;
        for it in self.triangles[&it2].ips.iter() {
            if t1_ipset.contains(it) {
                res += 1;
            }
        }

        return res;
    }

    pub fn find_segment_conjugated_triangles(&self, it: usize) -> Vec<usize> {
        let mut res : Vec<usize> = Vec::new();

        for int in self.triangles[&it].ins.iter() {
            let number = self.get_number_of_coincident_points(it, *int);
            if number == 2 {
                res.push(int.clone());
            } else if number > 2 {
                panic!("Something goes wrong!");
            }

        }
        return res;
    }


    /*
    pub fn get_normal_of_triangle(&self, index : usize) -> Vector {
        return self.triangles[index].normal.clone();
    }
    */

}

#[cfg(test)]
mod test {
    use std;
    use std::io::Cursor;
    use std::fs::File;

    use bidir_map::BidirMap;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

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

                // println!("mesh: {:?}", stl);
                assert!(stl.header.num_triangles == mesh.header.num_triangles);
                assert!(stl.triangles.len() == 1);
                assert!(stl.triangles[&0] == mesh.triangles[&0])
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

        //println!("mesh: {:?}", mesh);

        let mut f = File::create(file_name).unwrap();

        match mesh.write_stl(&mut f) {
            Ok(_) => (),
            Err(_) => panic!()
        };

        let mut f = File::open(file_name).unwrap();

        match mesh::Mesh::read_stl(&mut f) {
            Ok(stl) => {
                assert!(stl.header.num_triangles == mesh.header.num_triangles);
                assert!(stl.triangles.len() == 1);
                assert!(stl.triangles[&0] == mesh.triangles[&0]);
                assert!(stl.ip_to_p.get(&0) == mesh.ip_to_p.get(&0));
                assert!(stl.ip_to_p.get(&1) == mesh.ip_to_p.get(&1));
                assert!(stl.ip_to_p.get(&2) == mesh.ip_to_p.get(&2));
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

    #[ignore]
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




