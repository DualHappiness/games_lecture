use super::*;

pub struct Scene {
    pub width: usize,
    pub height: usize,
    pub fov: f32,
    pub background_color: Vector3f,
    pub max_depth: i32,
    pub epsilon: f32,
    pub russian_roulette: f32,
    objects: Vec<Rc<dyn Object>>,
    lights: Vec<Light>,
    bvh: Option<Accel>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 960,
            fov: 40f32,
            background_color: Vector3f::new(0.23529, 0.67451, 0.843137),
            max_depth: 1,
            epsilon: 0.00001,
            russian_roulette: 0.8,
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

    pub fn add_obj(&mut self, obj: &Rc<dyn Object>) {
        self.objects.push(Rc::clone(obj));
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn get_objs(&self) -> &Vec<Rc<dyn Object>> {
        &self.objects
    }

    pub fn get_lights(&self) -> &Vec<Light> {
        &self.lights
    }

    pub fn build_bvh(&mut self) {
        println!(" - Generating BVH...\n\n");
        self.bvh = Some(Accel::new(&self.objects, 1, SplitMethod::NAIVE));
    }

    pub fn intersect(&self, ray: &Ray) -> Intersection {
        match &self.bvh {
            None => Default::default(),
            Some(bvh) => bvh.intersect(ray),
        }
    }

    pub fn sample_light(&self, pos: &mut Intersection, pdf: &mut f32) {
        let emit_area_sum: f32 = self
            .objects
            .iter()
            .filter(|obj| obj.has_emit())
            .map(|obj| obj.get_area())
            .sum();
        let mut p = get_random_float() * emit_area_sum;
        self.objects
            .iter()
            .filter(|obj| obj.has_emit())
            .find(|obj| {
                p -= obj.get_area();
                p < 0f32
            })
            .unwrap()
            .sample(pos, pdf);
    }
}
