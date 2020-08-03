use super::*;
use std::ops::Deref;

#[derive(Default)]
pub struct Obj {
    pub material_type: MaterialType,
    pub ior: f32,
    pub kd: f32,
    pub ks: f32,
    pub diffuse_color: Vector3f,
    pub specular_exponent: f32,
}

pub trait Object: Deref<Target = Obj> {
    fn intersect(&self, origin: &Vector3f, dir: &Vector3f) -> Option<(f32, usize, Vector2f)>;

    fn get_surface_properties(
        &self,
        _: &Vector3f,
        _: &Vector3f,
        index: &usize,
        uv: &Vector2f,
    ) -> (Vector3f, Vector2f);

    fn eval_diffuse_color(&self, _: &Vector2f) -> Vector3f;
}

pub mod sphere;
pub mod triangle;
