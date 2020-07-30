pub mod obj_loader;
pub mod rasterizer;
pub mod shader;
pub mod texture;
pub mod triangle;

use nalgebra::{Matrix3, Matrix4, Vector3, Vector4};

const MY_PI: f32 = 3.1415926;
const _TWO_PI: f32 = 2f32 * MY_PI;

type Vector3f = Vector3<f32>;
pub fn get_view_matrix(eye_pos: Vector3<f32>) -> Matrix4<f32> {
    let translate = Matrix4::from_columns(&[
        Vector4::x(),
        Vector4::y(),
        Vector4::z(),
        Vector4::new(-eye_pos[0], -eye_pos[1], -eye_pos[2], 1.0),
    ]);
    translate
}

pub fn get_model_matrix(rotation_angle: f32) -> Matrix4<f32> {
    let angle = rotation_angle / 180.0 * MY_PI;

    let rotation = Matrix4::from_columns(&[
        Vector4::new(angle.cos(), 0f32, -angle.sin(), 0.0),
        Vector4::y(),
        Vector4::new(angle.sin(), 0f32, angle.cos(), 0.0),
        Vector4::w(),
    ]);

    let scale = Matrix4::from_columns(&[
        Vector4::x() * 2.5,
        Vector4::y() * 2.5,
        Vector4::z() * 2.5,
        Vector4::w(),
    ]);

    rotation * scale
}

