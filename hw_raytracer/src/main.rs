#[macro_use]
use hw_raytracer::*;

fn main() {
    let mut scene = Scene::new(1280, 960);

    let sphere1: Rc<RefCell<dyn Object>> = Rc::new(RefCell::new(sphere::Sphere::new(
        Vector3f::new(-1f32, 0f32, -12f32),
        2f32,
    )));
    sphere1.borrow_mut().material_type = MaterialType::DiffuseAndGlossy;
    sphere1.borrow_mut().diffuse_color = Vector3f::new(0.6, 0.7, 0.8);

    let sphere2: Rc<RefCell<dyn Object>> = Rc::new(RefCell::new(sphere::Sphere::new(
        Vector3f::new(0.5, -0.5, -8f32),
        1.5,
    )));
    sphere2.borrow_mut().ior = 1.5;
    sphere2.borrow_mut().material_type = MaterialType::ReflectionAndRefraction;

    scene.add_obj(&sphere1);
    scene.add_obj(&sphere2);

    let verts = vec![
        vector3! {-5f32, -3f32, -6f32},
        vector3! {5f32, -3f32, -6f32},
        vector3! {5f32, -3f32, -16f32},
        vector3! {-5f32, -3f32, -16f32},
    ];
    let verts_index = vec![0, 1, 3, 1, 2, 3];
    let st = vec![
        vector2! {0f32, 0f32},
        vector2! {1f32, 0f32},
        vector2! {1f32, 1f32},
        vector2! {0f32, 1f32},
    ];
    let mesh: Rc<RefCell<dyn Object>> = Rc::new(RefCell::new(triangle::MeshTriangle::new(
        &verts,
        &verts_index,
        2,
        &st,
    )));
    scene.add_obj(&mesh);

    scene.add_light(Light::new(
        Vector3f::new(-20f32, 70f32, 20f32),
        Vector3f::from_element(0.5),
    ));
    scene.add_light(Light::new(
        Vector3f::new(30f32, 50f32, -12f32),
        Vector3f::from_element(0.5),
    ));

    render(&scene).expect("render error");
}
