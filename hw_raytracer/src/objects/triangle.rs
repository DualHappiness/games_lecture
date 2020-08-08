use super::*;

fn ray_triangle_intersect(
    v0: &Vector3f,
    v1: &Vector3f,
    v2: &Vector3f,
    origin: &Vector3f,
    dir: &Vector3f,
) -> Option<(f32, Vector2f)> {
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let s = origin - v0;
    let s1 = dir.cross(&e2);
    let s2 = s.cross(&e1);

    let inverse_s1e1 = 1f32 / s1.dot(&e1);
    let t = inverse_s1e1 * s2.dot(&e2);
    let b1 = inverse_s1e1 * s1.dot(&s);
    let b2 = inverse_s1e1 * s2.dot(&dir);

    if t >= 0f32 && b1 >= 0f32 && b2 >= 0f32 && 1f32 - b1 - b2 >= 0f32 {
        Some((t, Vector2f::new(b1, b2)))
    } else {
        None
    }
}

#[derive(Default)]
pub struct Triangle {
    pub obj: Obj,

    pub v0: Vector3f,
    pub v1: Vector3f,
    pub v2: Vector3f,

    pub e1: Vector3f,
    pub e2: Vector3f,

    pub t0: Vector3f,
    pub t1: Vector3f,
    pub t2: Vector3f,

    pub normal: Vector3f,
    pub area: f32,
    pub material: Option<Rc<Material>>,
}

impl Clone for Triangle {
    fn clone(&self) -> Self {
        let new_mat = match &self.material {
            None => None,
            Some(mat) => Some(Rc::clone(mat)),
        };
        Self {
            material: new_mat,
            ..*self
        }
    }
}

impl Deref for Triangle {
    type Target = Obj;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl DerefMut for Triangle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.obj
    }
}

impl Triangle {
    pub fn new([v0, v1, v2]: [Vector3f; 3], material: Option<Rc<Material>>) -> Self {
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        Self {
            v0,
            v1,
            v2,
            material,
            e1,
            e2,
            normal: e1.cross(&e2).normalize(),
            area: e1.cross(&e2).norm() * 0.5,
            ..Default::default()
        }
    }
}

impl Object for Triangle {
    fn intersect(&self, _ray: &ray::Ray) -> Option<(f32, usize)> {
        None
        // ! ???
    }
    fn get_intersection(&self, ray: &ray::Ray) -> intersection::Intersection {
        let mut intersection = Intersection::default();

        if ray.direction.dot(&self.normal) > 0f32 {
            return intersection;
        }

        let pvec = ray.direction.cross(&self.e2);
        let det = self.e1.dot(&pvec);
        if det.abs() < EPSILON {
            return intersection;
        }

        let det_inv = 1f32 / det;
        let tvec = ray.origin - self.v0;

        let u = tvec.dot(&pvec) * det_inv;
        if u < 0f32 || u > 1f32 {
            return intersection;
        }

        let qvec = tvec.cross(&self.e1);
        let v = ray.direction.dot(&qvec) * det_inv;
        if v < 0f32 || v + u > 1f32 {
            return intersection;
        }

        let t_tmp = self.e2.dot(&qvec) * det_inv;
        if t_tmp < 0f32 {
            return intersection;
        }

        intersection.happened = true;
        intersection.distance = t_tmp;
        intersection.normal = self.normal;
        intersection.coords = ray.at(t_tmp);
        intersection.obj = Some(Box::from(self.clone()) as Box<dyn Object>);
        intersection.m = match &self.material {
            None => None,
            Some(mat) => Some(Rc::clone(mat)),
        };
        intersection
    }
    fn get_surface_properties(
        &self,
        _: &Vector3f,
        _: &Vector3f,
        _: &usize,
        _: &Vector2f,
    ) -> (Vector3f, Vector2f) {
        (self.normal, nalgebra::zero())
    }

    fn eval_diffuse_color(&self, _: &Vector2f) -> Vector3f {
        Vector3f::from_element(0.5)
    }

    fn get_bounds(&self) -> bound::Bound3 {
        union_point(&Bound3::new(self.v0, self.v1), &self.v2)
    }
    fn get_area(&self) -> f32 {
        self.area
    }
    fn sample(&self, pos: &mut intersection::Intersection, pdf: &mut f32) {
        let x = get_random_float().sqrt();
        let y = get_random_float();

        pos.coords = self.v0 * (1f32 - x) + self.v1 * (x * (1f32 - y)) + self.v2 * (x * y);
        pos.normal = self.normal;
        *pdf = 1f32 / self.area;
    }
    fn has_emit(&self) -> bool {
        match &self.material {
            None => false,
            Some(m) => m.has_emission(),
        }
    }
}

#[derive(Default)]
pub struct MeshTriangle {
    pub obj: Obj,

