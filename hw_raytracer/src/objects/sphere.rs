use super::*;

#[derive(Default)]
pub struct Sphere {
    pub obj: Obj,
    pub center: Vector3f,
    pub radius: f32,
    pub radius2: f32,
    pub area: f32,
    pub material: Rc<Material>,
}

impl Clone for Sphere {
    fn clone(&self) -> Self {
        Self {
            material: Rc::clone(&self.material),
            ..*self
        }
    }
}

impl Sphere {
    pub fn new(center: Vector3f, radius: f32) -> Self {
        Self {
            center,
            radius,
            radius2: radius * radius,
            area: 4f32 * PI * radius * radius,
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

impl DerefMut for Sphere {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.obj
    }
}

impl Object for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<(f32, usize)> {
        let l = ray.origin - self.center;
        let a = ray.direction.dot(&ray.direction);
        let b = 2f32 * ray.direction.dot(&l);
        let c = l.dot(&l) - self.radius2;
        match solve_quadratic(a, b, c) {
            None => None,
            Some((t0, t1)) => {
                if t0 < 0f32 && t1 < 0f32 {
                    None
                } else {
                    Some((if t0 < 0f32 { t1 } else { t0 }, 0))
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
        nalgebra::zero()
    }

    fn get_intersection(&self, ray: &ray::Ray) -> intersection::Intersection {
        let mut ret: Intersection = Default::default();
        match self.intersect(&ray) {
            None => (),
            Some((t, _)) => {
                ret.happened = true;
                ret.coords = ray.origin + ray.direction * t;
                ret.normal = (ret.coords - self.center).normalize();
                ret.m = Some(Rc::clone(&self.material));
                ret.obj = Some(Box::from(self.clone()) as Box<dyn Object>);
                ret.distance = t;
            }
        }
        ret
    }
    fn get_bounds(&self) -> bound::Bound3 {
        let offset = Vector3f::from_element(self.radius);
        Bound3::new(self.center - offset, self.center + offset)
    }
    fn get_area(&self) -> f32 {
        self.area
    }
    fn sample(&self, pos: &mut intersection::Intersection, pdf: &mut f32) {
        let theta = 2f32 * PI * get_random_float();
        let phi = PI * get_random_float();

        let dir = vector3! {phi.cos(), phi.sin() * theta.cos(), phi.sin() * theta.sin()};
        pos.coords = self.center + self.radius * dir;
        pos.normal = dir;
        pos.emit = self.material.emission;

        *pdf = 1f32 / self.area;
    }
    fn has_emit(&self) -> bool {
        self.material.has_emission()
    }
}
