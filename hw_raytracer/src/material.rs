use super::*;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum MaterialType {
    DiffuseAndGlossy,
    ReflectionAndRefraction,
    Reflection,
}

impl Default for MaterialType {
    fn default() -> Self {
        MaterialType::DiffuseAndGlossy
    }
}

#[derive(Clone, Copy)]
pub struct Material {
    pub r#type: MaterialType,
    // pub color: Vector3f,
    pub emission: Vector3f,
    pub ior: f32,
    pub kd: Vector3f,
    pub ks: Vector3f,
    pub specular_exponent: i32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            r#type: MaterialType::DiffuseAndGlossy,
            // color: Vector3f::from_element(1f32),
            emission: Vector3f::from_element(0f32),
            ior: Default::default(),
            kd: Default::default(),
            ks: Default::default(),
            specular_exponent: Default::default(),
        }
    }
}

impl Material {
    pub fn new(r#type: MaterialType, emission: Vector3f) -> Self {
        Self {
            r#type,
            emission,
            ..Default::default()
        }
    }

    pub fn has_emission(&self) -> bool {
        self.emission.norm() > EPSILON
    }

    pub fn get_color_at(&self, _u: f32, _v: f32) -> Vector3f {
        nalgebra::zero()
    }

    pub fn sample(&self, _wi: &Vector3f, n: &Vector3f) -> Vector3f {
        // uniform sample on the hemisphere
        assert_eq!(self.r#type, MaterialType::DiffuseAndGlossy);
        let x1 = get_random_float();
        let x2 = get_random_float();

        let z = (1f32 - 2f32 * x1).abs();
        let r = (1f32 - z * z).sqrt();
        let phi = 2f32 * PI * x2;
        let local_ray = vector3! {r* phi.cos(), r*phi.sin(), z};
        to_world(&local_ray, n)
    }

    pub fn pdf(&self, _wi: &Vector3f, wo: &Vector3f, n: &Vector3f) -> f32 {
        assert_eq!(self.r#type, MaterialType::DiffuseAndGlossy);

        if wo.dot(n) > 0f32 {
            0.5 / PI
        } else {
            0f32
        }
    }

    pub fn eval(&self, _wi: &Vector3f, wo: &Vector3f, n: &Vector3f) -> Vector3f {
        assert_eq!(self.r#type, MaterialType::DiffuseAndGlossy);

        if n.dot(wo) > 0f32 {
            self.kd / PI
        } else {
            nalgebra::zero()
        }
    }
}

fn to_world(a: &Vector3f, n: &Vector3f) -> Vector3f {
    // 重建坐标系
    let c = if n.x.abs() > n.y.abs() {
        let inv_len = 1f32 / (n.x * n.x + n.z * n.z).sqrt();
        vector3! {n.z * inv_len, 0f32, -n.x * inv_len}
    } else {
        let inv_len = 1f32 / (n.y * n.y + n.z * n.z).sqrt();
        vector3! {0f32, n.z*inv_len, -n.y * inv_len}
    };
    let b = c.cross(n);
    a.x * b + a.y * c + a.z * n
}