    pub vertices: Vec<Vector3f>,
    pub num_triangles: usize,
    pub vertex_index: Vec<usize>,
    pub st_coordinates: Vec<Vector2f>,

    pub triangles: Vec<Rc<Triangle>>,

    pub bvh: Option<Accel>,
    pub area: f32,
    pub m: Material,

    pub bounding_box: Bound3,
}

impl MeshTriangle {
    pub fn new(filename: &str, m: Material) -> Self {
        let mut ret = Self::default();
        ret.m = m;
        let mut loader = obj_loader::Loader::default();
        loader.load_file(filename).expect("load file error");
        assert_eq!(loader.loaded_meshes.len(), 1);

        let mesh = &loader.loaded_meshes[0];
        let mut min_vert = Vector3f::from_element(f32::MAX);
        let mut max_vert = Vector3f::from_element(f32::MIN);

        let mut i = 0;
        while i < mesh.vertices.len() {
            let mut face_vertices: [Vector3f; 3] = [nalgebra::zero(); 3];
            for j in 0..3 {
                face_vertices[j] = mesh.vertices[i + j].position;
                min_vert = v3min(&min_vert, &face_vertices[j]);
                max_vert = v3max(&max_vert, &face_vertices[j]);
            }
            ret.triangles
                .push(Rc::new(Triangle::new(face_vertices, Some(Rc::from(m)))));

            i += 3;
        }
        ret.bounding_box = Bound3::new(min_vert, max_vert);
        #[cfg(feature = "show_print")]
        println!("obj bounding box is {:?}, {:?}", min_vert, max_vert);
        ret.area = ret.triangles.iter().map(|t| t.area).sum();

        let ptrs = ret
            .triangles
            .iter()
            .map(|t| Rc::clone(&t))
            .map(|t| t as Rc<dyn Object>)
            .collect();
        ret.bvh = Some(Accel::new(&ptrs, 1, SplitMethod::NAIVE));
        ret
    }
}

impl Deref for MeshTriangle {
    type Target = Obj;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl DerefMut for MeshTriangle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.obj
    }
}

impl Object for MeshTriangle {
    fn intersect(&self, ray: &Ray) -> Option<(f32, usize)> {
        let mut ret = None;
        for k in 0..self.num_triangles {
            let v0 = &self.vertices[self.vertex_index[k * 3]];
            let v1 = &self.vertices[self.vertex_index[k * 3 + 1]];
            let v2 = &self.vertices[self.vertex_index[k * 3 + 2]];

            match ray_triangle_intersect(v0, v1, v2, &ray.origin, &ray.direction) {
                None => (),
                Some((t, _uv)) => ret = Some((t, k)),
            }
        }
        ret
    }

    fn get_surface_properties(
        &self,
        _: &Vector3f,
        _: &Vector3f,
        index: &usize,
        uv: &Vector2f,
    ) -> (Vector3f, Vector2f) {
        let v0 = &self.vertices[self.vertex_index[index * 3]];
        let v1 = &self.vertices[self.vertex_index[index * 3 + 1]];
        let v2 = &self.vertices[self.vertex_index[index * 3 + 2]];

        let e0 = (v1 - v0).normalize();
        let e1 = (v2 - v0).normalize();
        let n = e0.cross(&e1).normalize();

        let st0 = self.st_coordinates[self.vertex_index[index * 3]];
        let st1 = self.st_coordinates[self.vertex_index[index * 3 + 1]];
        let st2 = self.st_coordinates[self.vertex_index[index * 3 + 2]];
        let st = st0 * (1f32 - uv.x - uv.y) + st1 * uv.x + st2 * uv.y;
        (n, st)
    }

    fn eval_diffuse_color(&self, st: &Vector2f) -> Vector3f {
        let scale = 5f32;
        let f = |n| if (n * scale) % 1f32 > 0.5 { 1f32 } else { 0f32 };
        let pattern = f(st.x).powf(f(st.y));
        Vector3f::new(0.815, 0.235, 0.031).lerp(&Vector3f::new(0.937, 0.937, 0.231), pattern)
    }
    fn get_intersection(&self, ray: &ray::Ray) -> intersection::Intersection {
        match &self.bvh {
            None => Intersection::default(),
            Some(bvh) => bvh.intersect(&ray),
        }
    }
    fn get_bounds(&self) -> bound::Bound3 {
        self.bounding_box
    }
    fn get_area(&self) -> f32 {
        self.area
    }
    fn sample(&self, pos: &mut intersection::Intersection, pdf: &mut f32) {
        self.bvh.as_ref().unwrap().sample(pos, pdf);
        pos.emit = self.m.emission;
    }
    fn has_emit(&self) -> bool {
        self.m.has_emission()
    }
}
