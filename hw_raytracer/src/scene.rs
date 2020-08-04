use super::*;



pub struct Scene {
    pub width: usize,
    pub height: usize,
    pub fov: f32,
    pub background_color: Vector3f,
    pub max_depth: i32,
    pub epsilon: f32,
    objects: Vec<Rc<RefCell<dyn Object>>>,
    lights: Vec<Light>,
    bvh: Option<BVHAccel>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 960,
            fov: 90f32,
            background_color: Vector3f::new(0.23529, 0.67451, 0.843137),
            max_depth: 5,
            epsilon: 0.00001,
            objects: vec![],
            lights: vec![],
            bvh: None,
        }
    }
}

impl Scene {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    pub fn add_obj(&mut self, obj: &Rc<RefCell<dyn Object>>) {
        self.objects.push(Rc::clone(obj));
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn get_objs(&self) -> &Vec<Rc<RefCell<dyn Object>>> {
        &self.objects
    }

    pub fn get_lights(&self) -> &Vec<Light> {
        &self.lights
    }

    pub fn build_bvh(&mut self) {
        println!(" - Generating BVH...\n\n");
        self.bvh = Some(BVHAccel::new(&self.objects, 1, SplitMethod::NAIVE));
    }

    pub fn intersect(&self, ray: &Ray) -> Intersection {
        match &self.bvh {
            None => Default::default(),
            Some(bvh) => bvh.intersect(ray),
        }
    }
}

