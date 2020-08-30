use super::texture::Texture;
use nalgebra::{Vector2, Vector3};

type Vector2f = Vector2<f32>;
type Vector3f = Vector3<f32>;
#[derive(Default)]
pub struct FragmentShaderPayload<'a> {
    pub view_pos: Vector3f,
    pub color: Vector3f,
    pub normal: Vector3f,
    pub tex_coords: Vector2f,
    pub texture: Option<&'a Texture>,
}

impl<'a> FragmentShaderPayload<'a> {
    pub fn new(
        color: Vector3f,
        normal: Vector3f,
        tex_coords: Vector2f,
        texture: Option<&'a Texture>,
    ) -> Self {
        Self {
            color,
            normal,
            tex_coords,
            texture,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct VertexShaderPayload {
    pub position: Vector3f,
}
