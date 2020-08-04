use super::*;
use std::io::*;

fn deg2rad(deg: &f32) -> f32 {
    deg * PI / 180f32
}

fn reflect(input: &Vector3f, normal: &Vector3f) -> Vector3f {
    input - 2f32 * input.dot(normal) * normal
}

fn refract(input: &Vector3f, normal: &Vector3f, ior: &f32) -> Vector3f {
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

fn trace(
    origin: &Vector3f,
    dir: &Vector3f,
    objects: &Vec<Rc<RefCell<dyn Object>>>,
) -> Option<HitPayload> {
    let mut t_near = INFINITY;
    let mut ret = None;
    for obj in objects {
        if let Some((t, index, uv)) = obj.borrow().intersect(origin, dir) {
            if t < t_near {
                t_near = t;
                ret = Some(HitPayload {
                    t_near,
                    index,
                    uv,
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
pub fn cast_ray(origin: &Vector3f, dir: &Vector3f, scene: &Scene, depth: i32) -> Vector3f {
    if depth > scene.max_depth {
        return nalgebra::zero();
    }
    let mut hit_color = scene.background_color;
    if let Some(payload) = trace(origin, dir, scene.get_objs()) {
        let hit_point = origin + dir * payload.t_near;
        let (normal, st) = payload.hit_obj.borrow().get_surface_properties(
            &hit_point,
            dir,
            &payload.index,
            &payload.uv,
        );
        let f = |dir: Vector3f, normal| {
            let ray_origin = if dir.dot(normal) < 0f32 {
                hit_point - normal * scene.epsilon
            } else {
                hit_point + normal * scene.epsilon
            };
            cast_ray(&ray_origin, &dir, scene, depth + 1)
        };
        hit_color = match payload.hit_obj.borrow().material_type {
            MaterialType::ReflectionAndRefraction => {
                let reflection_color = f(reflect(dir, &normal).normalize(), &normal);

                let refraction_color = f(
                    refract(dir, &normal, &payload.hit_obj.borrow().ior).normalize(),
                    &normal,
                );
                let kr = fresnel(dir, &normal, &payload.hit_obj.borrow().ior);
                reflection_color * kr + refraction_color * (1f32 - kr)
            }
            MaterialType::Reflection => {
                let kr = fresnel(dir, &normal, &payload.hit_obj.borrow().ior);
                let color = f(reflect(dir, &normal), &normal);
                color * kr
            }
            MaterialType::DiffuseAndGlossy => {
                let mut light_amt: Vector3f = nalgebra::zero();
                let mut specular_color: Vector3f = nalgebra::zero();
                let shadow_point_origin = if normal.dot(dir) < 0f32 {
                    hit_point + normal * scene.epsilon
                } else {
                    hit_point - normal * scene.epsilon
                };

                for light in scene.get_lights() {
                    let l = light.position - hit_point;
                    let dis2 = l.dot(&l);
                    let l = l.normalize();

                    light_amt += match trace(&shadow_point_origin, &l, scene.get_objs()) {
                        Some(shadow_res) if shadow_res.t_near * shadow_res.t_near < dis2 => {
                            nalgebra::zero()
                        }
                        _ => light.intensity * 0f32.max(l.dot(&normal)),
                    };

                    let reflection_dir = reflect(&-l, &normal);
                    specular_color += 0f32
                        .max(-1f32 * reflection_dir.dot(dir))
                        .powi(payload.hit_obj.borrow().specular_exponent)
                        * light.intensity
                }

                // println!("light amt : {}", light_amt);
                // println!(
                //     "diffuse color: {}",
                //     payload.hit_obj.borrow().eval_diffuse_color(&st)
                // );
                // println!("specular color: {}", specular_color);
                // println!(
                //     "kd: {}, ks: {}",
                //     payload.hit_obj.borrow().kd,
                //     payload.hit_obj.borrow().ks
                // );
                wise_product(
                    &light_amt,
                    &payload.hit_obj.borrow().eval_diffuse_color(&st),
                ) * payload.hit_obj.borrow().kd
                    + specular_color * payload.hit_obj.borrow().ks
            }
        }
    }
    hit_color
}

pub struct HitPayload {
    pub t_near: f32,
    pub index: usize,
    pub uv: Vector2f,
    pub hit_obj: Rc<RefCell<dyn Object>>,
}

pub fn render(scene: &Scene) -> std::io::Result<()> {
    let mut framebuffer = vec![nalgebra::zero(); scene.width * scene.height];
    let scale = deg2rad(&(scene.fov * 0.5)).tan();
    let aspect_ratio = scene.width as f32 / scene.height as f32;
    let (inverse_width, inverse_height) = (1f32 / scene.width as f32, 1f32 / scene.height as f32);

    // 默认屏幕距离为 1
    let eye_pos = nalgebra::zero();
    let mut m = 0;
    for j in 0..scene.height {
        for i in 0..scene.width {
            let x = (2f32 * (i as f32 + 0.5) * inverse_width - 1f32) * scale * aspect_ratio;
            let y = (2f32 * (scene.height as f32 - j as f32 + 0.5) * inverse_height - 1f32) * scale;
            let dir = Vector3f::new(x, y, -1f32).normalize();
            framebuffer[m] = cast_ray(&eye_pos, &dir, scene, 0);
            m += 1;
        }
        update_progress(j as f32 / scene.height as f32);
    }

    let mut fp = std::fs::File::create("binary.ppm")?;
    fp.write(&format!("P6\n{} {}\n255\n", scene.width, scene.height).as_bytes())?;
    for i in 0..scene.width * scene.height {
        let mut color = [0; 3];
        color[0] = (255f32 * clamp(0f32, 1f32, framebuffer[i].x)) as u8;
        color[1] = (255f32 * clamp(0f32, 1f32, framebuffer[i].y)) as u8;
        color[2] = (255f32 * clamp(0f32, 1f32, framebuffer[i].z)) as u8;
        fp.write(&color)?;
    }
    Ok(())
}
