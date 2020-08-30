use super::*;

#[derive(Debug)]
pub struct Bound3 {
    pub p_min: Vector3f,
    pub p_max: Vector3f,
}

impl Default for Bound3 {
    fn default() -> Self {
        Self {
            p_min: Vector3f::from_element(f32::MAX),
            p_max: Vector3f::from_element(f32::MIN),
        }
    }
}

impl Clone for Bound3 {
    fn clone(&self) -> Self {
        Self {
            p_min: self.p_min,
            p_max: self.p_max,
        }
    }
}

impl Copy for Bound3 {}

pub fn union(b1: &Bound3, b2: &Bound3) -> Bound3 {
    Bound3::new(v3min(&b1.p_min, &b2.p_min), v3max(&b1.p_max, &b2.p_max))
}

pub fn union_point(b1: &Bound3, p: &Vector3f) -> Bound3 {
    Bound3::new(v3min(&b1.p_min, p), v3max(&b1.p_max, p))
}

impl Bound3 {
    pub fn new(p1: Vector3f, p2: Vector3f) -> Self {
        Self {
            p_min: v3min(&p1, &p2),
            p_max: v3max(&p1, &p2),
        }
    }

    pub fn diagonal(&self) -> Vector3f {
        self.p_max - self.p_min
    }

    pub fn max_extent(&self) -> i32 {
        let d = self.diagonal();
        if d.x > d.y && d.x > d.z {
            0
        } else if d.y > d.z {
            1
        } else {
            2
        }
    }

    pub fn surface_area(&self) -> f32 {
        let d = self.diagonal();
        2f32 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }

    pub fn centroid(&self) -> Vector3f {
        0.5 * self.p_min + 0.5 * self.p_max
    }

    pub fn intersect(&self, other: &Self) -> Self {
        Self::new(
            v3min(&self.p_min, &other.p_min),
            v3max(&self.p_max, &other.p_max),
        )
    }

    pub fn offset(&self, p: &Vector3f) -> Vector3f {
        let mut o = p - self.p_min;
        if self.p_max.x > self.p_min.x {
            o.x /= self.p_max.x - self.p_min.x;
        }
        if self.p_max.y > self.p_min.y {
            o.y /= self.p_max.y - self.p_min.y;
        }
        if self.p_max.z > self.p_max.z {
            o.z /= self.p_max.z - self.p_min.z;
        }
        o
    }

    pub fn overlaps(b1: &Self, b2: &Self) -> bool {
        let x = (b1.p_max.x >= b2.p_min.x) && (b1.p_min.x <= b2.p_max.x);
        let y = (b1.p_max.y >= b2.p_min.y) && (b1.p_min.y <= b2.p_max.y);
        let z = (b1.p_max.z >= b2.p_min.z) && (b1.p_min.z <= b2.p_max.z);
        x && y && z
    }

    pub fn inside(p: &Vector3f, b: &Self) -> bool {
        p.x >= b.p_min.x
            && p.x <= b.p_max.x
            && p.y >= b.p_min.y
            && p.y <= b.p_max.y
            && p.z >= b.p_min.z
            && p.z <= b.p_max.z
    }

    pub fn intersect_ray(&self, ray: &Ray, inv_dir: &Vector3f, dir_is_neg: &[bool]) -> bool {
        let (t_enter, t_exit) = (0..3).fold((f32::MIN, f32::MAX), |(t_min, t_max), i| {
            let t1 = (self.p_min[i] - ray.origin[i]) * inv_dir[i];
            let t2 = (self.p_max[i] - ray.origin[i]) * inv_dir[i];

            let (t1, t2) = if !dir_is_neg[i] { (t2, t1) } else { (t1, t2) };
            (t_min.max(t1), t_max.min(t2))
        });
        #[cfg(feature = "show_print")]
        println!("bound box is {:?}", self);
        t_exit > 0f32 && t_enter <= t_exit
    }
}
