use super::global::*;

pub struct Light {
    pub position: Vector3f,
    pub intensity: Vector3f,
}

impl Light {
    pub fn new(position: Vector3f, intensity: Vector3f) -> Self {
        Self {
            position,
            intensity,
        }
    }
}
