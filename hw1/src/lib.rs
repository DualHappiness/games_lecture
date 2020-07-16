pub mod rasterizer;
pub mod triangle;

use nalgebra::{Matrix4, Vector3, Vector4};

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
    let angle = rotation_angle / 180.0 * std::f32::consts::PI;

    let translate = Matrix4::from_columns(&[
        Vector4::new(angle.cos(), angle.sin(), 0.0, 0.0),
        Vector4::new(-angle.sin(), angle.cos(), 0.0, 0.0),
        Vector4::z(),
        Vector4::w(),
    ]);

    translate
}

pub fn get_projection_matrix(
    eye_fov: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix4<f32> {
    let (n, f) = (z_near, z_far);
    println!("{}", -1f32.atan());
    let t = (eye_fov / 2.0 / 180.0 * std::f32::consts::PI).tan() * n.abs();
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
        Vector4::new(0f32, 0f32, n + f, 0f32),
        Vector4::new(0f32, 0f32, -n * f, 1f32),
    ]);

    println!("orthographic : {:?}", orthographic);
    println!("perspective_to_o: {:?}", perspective_to_orthographic);
    orthographic * perspective_to_orthographic
}
