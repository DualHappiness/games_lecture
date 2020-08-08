use hw_raytracer::*;
use std::time::SystemTime;

type RObj = Rc<dyn Object>;
fn main() {
    let mut scene = Scene::new(784, 784);

    let mut red = Material::default();
    red.kd = vector3! {0.63, 0.065, 0.05};
    let mut green = Material::default();
    green.kd = vector3! {0.14, 0.45,0.091};
    let mut white = Material::default();
    white.kd = vector3! {0.725, 0.71, 0.68};

    let mut light = Material::default();
    let a = 0.747;
    let b = 0.740;
    let c = 0.737;
    light.emission = 8f32 * vector3! {a+0.058, a+0.258, a}
        + 15.6f32 * vector3! {b + 0.287, b + 0.16 , b}
        + 18.4f32 * vector3! {c + 0.642, c + 0.159, c};
    light.kd = Vector3f::from_element(0.65);

    let path = "models/cornellbox/".to_owned();
    let new_obj = |path: String, material| {
        Rc::new(triangle::MeshTriangle::new(&path, material))
    };
    let floor: RObj = new_obj(path.clone() + "floor.obj", white);
    let shortbox: RObj = new_obj(path.clone() + "shortbox.obj", white);
    let tallbox: RObj = new_obj(path.clone() + "tallbox.obj", white);
    let left: RObj = new_obj(path.clone() + "left.obj", red);
    let right: RObj = new_obj(path.clone() + "right.obj", green);
    let light: RObj = new_obj(path.clone() + "light.obj", light);

    println!("light size is {}", light.get_area());
    scene.add_obj(&floor);
    scene.add_obj(&shortbox);
    scene.add_obj(&tallbox);
    scene.add_obj(&left);
    scene.add_obj(&right);
    scene.add_obj(&light);

    scene.build_bvh();

    let start = SystemTime::now();
    render(&scene).expect("render error");
    let elapsed = start.elapsed().unwrap();

    let hrs = elapsed.as_secs() / 3600;
    let mins = elapsed.as_secs() / 60 - hrs * 60;
    let secs = elapsed.as_secs() - mins * 60 - hrs * 3600;
    println!("Render complete: ");
    println!("Time taken: {} hrs, {} mins, {} secs", hrs, mins, secs);
}
