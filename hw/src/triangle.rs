extern crate nalgebra as na;
use na::{Vector2, Vector3, Vector4};

#[derive(Debug, Default)]
pub struct Triangle {
    pub v: [Vector3<f32>; 3],
    pub color: [Vector3<f32>; 3],
    tex_coords: [Vector2<f32>; 3],
    normal: [Vector3<f32>; 3],
}

impl Triangle {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn a(&self) -> Vector3<f32> {
        self.v[0]
    }
    pub fn b(&self) -> Vector3<f32> {
        self.v[1]
    }
    pub fn c(&self) -> Vector3<f32> {
        self.v[2]
    }

    pub fn set_vertex(&mut self, ind: usize, ver: Vector3<f32>) {
        self.v[ind] = ver;
    }

    pub fn set_normal(&mut self, ind: usize, n: Vector3<f32>) {
        self.normal[ind] = n;
    }

    pub fn set_color(&mut self, ind: usize, r: f32, g: f32, b: f32) -> Result<(), String> {
        if r < 0.0 || r > 255.0 || g < 0.0 || g > 255.0 || b < 0.0 || b > 255.0 {
            Err("Invalid color values".to_owned())
        } else {
            self.color[ind] = Vector3::new(r / 255.0, g / 255.0, b / 255.0);
            Ok(())
        }
    }

    pub fn set_tex_coord(&mut self, ind: usize, s: f32, t: f32) {
        self.tex_coords[ind] = Vector2::new(s, t)
    }

    pub fn to_vector4(&self) -> [Vector4<f32>; 3] {
        let mut ret = [Default::default(); 3];
        ret.iter_mut()
            .enumerate()
            .for_each(|(i, v)| *v = Vector4::new(self.v[i].x, self.v[i].y, self.v[i].z, 1.0));
        ret
    }

    pub fn get_color(&self) -> Vector3<f32> {
        self.color[0] * 255f32
    }
}
