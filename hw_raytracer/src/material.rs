use super::*;

#[derive(Clone, Copy)]
pub enum MaterialType {
    DiffuseAndGlossy,
    ReflectionAndRefraction,
    Reflection,
}

pub struct Material {
    pub r#type: MaterialType,
    pub color: Vector3f,
    pub emission: Vector3f,
    pub ior: f32,
    pub kd: f32,
    pub ks: f32,
    pub specular_exponent: i32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            r#type: MaterialType::DiffuseAndGlossy,
            color: Vector3f::from_element(1f32),
            emission: Vector3f::from_element(0f32),
            ior: Default::default(),
            kd: Default::default(),
            ks: Default::default(),
            specular_exponent: Default::default(),
        }
    }
}

impl Material {
    pub fn new(r#type: MaterialType, color: Vector3f, emission: Vector3f) -> Self {
        Self {
            r#type,
            color,
            emission,
            ..Default::default()
        }
    }
}
