use crate::utility::*;
use crate::material::MaterialId;

// TODO: separate the mesh (= vertices + indices) and the instance (= mesh + transformation + material)

#[derive(Clone)]
pub struct Vertex {
    pub position: Rvec3,
    pub normal: Rvec3,
    pub uv: Rvec2,
}

declare_index_wrapper!(MeshId, u32);
declare_index_wrapper!(TriangleId, u32);

// ------------------------------------------- Mesh storage -------------------------------------------

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: MaterialId,
}

impl Mesh {
    pub fn get_triangle(&self, triangle: TriangleId) -> (Vertex, Vertex, Vertex) {
        let a = self.vertices[self.indices[triangle.to_index() + 0] as usize].clone();
        let b = self.vertices[self.indices[triangle.to_index() + 1] as usize].clone();
        let c = self.vertices[self.indices[triangle.to_index() + 2] as usize].clone();
        (a, b, c)
    }

    pub fn iter_triangles(&self) -> impl Iterator<Item = TriangleId> {
        (0..self.indices.len() / 3).map(|i| TriangleId(3 * i as u32))
    }
}

// ------------------------------------------- Mesh loading -------------------------------------------

mod obj_parser {
    use std::{io::BufRead, error::Error};
    use nom::{
        IResult,
        bytes::complete::{tag, take_while},
        sequence::tuple,
        combinator::{map_res, map, opt},
        character::complete::space1,
        number::complete::double,
        multi::separated_list1,
        branch::alt,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Index {
        pub position: u32,
        pub normal: Option<u32>,
        pub texcoord: Option<u32>,
    }

    fn parse_index(input: &str) -> IResult<&str, Index> {
        let integer = map_res(take_while(|c: char| c.is_ascii_digit()), |x| u32::from_str_radix(x, 10));

        map_res(
            separated_list1(tag("/"), opt(integer)),
            |indices: Vec<Option<u32>>| -> Result<_, &str> {
                let position = indices.get(0).cloned().flatten().ok_or("Position index not provided").map(|x| x - 1)?;
                let normal = indices.get(2).cloned().flatten().map(|x| x - 1);
                let texcoord = indices.get(1).cloned().flatten().map(|x| x - 1);
                Ok(Index {position, normal, texcoord})
            }
        )(input)
    }

    enum Line {
        V([f64; 3]),
        Vn([f64; 3]),
        Vt([f64; 2]),
        F(Vec<Index>),
    }
    
    fn parse_vec3(input: &str) -> IResult<&str, [f64; 3]> {
        map(tuple((double, space1, double, space1, double)), |(x, _, y, _, z)| [x, y, z])(input)
    }

    fn parse_vec2(input: &str) -> IResult<&str, [f64; 2]> {
        map(tuple((double, space1, double)), |(x, _, y)| [x, y])(input)
    }

    fn parse_line(input: &str) -> IResult<&str, Line> {
        let v = map(tuple((tag("v"), space1, parse_vec3)), |(_, _, v)| Line::V(v));
        let vn = map(tuple((tag("vn"), space1, parse_vec3)), |(_, _, vn)| Line::Vn(vn));
        let vt = map(tuple((tag("vt"), space1, parse_vec2)), |(_, _, vt)| Line::Vt(vt));
        let f = map(tuple((tag("f"), space1, separated_list1(space1, parse_index))), |(_, _, f)| Line::F(f));

        alt((v, vn, vt, f))(input)
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Face {
        pub first_vertex: u32,
        pub num_vertices: u32,
    }
    
    #[derive(Default, Clone)]
    pub struct ParsedObj {
        pub positions: Vec<[f64; 3]>,
        pub normals: Vec<[f64; 3]>,
        pub texcoords: Vec<[f64; 2]>,
        pub vertices: Vec<Index>,
        pub faces: Vec<Face>,
    }

    pub fn parse_obj<B: BufRead>(obj: B) -> Result<ParsedObj, Box<dyn Error>> {
        let mut parsed_obj = ParsedObj::default();
        
        for line in obj.lines() {
            let line = line?;
            let parsed_line = match parse_line(&line) {
                Ok((_, parsed_line)) => parsed_line,
                Err(_) => continue
            };
            match parsed_line {
                Line::V(v) => parsed_obj.positions.push(v),
                Line::Vn(vn) => parsed_obj.normals.push(vn),
                Line::Vt(vt) => parsed_obj.texcoords.push(vt),
                Line::F(f) => {
                    let first_vertex = parsed_obj.vertices.len() as _;
                    let num_vertices = f.len() as _;
                    parsed_obj.faces.push(Face {first_vertex, num_vertices});
                    parsed_obj.vertices.extend(f.iter());
                }
            }
        }

        Ok(parsed_obj)
    }
}

pub mod obj {
    use super::*;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::BufReader;
    use std::error::Error;

    pub fn load(path: &str) -> Result<Mesh, Box<dyn Error>> {
        const DEFAULT_NORMAL: Rvec3 = vector![0.0, 0.0, 0.0];
        const DEFAULT_UV: Rvec2 = vector![0.0, 0.0];

        let parsed_obj = obj_parser::parse_obj(BufReader::new(File::open(path)?))?;

        let mut unique_vertices = HashMap::<obj_parser::Index, u32>::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Fill in the mesh's vertices
        for v in parsed_obj.vertices.iter() {
            if unique_vertices.get(&v).is_none() {
                // New vertex encountered, add it to the mesh
                let new_index = vertices.len() as u32;
                unique_vertices.insert(*v, new_index);
                let position = parsed_obj.positions[v.position as usize].into();
                let normal = v.normal.map_or(DEFAULT_NORMAL, |x| parsed_obj.normals[x as usize].into());
                let uv = v.texcoord.map_or(DEFAULT_UV, |x| parsed_obj.texcoords[x as usize].into());
                vertices.push(Vertex {position, normal, uv});
            }
        }

        // Fill in the mesh's indices
        for f in parsed_obj.faces.iter() {
            if f.num_vertices != 3 {
                return Err("Non-triangular face are not supported".into())
            }
            let a = unique_vertices[&parsed_obj.vertices[f.first_vertex as usize + 0]];
            let b = unique_vertices[&parsed_obj.vertices[f.first_vertex as usize + 1]];
            let c = unique_vertices[&parsed_obj.vertices[f.first_vertex as usize + 2]];
            indices.push(a);
            indices.push(b);
            indices.push(c);
        }
        
        let material = MaterialId(0);
        Ok(Mesh {vertices, indices, material})
    }
}