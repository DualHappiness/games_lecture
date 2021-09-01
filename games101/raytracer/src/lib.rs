use nalgebra::Vector3;
use opencv::{core, imgcodecs, prelude::*};
use std::f32::INFINITY;
use std::fs;
use std::io::prelude::*;

pub mod sphere;

type Vector3f = Vector3<f32>;

const MAX_DEPTH: i32 = 2;
fn mix(a: f32, b: f32, mix: f32) -> f32 {
    b * mix + a * (1f32 - mix)
}

fn mul_color(a: &Vector3f, b: &Vector3f) -> Vector3f {
    Vector3f::new(a.x * b.x, a.y * b.y, a.z * b.z)
}

fn calc_reflection(
    spheres: &Vec<sphere::Sphere>,
    raydir: Vector3f,
    point_of_hit: Vector3f,
    normal_at_hit: Vector3f,
    bias: f32,
    depth: i32,
) -> Vector3f {
    // compute reflection direction (not need to normalize because all vectors
    // are already normalized)
    let reflection_dir = raydir - normal_at_hit * 2f32 * raydir.dot(&normal_at_hit);
    let reflection_dir = reflection_dir.normalize();
    trace(
        point_of_hit + normal_at_hit * bias,
        reflection_dir,
        spheres,
        depth + 1,
    )
}

fn calc_refrection(
    sphere: &sphere::Sphere,
    spheres: &Vec<sphere::Sphere>,
    raydir: Vector3f,
    inside: bool,
    point_of_hit: Vector3f,
    normal_at_hit: Vector3f,
    bias: f32,
    depth: i32,
) -> Vector3f {
    if sphere.transparency > 0f32 {
        // fixed index of refraction
        let ior = 1.1;
        let eta = if inside { ior } else { 1f32 / ior };
        let cosi = -normal_at_hit.dot(&raydir);
        let k = 1f32 - eta * eta * (1f32 - cosi * cosi);
        let refraction_dir = raydir * eta + normal_at_hit * (eta * cosi - k.sqrt());
        let refraction_dir = refraction_dir.normalize();

        trace(
            point_of_hit - normal_at_hit * bias,
            refraction_dir,
            spheres,
            depth + 1,
        )
    } else {
        nalgebra::zero()
    }
}

fn calc_non_diffuse(
    sphere: &sphere::Sphere,
    spheres: &Vec<sphere::Sphere>,
    raydir: Vector3f,
    point_of_hit: Vector3f,
    normal_at_hit: Vector3f,
    inside: bool,
    bias: f32,
    depth: i32,
) -> Vector3f {
    let reflection = calc_reflection(spheres, raydir, point_of_hit, normal_at_hit, bias, depth);
    // compute refreaction
    let refreaction = calc_refrection(
        sphere,
        spheres,
        raydir,
        inside,
        point_of_hit,
        normal_at_hit,
        bias,
        depth,
    );
    // the result is a mix of reflection and refraction (if the sphere is transparent)
    // change the mix value to tweak the effect
    let facingratio = -raydir.dot(&normal_at_hit);
    let fresneleffect = mix((1f32 - facingratio).powf(3f32), 1f32, 0.1);

    mul_color(
        &(reflection * fresneleffect + refreaction * (1f32 - fresneleffect) * sphere.transparency),
        &sphere.surface_color,
    )
}

fn calc_diffuse(
    sphere: &sphere::Sphere,
    spheres: &Vec<sphere::Sphere>,
    point_of_hit: Vector3f,
    normal_at_hit: Vector3f,
    bias: f32,
) -> Vector3f {
    // if it's a diffuse object, no need to raytrace anymore
    spheres
        .iter()
        .filter(|s| s.emission_color.x > 0f32)
        .map(|light_sphere| {
            let light_dirction = light_sphere.center - point_of_hit;
            let light_dirction = light_dirction.normalize();

            if spheres
                .iter()
                .filter(|&s| !std::ptr::eq(s, light_sphere))
                .any(|s2| {
                    // because intersection is check dot so self will not intersect
                    s2.intersect(&(point_of_hit + normal_at_hit * bias), &light_dirction)
                        .is_some()
                })
            {
                nalgebra::zero()
            } else {
                mul_color(
                    &(sphere.surface_color * 0f32.max(normal_at_hit.dot(&light_dirction))),
                    &light_sphere.emission_color,
                )
            }
        })
        .sum()
}

