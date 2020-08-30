use super::*;

pub struct Ray {
    pub origin: Vector3f,
    pub direction: Vector3f,
    pub direction_inv: Vector3f,
    pub t: f32,
    pub t_min: f32,
    pub t_max: f32,
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            origin: nalgebra::zero(),
            direction: nalgebra::zero(),
            direction_inv: nalgebra::zero(),
            t: 0f32,
            t_min: 0f32,
            t_max: f32::MAX,
        }
    }
}

impl Ray {
    pub fn new(origin: Vector3f, direction: Vector3f, t: f32) -> Self {
        Self {
            origin,
            direction,
            t,
            direction_inv: Vector3f::new(
                1f32 / direction.x,
                1f32 / direction.y,
                1f32 / direction.z,
            ),
            ..Default::default()
        }
    }

    pub fn at(&self, t: f32) -> Vector3f {
        self.origin + t * self.direction
    }
}