pub fn get_projection_matrix(
    eye_fov: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix4<f32> {
    let (n, f) = (z_near, z_far);
    let t = (eye_fov / 2.0 / 180.0 * MY_PI).tan() * n.abs();
    let r = t * aspect_ratio;

    let orthographic_t = Matrix4::from_columns(&[
        Vector4::x(),
        Vector4::y(),
        Vector4::z(),
        Vector4::new(0.0, 0.0, -(n + f) / 2.0, 1.0),
    ]);
    let orthographic_s = Matrix4::from_columns(&[
        Vector4::new(1f32 / r, 0f32, 0f32, 0f32),
        Vector4::new(0f32, 1f32 / t, 0f32, 0f32),
        Vector4::new(0f32, 0f32, 2f32 / (n - f), 0f32),
        Vector4::w(),
    ]);

    let orthographic = orthographic_s * orthographic_t;

    let perspective_to_orthographic = Matrix4::from_columns(&[
        Vector4::new(n, 0f32, 0f32, 0f32),
        Vector4::new(0f32, n, 0f32, 0f32),
        Vector4::new(0f32, 0f32, n + f, 1f32),
        Vector4::new(0f32, 0f32, -n * f, 0f32),
    ]);

    #[cfg(feature = "show_print")]
    {
        println!("s: {:?}, t: {:?}", orthographic_s, orthographic_t);
        println!("orthographic : {:?}", orthographic);
        println!("perspective_to_o: {:?}", perspective_to_orthographic);
        println!(
            "perspective: {:?}",
            orthographic * perspective_to_orthographic
        );
    }
    orthographic * perspective_to_orthographic
}

pub fn vertex_shader(payload: &shader::VertexShaderPayload) -> Vector3<f32> {
    payload.position
}

pub fn normal_fragment_shader(payload: &shader::FragmentShaderPayload) -> Vector3<f32> {
    let return_color = (payload.normal.normalize() + Vector3::from_element(1f32)) / 2f32;
    return_color * 255f32
}

pub fn reflect(vec: &Vector3<f32>, axis: &Vector3<f32>) -> Vector3<f32> {
    let costheta = vec.dot(axis);
    (2f32 * costheta * axis - vec).normalize()
}

pub struct Light {
    position: Vector3f,
    intensity: Vector3f,
}

fn scale(a: Vector3f, b: Vector3f) -> Vector3f {
    Vector3f::new(a.x * b.x, a.y * b.y, a.z * b.z)
}

fn blinn_phone_calc(
    ka: Vector3f,
    kd: Vector3f,
    ks: Vector3f,
    _color: Vector3f,
    point: Vector3f,
    normal: Vector3f,
) -> Vector3f {
    let l1: Light = Light {
        position: Vector3f::from_element(20f32),
        intensity: Vector3f::from_element(500f32),
    };
    let l2: Light = Light {
        position: Vector3f::new(-20f32, 20f32, 0f32),
        intensity: Vector3f::from_element(500f32),
    };
    let lights: Vec<Light> = vec![l1, l2];

    let ambient_light_intensity: Vector3f = Vector3f::from_element(10f32);

    let p: i32 = 150;

    let eye_pos: Vector3f = Vector3::new(0f32, 0f32, 10f32);

    let mut ret: Vector3f = nalgebra::zero();
    for light in lights {
        let l = light.position - point;
        let dis_sqrt = l.magnitude_squared();

        let l = l.normalize();
        let ld = scale(kd, (light.intensity / dis_sqrt) * 0f32.max(normal.dot(&l)));
        ret += ld;

        let v = (eye_pos - point).normalize();
        let h = (l + v) / (l + v).magnitude();
        let ls = scale(
            ks,
            (light.intensity / dis_sqrt) * 0f32.max(normal.dot(&h)).powi(p),
        );

        ret += ls;

        let la = scale(ka, ambient_light_intensity);
        ret += la;
    }

    ret * 255f32
}

pub fn texture_fragment_shader(payload: &shader::FragmentShaderPayload) -> Vector3f {
    let texture_color = match payload.texture {
        None => nalgebra::zero(),
        Some(texture) => texture.get_color_bilinear(payload.tex_coords[0], payload.tex_coords[1]),
    };

    let ka = Vector3f::from_element(0.005);
    let kd = texture_color / 255f32;
    let ks = Vector3f::from_element(0.7937);

    let color = texture_color;
    let point = payload.view_pos;
    let normal = payload.normal;

    blinn_phone_calc(ka, kd, ks, color, point, normal)
}

pub fn phone_fragment_shader(payload: &shader::FragmentShaderPayload) -> Vector3f {
    let ka = Vector3f::from_element(0.005);
    let kd = payload.color;
    let ks = Vector3f::from_element(0.7937);

    let color = payload.color;
    let point = payload.view_pos;
    let normal = payload.normal;

    blinn_phone_calc(ka, kd, ks, color, point, normal)
}

fn calc_bump_normal(payload: &shader::FragmentShaderPayload) -> Vector3f {
    let kh = 0.2;
    let kn = 0.1;

    let normal = payload.normal;
    let (x, y, z) = (normal.x, normal.y, normal.z);
    let t = Vector3f::new(
        x * y / (x * x + z * z).sqrt(),
        (x * x + z * z).sqrt(),
        z * y / (x * x + z * z).sqrt(),
    );
    let b = normal.cross(&t);

    let u = payload.tex_coords[0];
    let v = payload.tex_coords[1];
    let texture = payload.texture.unwrap();
    let h = |u, v| texture.get_color(u, v).magnitude();

    let tbn = Matrix3::from_columns(&[t, b, normal]);
    let du = kh * kn * (h(u + 1f32 / texture.width as f32, v) - h(u, v));
    let dv = kh * kn * (h(u, v + 1f32 / texture.height as f32) - h(u, v));
    let ln = Vector3f::new(-du, -dv, 1f32);

    (tbn * ln).normalize()
}

pub fn displacement_fragment_shader(payload: &shader::FragmentShaderPayload) -> Vector3f {
    let ka = Vector3f::from_element(0.005);
    let kd = payload.color;
    let ks = Vector3f::from_element(0.7937);

    let color = payload.color;
    let point = payload.view_pos;

    let normal = calc_bump_normal(payload);

    blinn_phone_calc(ka, kd, ks, color, point, normal)
}

pub fn bump_fragment_shader(payload: &shader::FragmentShaderPayload) -> Vector3f {
    let normal = calc_bump_normal(payload);
    normal * 255f32
}