fn calc_surface_color(
    sphere: &sphere::Sphere,
    spheres: &Vec<sphere::Sphere>,
    rayorig: Vector3f,
    raydir: Vector3f,
    t: f32,
    depth: i32,
) -> Vector3f {
    let point_of_hit = rayorig + raydir * t;
    let normal_at_hit = point_of_hit - sphere.center;
    let mut normal_at_hit = normal_at_hit.normalize();

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
        calc_non_diffuse(
            sphere,
            spheres,
            raydir,
            point_of_hit,
            normal_at_hit,
            inside,
            bias,
            depth,
        )
    } else {
        calc_diffuse(sphere, spheres, point_of_hit, normal_at_hit, bias)
    }
}

fn trace(
    rayorig: Vector3f,
    raydir: Vector3f,
    spheres: &Vec<sphere::Sphere>,
    depth: i32,
) -> Vector3f {
    // assert_eq!(raydir.len(), 1f32);
    match spheres
        .iter()
        .filter_map(|sphere| match sphere.intersect(&rayorig, &raydir) {
            None => None,
            Some((t0, t1)) => Some((if t0 < 0f32 { t1 } else { t0 }, sphere)),
        })
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
    {
        None => Vector3f::from_element(2f32),
        Some((t, sphere)) => {
            let surface_color = calc_surface_color(sphere, spheres, rayorig, raydir, t, depth);
            surface_color + sphere.emission_color
        }
    }
}

pub fn render(spheres: &Vec<sphere::Sphere>) -> std::io::Result<()> {
    let (width, height) = (640, 480);
    let mut image = vec![nalgebra::zero(); width * height];

    let (inv_width, inv_height) = (1f32 / width as f32, 1f32 / height as f32);
    let fov = 30f32;
    let aspect_ratio = width as f32 / height as f32;

    let angle = (std::f32::consts::PI * 0.5 * fov / 180f32).tan();

    // trace trays
    let mut pixel_index = 0;
    for y in 0..height {
        for x in 0..width {
            let xx = (2f32 * ((x as f32 + 0.5) * inv_width) - 1f32) * angle * aspect_ratio;
            let yy = (1f32 - 2f32 * ((y as f32 + 0.5) * inv_height)) * angle;

            let raydir = Vector3f::new(xx, yy, -1f32);
            let raydir = raydir.normalize();

            image[pixel_index] = trace(nalgebra::zero(), raydir, spheres, 0);
            pixel_index += 1;
        }
    }

    // save result to a PPM image
    let mut file = fs::File::create("./untitled.ppm")?;
    file.write(format!("P6\n{} {}\n255\n", width, height).as_bytes())?;
    image.iter().for_each(|pixel| {
        file.write(&[
            (1f32.min(pixel.x) * 255f32) as u8,
            (1f32.min(pixel.y) * 255f32) as u8,
            (1f32.min(pixel.z) * 255f32) as u8,
        ])
        .unwrap();
    });

    // let mut buffer = Vec::with_capacity(width as usize * height as usize);
    // image.iter().for_each(|pixel| {
    //     buffer.push(1f32.min(pixel.x) * 255f32);
    //     buffer.push(1f32.min(pixel.y) * 255f32);
    //     buffer.push(1f32.min(pixel.z) * 255f32);
    // });
    // let ptr = buffer.as_mut_ptr() as *mut std::ffi::c_void;
    // let mut ret = Mat::default().unwrap();
    // unsafe {
    //     let image = Mat::new_rows_cols_with_data(
    //         width as i32,
    //         height as i32,
    //         core::CV_32FC3,
    //         ptr,
    //         core::Mat_AUTO_STEP,
    //     )
    //     .expect("build image fail");
    //     image
    //         .convert_to(&mut ret, core::CV_8UC3, 1f64, 0f64)
    //         .expect("convert err");
    // }
    // imgcodecs::imwrite("output.png", &ret, &core::Vector::new()).unwrap();
    Ok(())
}
