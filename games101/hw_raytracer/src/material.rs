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

    pub fn pdf(&self, wi: &Vector3f, wo: &Vector3f, n: &Vector3f) -> f32 {
        assert_eq!(self.r#type, MaterialType::DiffuseAndGlossy);

        if wo.dot(n) > 0f32 {
            f(&-wi, wo, n)
        } else {
            0f32
        }
    }

    pub fn eval(&self, wi: &Vector3f, wo: &Vector3f, n: &Vector3f) -> Vector3f {
        assert_eq!(self.r#type, MaterialType::DiffuseAndGlossy);

        if n.dot(wo) > 0f32 {
            self.kd * f(&-wi, wo, n) * 2f32
        } else {
            nalgebra::zero()
        }
    }
}

fn f(wi: &Vector3f, wo: &Vector3f, n: &Vector3f) -> f32 {
    let h = (wi + wo).normalize();
    let f = fresnel(wi, &h, 0.2);
    let g = g(wi, wo, &h, n);
    let d = distribute(&h, n);

    f * g * d / (4f32 * n.dot(wi) * n.dot(wo))
}

fn distribute(h: &Vector3f, n: &Vector3f) -> f32 {
    let costheta = h.dot(n);
    let tan2 = (1f32 - costheta * costheta) / (costheta * costheta);
    let cos2 = costheta * costheta;
    let alpha = 0.5;

    let root = alpha / (cos2 * (alpha * alpha + tan2));
    INV_PI * root * root
}

fn fresnel(i: &Vector3f, h: &Vector3f, ior: f32) -> f32 {
    let cosi = clamp(-1f32, 1f32, i.dot(h));
    let (etai, etat) = if cosi > 0f32 {
        (ior, 1f32)
    } else {
        (1f32, ior)
    };
    let sint = etai / etat * 0f32.max(1f32 - cosi * cosi).sqrt();
    if sint > 1f32 {
        1f32
    } else {
        let cost = 0f32.max(1f32 - sint * sint).sqrt();
        let cosi = cosi.abs();
        let rs = (etat * cosi - etai * cost) / (etat * cosi + etai * cost);
        let rp = (etai * cosi - etat * cost) / (etai * cosi + etat * cost);
        (rs * rs + rp * rp) / 2f32
    }
}

fn g(i: &Vector3f, o: &Vector3f, h: &Vector3f, n: &Vector3f) -> f32 {
    let local_g = |v: &Vector3f, h| {
        let cos = v.dot(n);
        let tan = (1f32 - cos * cos).sqrt() / cos;
        if tan == 0f32 {
            return 1f32;
        }
        if v.dot(h) * cos <= 0f32 {
            return 0f32;
        }

        let alpha = 0.5;
        let root = alpha * tan;
        2f32 / (1f32 + (1f32 + root * root).sqrt())
    };
    local_g(o, h) * local_g(i, h)
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
