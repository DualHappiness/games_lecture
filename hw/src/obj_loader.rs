use nalgebra::{Vector2, Vector3};
type Vector3f = Vector3<f32>;
type Vector2f = Vector2<f32>;

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vector3f,
    pub normal: Vector3f,
    pub texture_coordinates: Vector2f,
}

#[derive(Default)]
pub struct Material {
    pub name: String,
    /// Ambient Color
    pub ka: Vector3f,
    /// Diffuse Color
    pub kd: Vector3f,
    /// Specular Color
    pub ks: Vector3f,

    /// Specular Exponent
    pub ns: f32,
    /// Optical Density
    pub ni: f32,
    /// Dissolve
    pub d: f32,
    /// Illumination
    pub illum: i32,

    pub map_ka: String,
    pub map_kd: String,
    pub map_ks: String,
    pub map_ns: String,
    pub map_d: String,
    pub map_bump: String,
}

#[derive(Default)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            ..Default::default()
        }
    }
}

pub struct Loader {
    pub loaded_meshes: Vec<Mesh>,
    pub loaded_vertices: Vec<Vertex>,
    pub loaded_indices: Vec<u32>,
    pub loaded_materials: Vec<Material>,
}

impl Drop for Loader {
    fn drop(&mut self) {
        self.clear();
    }
}

mod algorithm {
    fn is_space(c: &char) -> bool {
        *c == '\t' || *c == ' '
    }
    pub fn first_token(s: &str) -> String {
        s.chars()
            .skip_while(is_space)
            .take_while(|c| !is_space(c))
            .collect()
    }

    pub fn tail(s: &str) -> String {
        let last = s.chars().rev().take_while(|c| !is_space(c)).count();
        s.chars()
            .take(s.len() - last)
            .skip_while(is_space)
            .skip_while(|c| !is_space(c))
            .skip_while(is_space)
            .collect()
    }

    pub fn split<'a>(s: &'a str, token: &str) -> Vec<&'a str> {
        s.split(token).collect()
    }
}

use std::io::prelude::*;
use std::io::{self, BufReader};
impl Loader {
    fn clear(&mut self) {
        self.loaded_indices.clear();
        self.loaded_meshes.clear();
        self.loaded_vertices.clear();
    }
    pub fn LoadFile(&mut self, path: &str) -> io::Result<()> {
        if path.chars().skip(path.len() - 4).collect::<String>() != ".obj" {
            return Err(io::Error::new(io::ErrorKind::Other, "err path"));
        }

        let f = std::fs::File::open(path)?;
        let f = BufReader::new(f);
        self.clear();

        let mut positions = vec![];
        let mut tex_coords = vec![];
        let mut normals = vec![];
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut mesh_mat_names = vec![];

        let mut listening = false;
        let mut mesh_name = "";

        #[cfg(feather = "show_loader_print")]
        let mut output_every_nth = 1000;
        #[cfg(feather = "show_loader_print")]
        let mut output_indicator = output_every_nth;

        for line in f.lines().map(|l| l.unwrap()) {
            #[cfg(feather = "show_loader_print")]
            {
                output_indicator = (output_indicator + 1) % output_every_nth;
                if output_indicator == 1 && !mesh_name.is_empty() {
                    print!("\r- {}", mesh_name);
                    print!("\t| vertices > {}", positions.len());
                    print!("\t| texcoords > {}", tex_coords.len());
                    print!("\t| normals > {}", normals.len());
                    print!("\t| triangles > {}", vertices.len() / 3);
                    print!("\t| material: {}", mesh_mat_names.last().unwrap_or(""));
                    println!("")
                }
            }

            match &algorithm::first_token(&line)[..] {
                s if s == "o" || s == "g" || line.chars().nth(0).unwrap() == 'g' => {
                    if listening && !indices.is_empty() && !vertices.is_empty() {
                        let mut temp = Mesh::new(vertices.clone(), indices.clone());
                        temp.name = mesh_name.to_string();

                        self.loaded_meshes.push(temp);

                        vertices.clear();
                        indices.clear();
                        mesh_name = &algorithm::tail(&line);
                    } else {
                        listening = true;
                        mesh_name = match &algorithm::first_token(&line)[..] {
                            "o" | "g" => &algorithm::tail(&line),
                            _ => "unnamed",
                        }
                    }
                    #[cfg(feather = "show_loader_print")]
                    {
                        println!("");
                        output_indicator = 0;
                    }
                }
                "v" => {
                    let s = algorithm::split(&algorithm::tail(&line), " ");
                    assert_eq!(s.len(), 3);
                    positions.push(Vector3f::new(
                        s[0].parse().unwrap(),
                        s[1].parse().unwrap(),
                        s[2].parse().unwrap(),
                    ));
                }
                "vt" => {
                    let s = algorithm::split(&algorithm::tail(&line), " ");
                    assert_eq!(s.len(), 2);
                    tex_coords.push(Vector2f::new(
                        s[0].parse().unwrap(),
                        s[1].parse().unwrap(),
                    ));
                }
                "vn" => {
                    let s = algorithm::split(&algo, token: &str)
                }
                "f" => {}
                "usemtl" => {}
                "mtllib" => {}
            }
        }
        Ok(())
    }
}
