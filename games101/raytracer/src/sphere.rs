use nalgebra::Vector3;

type Vector3f = Vector3<f32>;

#[derive(Debug, Default)]
pub struct Sphere {
    pub center: Vector3f,
    pub radius: f32,
    pub radius2: f32,
    pub surface_color: Vector3f,
    pub transparency: f32,
    pub reflection: f32,
    pub emission_color: Vector3f,
}

#[macro_export]
macro_rules! new_sphere {
    ($a:expr, $b:expr, $c:expr) => {
        new_sphere!($a, $b, $c, nalgebra::zero());
    };
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr) => {
        raytracer::sphere::Sphere::new($a, $b, $c, $d, $e, $f)
    };
    ($($a:expr),*) => {
        new_sphere!($($a),*, 0f32);
    };
}

impl Sphere {
    pub fn new(
        center: Vector3f,
        radius: f32,
        surface_color: Vector3f,
        emission_color: Vector3f,
        reflection: f32,
        transparency: f32,
    ) -> Self {
        Sphere {
            center,
            radius,
            surface_color,
            emission_color,
            transparency,
            reflection,
            radius2: radius * radius,
        }
    }

    pub fn intersect(&self, rayorig: &Vector3f, raydir: &Vector3f) -> Option<(f32, f32)> {
        let l = self.center - rayorig;
        let tca = l.dot(raydir);
        if tca < 0f32 {
            return None;
        }

        let d2 = l.dot(&l) - tca * tca;
        if d2 > self.radius2 {
            return None;
        }

        let thc = (self.radius2 - d2).sqrt();

        Some((tca - thc, tca + thc))
    }
}
