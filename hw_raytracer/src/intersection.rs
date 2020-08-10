use super::*;

#[derive(Default)]
pub struct Intersection {
    pub happened: bool,
    pub coords: Vector3f,
    pub tcoords: Vector3f,
    pub normal: Vector3f,
    pub emit: Vector3f,
    pub distance: f32,
    pub obj: Option<Box<dyn Object>>,
    pub m: Option<Arc<Material>>,
}
