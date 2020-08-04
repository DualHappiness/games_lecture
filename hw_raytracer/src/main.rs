#[macro_use]
use hw_raytracer::*;
use std::time::SystemTime;

fn main() {
    let mut scene = Scene::new(1280, 960);

    let bunny: Rc<RefCell<dyn Object>> = Rc::new(RefCell::new(triangle::MeshTriangle::new(
        "../models/bunny/bunny.obj",
    )));

    scene.add_obj(&bunny);
    scene.add_light(Light::new(
        Vector3f::new(-20f32, 70f32, 20f32),
        Vector3f::from_element(1f32),
    ));
    scene.add_light(Light::new(
        Vector3f::new(20f32, 70f32, 20f32),
        Vector3f::from_element(1f32),
    ));

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
