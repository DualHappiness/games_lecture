use nalgebra::Vector3;
use std::f32::INFINITY;

pub mod sphere;

type Vector3f = Vector3<f32>;

const MAX_DEPTH: i32 = 5;
pub fn mix(a: f32, b: f32, mix: f32) -> f32 {
    b * mix + a * (1f32 - mix)
}

pub fn trace(
    rayorig: Vector3f,
    raydir: Vector3f,
    spheres: &Vec<sphere::Sphere>,
    depth: i32,
) -> Vector3f {
    assert_eq!(raydir.len(), 1f32);
    let tnear = INFINITY;

    match spheres
        .iter()
        .map(|sphere| match sphere.intersect(&rayorig, &raydir) {
            None => (INFINITY, sphere),
            Some((t0, t1)) => (if t0 < 0f32 { t1 } else { t0 }, sphere),
        })
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
    {
        None => nalgebra::zero(),
        Some((t, sphere)) => {
            let mut surface_color = nalgebra::zero();
            let point_of_hit = rayorig + raydir * t;
            let mut normal_at_hit = point_of_hit - sphere.center;
            normal_at_hit.normalize();

            // If the normal and the view direction are not opposite to each other
            // reverse the normal direction. That also means we are inside the sphere so set
            // the inside bool to true. Finally reverse the sign of IdotN which we want
            // positive.
            let bias = 1e-4; // add some bias to the point from which we will be tracing
            let inside = raydir.dot(&normal_at_hit) > 0f32;
            if inside {
                normal_at_hit = -normal_at_hit;
            }

            if (sphere.transparency > 0f32 || sphere.reflection > 0f32) && depth < MAX_DEPTH {
                // change the mix value to tweak the effect
                let facingratio = -raydir.dot(&normal_at_hit);
                let fresneleffect = mix((1f32 - facingratio).powf(3f32), 1f32, 0.1);

                // compute reflection direction (not need to normalize because all vectors
                // are already normalized)
                let reflection_dir = raydir - normal_at_hit * 2f32 * raydir.dot(&normal_at_hit);
                reflection_dir.normalize();
                let reflection = trace(
                    point_of_hit + normal_at_hit * bias,
                    reflection_dir,
                    spheres,
                    depth + 1,
                );
                let refreaction = nalgebra::zero();
            }

            surface_color
        }
    }
}
