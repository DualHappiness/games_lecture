use nalgebra::{Vector2, Vector3};
type Vector3f = Vector3<f32>;
type Vector2f = Vector2<f32>;

#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub position: Vector3f,
    pub normal: Vector3f,
    pub texture_coordinates: Vector2f,
}

#[derive(Default, Clone)]
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
    pub indices: Vec<usize>,
    pub material: Material,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<usize>) -> Self {
        Self {
            vertices,
            indices,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct Loader {
    pub loaded_meshes: Vec<Mesh>,
    pub loaded_vertices: Vec<Vertex>,
    pub loaded_indices: Vec<usize>,
    pub loaded_materials: Vec<Material>,
}

impl Drop for Loader {
    fn drop(&mut self) {
        self.clear();
    }
}

mod math {
    use super::Vector3f;
    pub fn angle_between_v3(a: Vector3f, b: Vector3f) -> f32 {
        let mut angle = a.dot(&b);
        angle /= a.magnitude() * b.magnitude();
        angle.acos()
    }
}

mod algorithm {
    use super::Vector3f;
    fn same_side(p1: Vector3f, p2: Vector3f, a: Vector3f, b: Vector3f) -> bool {
        let cp1 = (b - a).cross(&(p1 - a));
        let cp2 = (b - a).cross(&(p2 - a));

        cp1.dot(&cp2) >= 0f32
    }
    pub fn in_triangle(point: Vector3f, a: Vector3f, b: Vector3f, c: Vector3f) -> bool {
        let within_tri_prisim =
            same_side(point, a, b, c) && same_side(point, b, a, c) && same_side(point, c, a, b);
        if !within_tri_prisim {
            return false;
        }

        let normal = (b - a).cross(&(c - a)).normalize();
        normal.dot(&point) == 0f32
    }

    fn parse(c: &&str) -> f32 {
        c.parse().unwrap()
    }

    pub fn parse_str_vec(v: Vec<&str>) -> Vec<f32> {
        v.iter().map(parse).collect()
    }

    pub fn get_vector3_from_line(line: &str) -> Vector3f {
        let t = tail(&line);
        let s = split(&line, " ");
        assert_eq!(s.len(), 3);
        Vector3f::from_column_slice(&parse_str_vec(s))
    }

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

    pub fn get_element<T: Clone>(elements: &Vec<T>, index: &str) -> T {
        let index: i32 = index.parse().unwrap();
        let index = if index < 0 {
            elements.len() as i32 + index
        } else {
            index - 1
        };
        elements[index as usize].clone()
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

    fn gen_vertices_from_raw_obj(
        positions: &Vec<Vector3f>,
        tex_coords: &Vec<Vector2f>,
        normals: &Vec<Vector3f>,
        line: &str,
    ) -> Vec<Vertex> {
        let mut ret = vec![];
        let no_normal = algorithm::split(&algorithm::tail(line), " ")
            .iter()
            .map(|face| {
                let t = algorithm::tail(&face);
                let svert = algorithm::split(&t, "/");
                assert!(!svert.is_empty());
                let mut vertex: Vertex = Default::default();
                vertex.position = algorithm::get_element(positions, svert[0]);

                if svert.len() > 1 && !svert[1].is_empty() {
                    vertex.texture_coordinates = algorithm::get_element(tex_coords, svert[1]);
                }

                if svert.len() > 2 {
                    vertex.normal = algorithm::get_element(normals, svert[2]);
                }
                ret.push(vertex);
                svert.len() <= 2
            })
            .any(|no_normal| no_normal);

        // take care of missing normals
        // these may not be truly acurate but it is the
        // best they get for not compiling a mesh with normals
        if no_normal {
            assert!(ret.len() > 2);
            let a = ret[0].position - ret[1].position;
            let b = ret[2].position - ret[1].position;

            let normal = a.cross(&b);

            ret.iter_mut().for_each(|vert| vert.normal = normal);
        }
        ret
    }

    // Triangulate a list of vertices into a face by printing
    // inducies corresponding with triangles within it
    fn vertex_triangluation(vertices: &Vec<Vertex>) -> Vec<usize> {
        if vertices.len() < 3 {
            return vec![];
        }

        // it is a triangle
        if vertices.len() == 3 {
            return vec![0, 1, 2];
        }

        // 索引数组
        let mut temp: Vec<_> = (0..vertices.len()).collect();
        let mut ret = vec![];
        let find = |index_vec: &Vec<usize>| {
            let n = index_vec.len();
            (0..n)
                .map(|i| {
                    (
                        if i == 0 { n - 1 } else { i - 1 },
                        i,
                        if i == n - 1 { 0 } else { i + 1 },
                    )
                })
                .map(|(prev, cur, next)| {
                    (
                        (prev, cur, next),
                        (
                            vertices[index_vec[prev]].position,
                            vertices[index_vec[cur]].position,
                            vertices[index_vec[next]].position,
                        ),
                    )
                })
                .filter(|(_index, (prev, cur, next))| {
                    let angle =
                        math::angle_between_v3(prev - cur, next - cur) * (180f32 / 3.14159265359);
                    angle > 0f32 && angle < 180f32
                })
                // If any vertices are within this triangle
                .filter(|&((prev_idx, cur_idx, next_idx), (prev, cur, next))| {
                    !vertices
                        .iter()
                        .enumerate()
                        .filter(|&(idx, _)| {
                            idx != index_vec[prev_idx]
                                && idx != index_vec[cur_idx]
                                && idx != index_vec[next_idx]
                        })
                        .any(|(_, vert)| algorithm::in_triangle(vert.position, prev, cur, next))
                })
                .map(|(index, _)| index)
                .nth(0)
        };
        loop {
            // spicial case
            match temp.len() {
                3 => {
                    ret.append(&mut vec![temp[0], temp[2], temp[1]]);
                    return ret;
                }
                4 => {
                    ret.append(&mut vec![temp[0], temp[3], temp[1]]);
                    ret.append(&mut vec![temp[3], temp[1], temp[2]]);
                    return ret;
                }
                _ => (),
            }
            // 查找符合条件的点
            match find(&temp) {
                None => {
                    if ret.is_empty() || temp.is_empty() {
                        return ret;
                    }
                }
                Some((cur, prev, next)) => {
                    ret.append(&mut vec![temp[cur], temp[prev], temp[next]]);
                    temp.remove(cur);
                }
            }
        }
    }

    fn load_materials(&mut self, path: &str) -> io::Result<()> {
        if path.chars().skip(path.len() - 4).collect::<String>() != ".mtl" {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "wrong format, not mtl",
            ));
        }
        let f = std::fs::File::open(path)?;
        let f = BufReader::new(f);

        let mut first = true;
        let mut temp = Default::default();
        for line in f.lines().map(|l| l.unwrap()) {
            match &algorithm::first_token(&line)[..] {
                "newmtl" => {
                    if !first {
                        self.loaded_materials.push(temp);
                        temp = Default::default();
                    }
                    first = false;
                    temp.name = if line.len() > 7 {
                        algorithm::tail(&line)
                    } else {
                        "none".to_string()
                    };
                }
                // ambient color
                "Ka" => temp.ka = algorithm::get_vector3_from_line(&line),
                // diffuse color
                "Kd" => temp.kd = algorithm::get_vector3_from_line(&line),
                // specular color
                "Ks" => temp.kd = algorithm::get_vector3_from_line(&line),
                // specular exponent
                "Ns" => temp.ns = algorithm::tail(&line).parse().unwrap(),
                // optical density
                "Ni" => temp.ni = algorithm::tail(&line).parse().unwrap(),
                // dissolv
                "d" => temp.d = algorithm::tail(&line).parse().unwrap(),
                // Illumination
                "illum" => temp.illum = algorithm::tail(&line).parse().unwrap(),
                // ambient texture map
                "map_Ka" => temp.map_ka = algorithm::tail(&line),
                // diffuse texture map
                "map_Kd" => temp.map_kd = algorithm::tail(&line),
                // specular texture map
                "map_Ks" => temp.map_kd = algorithm::tail(&line),
                // specular highlight map
                "map_Ns" => temp.map_ns = algorithm::tail(&line),
                // alpha texture map
                "map_d" => temp.map_d = algorithm::tail(&line),
                // bump map
                "map_Bump" | "map_bump" | "bump" => temp.map_bump = algorithm::tail(&line),
                _ => panic!("wrong material format"),
            };
        }
        self.loaded_materials.push(temp);
        Ok(())
    }

    pub fn load_file(&mut self, path: &str) -> io::Result<()> {
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
        let mut mesh_name = "".to_owned();

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
                        mesh_name = algorithm::tail(&line);
                    } else {
                        listening = true;
                        mesh_name = match &algorithm::first_token(&line)[..] {
                            "o" | "g" => algorithm::tail(&line),
                            _ => "unnamed".to_owned(),
                        }
                    }
                    #[cfg(feather = "show_loader_print")]
                    {
                        println!("");
                        output_indicator = 0;
                    }
                }
                "v" => positions.push(algorithm::get_vector3_from_line(&line)),
                "vt" => {
                    let t = algorithm::tail(&line);
                    let s = algorithm::split(&t, " ");
                    assert_eq!(s.len(), 2);
                    tex_coords.push(Vector2f::from_column_slice(&algorithm::parse_str_vec(s)));
                }
                "vn" => normals.push(algorithm::get_vector3_from_line(&line)),
                "f" => {
                    let verts =
                        Self::gen_vertices_from_raw_obj(&positions, &tex_coords, &normals, &line);
                    let vert_len = verts.len();
                    for vert in verts {
                        vertices.push(vert);
                        self.loaded_vertices.push(vert);
                    }

                    let inds = Self::vertex_triangluation(&vertices);

                    let vert_offset = vertices.len() - vert_len;
                    let loaded_vert_offset = self.loaded_vertices.len() - vert_len;
                    // vertex index
                    for i in 0..inds.len() {
                        indices.push(vert_offset + inds[i]);
                        self.loaded_indices.push(loaded_vert_offset + inds[i]);
                    }
                }
                "usemtl" => {
                    mesh_mat_names.push(algorithm::tail(&line));
                    // Create new Mesh, if Material changes within a group
                    if !indices.is_empty() && !vertices.is_empty() {
                        let mut temp_mesh = Mesh::new(vertices.clone(), indices.clone());
                        vertices.clear();
                        indices.clear();
                        temp_mesh.name = mesh_name.to_string();
                        // ! 不知道他这里在干啥  感觉应该是个bug
                        #[cfg(feather = "bug?")]
                        {
                            let mut i = 2;
                            loop {
                                temp_mesh.name = format!("{}_{}", mesh_name, i.to_string());
                                for m in &self.loaded_meshes {
                                    if m.name = temp_mesh.name {
                                        continue;
                                    }
                                }
                            }
                        }
                        self.loaded_meshes.push(temp_mesh);
                    }

                    #[cfg(feather = "show_loader_print")]
                    {
                        output_indicator = 0;
                    }
                }
                "mtllib" => {
                    let temp = algorithm::split(&path, "/");
                    let mut pathtomat = temp
                        .iter()
                        .fold("".to_owned(), |acc, cur| acc + cur + "/")
                        .to_string();
                    pathtomat += &algorithm::tail(&line);
                    #[cfg(feather = "show_loader_print")]
                    {
                        println!("");
                        println!("- find material in: {}", pathtomat);
                    }
                    self.load_materials(&pathtomat);
                }
                _ => panic!("wrong format"),
            }

            #[cfg(feather = "show_loader_print")]
            {
                println!("");
            }

            if !indices.is_empty() && !vertices.is_empty() {
                let mut temp = Mesh::new(vertices.clone(), indices.clone());
                temp.name = mesh_name.to_owned();

                self.loaded_meshes.push(temp);
            }

            for i in 0..mesh_mat_names.len() {
                match self
                    .loaded_materials
                    .iter()
                    .find(|mat| mat.name == mesh_mat_names[i])
                {
                    None => println!(
                        "no material found, mesh: {}, mat name: {}",
                        self.loaded_meshes[i].name, mesh_mat_names[i]
                    ),
                    Some(mat) => self.loaded_meshes[i].material = mat.clone(),
                }
            }
        }
        if self.loaded_meshes.is_empty()
            && self.loaded_vertices.is_empty()
            && self.loaded_indices.is_empty()
        {
            Err(io::Error::new(io::ErrorKind::Other, "cant load file"))
        } else {
            Ok(())
        }
    }
}
