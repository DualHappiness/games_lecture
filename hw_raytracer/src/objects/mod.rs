use super::*;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy)]
pub struct Obj {
    pub material_type: MaterialType,
    pub ior: f32,
    pub kd: f32,
    pub ks: f32,
    pub diffuse_color: Vector3f,
    pub specular_exponent: i32,
}

impl Default for Obj {
    fn default() -> Self {
        Self {
            material_type: MaterialType::default(),
            ior: 1.3,
            kd: 0.8,
            ks: 0.2,
            diffuse_color: Vector3f::from_element(0.2),
            specular_exponent: 25,
        }
    }
}

pub trait Object: Deref<Target = Obj> + DerefMut<Target = Obj> {
    fn intersect(&self, ray: &Ray) -> Option<(f32, usize)>;

    fn get_intersection(&self, ray: &Ray) -> Intersection;

    fn get_surface_properties(
        &self,
        _: &Vector3f,
        _: &Vector3f,
        index: &usize,
        uv: &Vector2f,
    ) -> (Vector3f, Vector2f);

    fn eval_diffuse_color(&self, _: &Vector2f) -> Vector3f;

    fn get_bounds(&self) -> Bound3;

    fn get_area(&self) -> f32;

    fn sample(&self, pos: &mut Intersection, pdf: &mut f32);

    fn has_emit(&self) -> bool;
}

pub mod sphere;
pub mod triangle;
