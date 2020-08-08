use super::*;
use std::io::*;

pub struct HitPayload {
    pub t_near: f32,
    pub index: usize,
    // pub uv: Vector2f,
    pub hit_obj: Rc<dyn Object>,
}

fn deg2rad(deg: &f32) -> f32 {
    deg * PI / 180f32
}

fn shade(p: &Vector3f, wo: &Vector3f, n: &Vector3f, m: Rc<Material>, scene: &Scene) -> Vector3f {
    let mut inter = Intersection::default();
    let mut pdf_light = 0f32;
    scene.sample_light(&mut inter, &mut pdf_light);
    let x = inter.coords;

    let ws = (x - p).normalize();
    let nn = inter.normal;
    let emit = inter.emit / (x - p).norm_squared();
    let new_ray = Ray::new(p.clone(), x - p, 0f32);
    let inter = scene.intersect(&new_ray);

    let l_dir = if3!(
        inter.happened && inter.distance > 1f32 - EPSILON,
        wise_product(&emit, &m.eval(wo, &ws, n)) * ws.dot(n) * -ws.dot(&nn) / (pdf_light + EPSILON),
        nalgebra::zero()
    );
    let l_indir = if3!(
        get_random_float() < scene.russian_roulette,
        nalgebra::zero(),
        {
            let wi = m.sample(wo, n);
            let new_ray = Ray::new(p.clone(), wi, 0f32);
            let inter = scene.intersect(&new_ray);
            if !inter.happened || inter.obj.unwrap().has_emit() {
                nalgebra::zero()
            } else {
                wise_product(
                    &shade(
                        &inter.coords,
                        &new_ray.direction,
                        &inter.normal,
                        Rc::clone(&inter.m.unwrap()),
                        scene,
                    ),
                    &m.eval(wo, &wi, n),
                ) * wi.dot(n)
                    / m.pdf(wo, &wi, n)
                    / scene.russian_roulette
            }
        }
    );

    m.emission + l_dir + l_indir
}

pub fn cast_ray(ray: &Ray, scene: &Scene, _depth: i32) -> Vector3f {
    let intersection = scene.intersect(ray);
    if !intersection.happened {
        return nalgebra::zero();
    }
    shade(
        &intersection.coords,
        &ray.direction,
        &intersection.normal,
        Rc::clone(&intersection.m.unwrap()),
        scene,
    )
}

pub fn trace(ray: &Ray, objects: &Vec<Rc<dyn Object>>) -> Option<HitPayload> {
    let mut t_near = INFINITY;
    let mut ret = None;
    for obj in objects {
        if let Some((t, index)) = obj.intersect(ray) {
            if t < t_near {
                t_near = t;
                ret = Some(HitPayload {
                    t_near,
                    index,
                    hit_obj: Rc::clone(obj),
                });
            }
        }
    }
    ret
}

fn wise_product(a: &Vector3f, b: &Vector3f) -> Vector3f {
    Vector3f::new(a.x * b.x, a.y * b.y, a.z * b.z)
}

fn _reflect(input: &Vector3f, normal: &Vector3f) -> Vector3f {
    input - 2f32 * input.dot(normal) * normal
}

fn _refract(input: &Vector3f, normal: &Vector3f, ior: &f32) -> Vector3f {
    let mut cosi = clamp(-1f32, 1f32, input.dot(normal));
    let (mut etai, mut etat) = (1f32, *ior);

    let mut normal = normal.clone();
    if cosi < 0f32 {
        cosi = -cosi;
    } else {
        std::mem::swap(&mut etai, &mut etat);
        normal = -normal;
    }
    let eta = etai / etat;
    let k = 1f32 - eta * eta * (1f32 - cosi * cosi);
    if k < 0f32 {
        nalgebra::zero()
    } else {
        eta * input + (eta * cosi - k.sqrt()) * normal
    }
}

pub fn fresnel(input: &Vector3f, normal: &Vector3f, ior: &f32) -> f32 {
    let cosi = clamp(-1f32, 1f32, input.dot(normal));
    let (etai, etat) = if cosi > 0f32 {
        (*ior, 1f32)
    } else {
        (1f32, *ior)
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

pub fn render(scene: &Scene) -> std::io::Result<()> {
    let mut framebuffer: Vec<Vector3f> = vec![nalgebra::zero(); scene.width * scene.height];
    let scale = deg2rad(&(scene.fov * 0.5)).tan();
    let aspect_ratio = scene.width as f32 / scene.height as f32;
    let (inverse_width, inverse_height) = (1f32 / scene.width as f32, 1f32 / scene.height as f32);

    // 默认屏幕距离为 1
    let eye_pos = Vector3f::new(278f32, 273f32, -800f32);
    let mut m = 0;

    let spp = 16f32;
    println!("SPP: {}", spp);
    for j in 0..scene.height {
        for i in 0..scene.width {
            let x = (2f32 * (i as f32 + 0.5) * inverse_width - 1f32) * scale * aspect_ratio;
            let y = (1f32 - 2f32 * (j as f32 + 0.5) * inverse_height) * scale;
            let dir = Vector3f::new(-x, y, 1f32).normalize();
            for _ in 0..spp as usize {
                framebuffer[m] += cast_ray(&Ray::new(eye_pos, dir, 0f32), scene, 0) / spp;
            }
            m += 1;
        }
        update_progress(j as f32 / scene.height as f32);
    }
    update_progress(1f32);

    let mut fp = std::fs::File::create("binary.ppm")?;
    fp.write(&format!("P6\n{} {}\n255\n", scene.width, scene.height).as_bytes())?;
    for i in 0..scene.width * scene.height {
        let mut color = [0; 3];
        color[0] = (255f32 * clamp(0f32, 1f32, framebuffer[i].x).powf(0.6)) as u8;
        color[1] = (255f32 * clamp(0f32, 1f32, framebuffer[i].y).powf(0.6)) as u8;
        color[2] = (255f32 * clamp(0f32, 1f32, framebuffer[i].z).powf(0.6)) as u8;
        fp.write(&color)?;
    }
    Ok(())
}
