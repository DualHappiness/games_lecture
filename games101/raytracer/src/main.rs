use nalgebra::Vector3;
use raytracer::*;
type Vector3f = Vector3<f32>;

fn main() {
    let mut spheres = vec![];
    spheres.push(new_sphere!(
        Vector3f::new(0f32, -10004f32, -20f32),
        10000f32,
        Vector3f::new(0.2, 0.2, 0.2)
    ));
    spheres.push(new_sphere!(
        Vector3f::new(0f32, 0f32, -20f32),
        4f32,
        Vector3f::new(1f32, 0.32, 0.36),
        nalgebra::zero(),
        1f32,
        0.5
    ));
    spheres.push(new_sphere!(
        Vector3f::new(5f32, -1f32, -15f32),
        2f32,
        Vector3f::new(0.9, 0.76, 0.46),
        nalgebra::zero(),
        1f32
    ));
    spheres.push(new_sphere!(
        Vector3f::new(5f32, 0f32, -25f32),
        3f32,
        Vector3f::new(0.65, 0.77, 0.97),
        nalgebra::zero(),
        1f32
    ));
    spheres.push(new_sphere!(
        Vector3f::new(-5.5, 0f32, -15f32),
        3f32,
        Vector3f::new(0.9, 0.9, 0.9),
        nalgebra::zero(),
        1f32
    ));

    // light
    spheres.push(new_sphere!(
        Vector3f::new(0f32, 20f32, -30f32),
        3f32,
        nalgebra::zero(),
        Vector3f::from_element(3f32)
    ));
    render(&spheres).expect("render err");
}
