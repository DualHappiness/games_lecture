use super::*;

#[derive(Default)]
pub struct Sphere {
    pub obj: Obj,
    pub center: Vector3f,
    pub radius: f32,
    pub radius2: f32,
}

impl Sphere {
    pub fn new(center: Vector3f, radius: f32) -> Self {
        Self {
            center,
            radius,
            radius2: radius * radius,
            ..Default::default()
        }
    }
}

impl Deref for Sphere {
    type Target = Obj;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl Object for Sphere {
    fn intersect(&self, origin: &Vector3f, dir: &Vector3f) -> Option<(f32, usize, Vector2f)> {
        let l = origin - self.center;
        let a = dir.dot(&dir);
        let b = 2f32 * dir.dot(&l);
        let c = l.dot(&l) - self.radius2;
        match solve_quadratic(a, b, c) {
            None => None,
            Some((t0, t1)) => {
                if t0 < 0f32 && t1 < 0f32 {
                    None
                } else {
                    Some((if t0 < 0f32 { t1 } else { t0 }, 0, nalgebra::zero()))
                }
            }
        }
    }

    fn get_surface_properties(
        &self,
        p: &Vector3f,
        _: &Vector3f,
        _index: &usize,
        _uv: &Vector2f,
    ) -> (Vector3f, Vector2f) {
        ((p - self.center).normalize(), nalgebra::zero())
    }

    fn eval_diffuse_color(&self, _: &Vector2f) -> Vector3f {
        self.obj.diffuse_color
    }
}
