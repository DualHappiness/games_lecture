use super::*;

fn ray_triangle_intersect(
    v0: &Vector3f,
    v1: &Vector3f,
    v2: &Vector3f,
    origin: &Vector3f,
    dir: &Vector3f,
) -> Option<(f32, Vector2f)> {
    None
}

#[derive(Default)]
pub struct MeshTriangle {
    pub obj: Obj,

    pub vertices: Vec<Vector3f>,
    pub num_triangles: usize,
    pub vertex_index: Vec<usize>,
    pub st_coordinates: Vec<Vector2f>,
}

impl MeshTriangle {
    pub fn new(
        verts: &Vec<Vector3f>,
        verts_index: &Vec<usize>,
        num_triangles: usize,
        st: &Vec<Vector2f>,
    ) -> Self {
        Self {
            num_triangles,
            vertices: verts.clone(),
            vertex_index: verts_index.clone(),
            st_coordinates: st.clone(),
            ..Default::default()
        }
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
    fn intersect(&self, origin: &Vector3f, dir: &Vector3f) -> Option<(f32, usize, Vector2f)> {
        let mut ret = None;
        for k in 0..self.num_triangles {
            let v0 = &self.vertices[self.vertex_index[k * 3]];
            let v1 = &self.vertices[self.vertex_index[k * 3 + 1]];
            let v2 = &self.vertices[self.vertex_index[k * 3 + 2]];

            match ray_triangle_intersect(v0, v1, v2, origin, dir) {
                None => (),
                Some((t, uv)) => ret = Some((t, k, uv)),
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
}
