use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(FromPrimitive, std::cmp::PartialEq, Clone)]
pub enum Buffers {
    Color = 1,
    Depth = 2,
    Both = 3,
}

impl std::ops::BitOr for Buffers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        FromPrimitive::from_i32(self as i32 | rhs as i32).unwrap()
    }
}

impl std::ops::BitAnd for Buffers {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        FromPrimitive::from_i32(self as i32 & rhs as i32).unwrap()
    }
}

#[derive(Debug, PartialEq)]
pub enum Primitive {
    // Line,
    Triangle,
}

#[derive(Default, Debug, Clone)]
pub struct PosBufID {
    pos_id: usize,
}

#[derive(Default, Debug, Clone)]
pub struct IndBufID {
    ind_id: usize,
}

#[derive(Default, Debug, Clone)]
pub struct ColBufID {
    col_id: usize,
}

extern crate nalgebra as na;
use na::{Matrix4, Point3, Vector2, Vector3, Vector4};
use std::collections::HashMap;

type Vector3f = Vector3<f32>;
#[derive(Debug, Default)]
pub struct Rasterizer {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    projection: Matrix4<f32>,

    pos_buf: HashMap<usize, Vec<Point3<f32>>>,
    ind_buf: HashMap<usize, Vec<Vector3<usize>>>,
    color_buf: HashMap<usize, Vec<Vector3f>>,

    frame_buf: Vec<[Vector3<f32>; 4]>,
    depth_buf: Vec<[f32; 4]>,

    width: i32,
    height: i32,
    next_id: usize,
}

impl Rasterizer {
    pub fn new(width: i32, height: i32) -> Self {
        let mut ret = Self {
            width,
            height,
            ..Default::default()
        };
        ret.frame_buf
            .resize((width * height) as usize, [na::zero(); 4]);
        ret.depth_buf.resize((width * height) as usize, [0f32; 4]);
        ret
    }

    pub fn load_colors(&mut self, colors: &Vec<Vector3f>) -> ColBufID {
        let col_id = self.get_next_id();
        self.color_buf.insert(col_id, colors.clone());
        ColBufID { col_id }
    }

    pub fn load_positions(&mut self, positions: &Vec<Point3<f32>>) -> PosBufID {
        let pos_id = self.get_next_id();
        self.pos_buf.insert(pos_id, positions.clone());
        PosBufID { pos_id }
    }

    pub fn load_indices(&mut self, indices: &Vec<Vector3<usize>>) -> IndBufID {
        let ind_id = self.get_next_id();
        self.ind_buf.insert(ind_id, indices.clone());
        IndBufID { ind_id }
    }

    pub fn set_model(&mut self, m: &Matrix4<f32>) {
        self.model = m.clone();
    }
    pub fn set_view(&mut self, v: &Matrix4<f32>) {
        self.view = v.clone();
    }
    pub fn set_projection(&mut self, p: &Matrix4<f32>) {
        self.projection = p.clone();
    }

    pub fn set_pixel(&mut self, point: &Vector3<i32>, index: usize, color: &Vector3<f32>) {
        if point.x < 0 || point.x > self.width || point.y < 0 || point.y > self.height {
            return;
        }

        let ind = point.y * self.width + self.width - point.x;
        #[cfg(feature = "show_print_more")]
        println!(
            "set pixel, {:?} point: {:?}, color: {:?}",
            ind as usize, point, color
        );
        self.frame_buf[ind as usize][index] = color.clone();
    }

    pub fn clear(&mut self, buff: Buffers) {
        if (buff.clone() & Buffers::Color) == Buffers::Color {
            self.frame_buf
                .iter_mut()
                .for_each(|f| *f = [Vector3::new(0.0, 0.0, 0.0); 4]);
        }
        if (buff.clone() & Buffers::Depth) == Buffers::Depth {
            self.depth_buf.iter_mut().for_each(|d| *d = [f32::MIN; 4]);
        }
    }

    pub fn draw(
        &mut self,
        pos_buf_id: PosBufID,
        ind_buf_id: IndBufID,
        col_buf_id: ColBufID,
        r#type: Primitive,
    ) {
        assert_eq!(r#type, Primitive::Triangle);

        let buf = &self.pos_buf[&pos_buf_id.pos_id];
        let ind = &self.ind_buf[&ind_buf_id.ind_id];
        let color = &self.color_buf[&col_buf_id.col_id];

        let f1 = (100f32 - 0.1) / 2f32;
        let f2 = (100f32 + 0.1) / 2f32;

        let mvp = self.projection * self.view * self.model;

        #[cfg(feature = "show_print")]
        {
            println!(
                "mvp is :\n\nmodel: {:?}\n\nview: {:?}\n\nprojection: {:?}",
                self.model, self.view, self.projection
            );
            println!("mvp~~~ is : {:?}", mvp);
        }
        let mut ts = vec![];
        for i in ind {
            let v: [Vector4<f32>; 3] = [
                mvp * buf[i[0]].to_homogeneous(),
                mvp * buf[i[1]].to_homogeneous(),
                mvp * buf[i[2]].to_homogeneous(),
            ];

            #[cfg(feature = "show_print")]
            println!("original v is {:?}", v);
            let v: Vec<Vector4<f32>> = v
                .iter()
                .map(|vec| vec / vec.w)
                .map(|vert| {
                    Vector4::from_row_slice(&[
                        0.5 * self.width as f32 * (vert.x + 1.0),
                        0.5 * self.height as f32 * (vert.y + 1.0),
                        vert.z * f1 + f2,
                        1.0,
                    ])
                })
                .collect();
            let mut t = super::triangle::Triangle::new();
            (0..3).for_each(|index| {
                t.set_vertex(index, Vector3::from_column_slice(&v[index].as_slice()[..3]));
                let col = color[i[index]];
                t.set_color(index, col.x, col.y, col.z)
                    .expect("set wrong color");
            });
            ts.push(t);
        }
        ts.iter().for_each(|t| self.rasterize_triangle(&t));
    }

    pub fn frame_buffer(&mut self) -> Vec<f32> {
        let mut ret = Vec::with_capacity(self.width as usize * self.height as usize);
        self.frame_buf
            .iter()
            .map(|colors| colors.iter().sum::<Vector3f>() / colors.len() as f32)
            .for_each(|color| color.data.iter().for_each(|f| ret.push(*f)));
        ret
    }

    fn draw_line(&mut self, begin: Vector3<f32>, end: Vector3<f32>) {
        #[cfg(feature = "show_print")]
        println!("draw line : {:?} , {:?}", begin, end);
        let (x1, y1) = (begin.x, begin.y);
        let (x2, y2) = (end.x, end.y);

        let line_color = Vector3::new(255.0, 255.0, 255.0);
        let (dx, dy) = ((x2 - x1) as i32, (y2 - y1) as i32);
        let line_dir = (dx < 0 && dy < 0) || (dx > 0 && dy > 0);
        let (dx1, dy1) = (dx.abs(), dy.abs());

        let mut px = 2 * dy1 - dx1;
        let mut py = 2 * dx1 - dy1;

        let (x1, y1, x2, y2) = (x1 as i32, y1 as i32, x2 as i32, y2 as i32);
        let (dir, (mut x, mut y, range), (p, d1, d2)) = if dy1 <= dx1 {
            let l = (&mut px, dy1, dx1);
            let n = if dx >= 0 { (x1, y1, x2) } else { (x2, y2, x1) };
            (true, n, l)
        } else {
            let l = (&mut py, dx1, dy1);
            let n = if dy >= 0 { (x1, y1, y2) } else { (x2, y2, y1) };
            (false, n, l)
        };
        let point = Vector3::new(x, y, 1);
        self.set_pixel(&point, 0, &line_color);

        #[cfg(feature = "show_print")]
        {
            println!("{:?}", (dir, (x, y, range), (&p, d1, d2)));
            println!("range is {}", range);
        }
        if !dir {
            std::mem::swap(&mut x, &mut y);
        }
        while x < range {
            x += 1;
            if *p < 0 {
                *p += 2 * d1;
            } else {
                y += if line_dir { 1 } else { -1 };
                *p += 2 * (d1 - d2);
            }
            let point = if dir {
                Vector3::new(x, y, 1)
            } else {
                Vector3::new(y, x, 1)
            };
            self.set_pixel(&point, 0, &line_color);
        }
    }

    fn rasterize_wireframe(&mut self, t: &super::triangle::Triangle) {
        self.draw_line(t.c(), t.a());
        self.draw_line(t.c(), t.b());
        self.draw_line(t.b(), t.a());
    }

    fn get_index(&self, x: i32, y: i32) -> i32 {
        (self.height - y) * self.width + x
    }

    fn get_next_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id
    }

    fn inside_triangle(x: f32, y: f32, v: &[Vector3<f32>; 3]) -> bool {
        let p = Vector3::new(x, y, 0f32);
        let f = |o: Vector3f, p1: Vector3f, p2: Vector3f| {
            // ! 除了v3不让用cross 坑的要死
            let n = (p1 - o).cross(&(p2 - o));
            #[cfg(feather = "show_print_more")]
            {
                println!("points  {}, {} , {}", o, p1, p2);
                println!("vector is {}, {}", p1 - o, p2 - o);
                println!("normal is {}", n);
            }
            n.z > 0f32
        };

        let b = f(v[0], v[1], p);

        #[cfg(feature = "show_print_more")]
        println!(
            "inside check is {}, {}, {}",
            b,
            f(v[1], v[2], p),
            f(v[2], v[0], p)
        );
        b == f(v[1], v[2], p) && b == f(v[2], v[0], p)
    }

    // todo 自己写一个正确的 现在输出的是负值
    fn compute_barycentric2d(x: f32, y: f32, v: &[Vector3f; 3]) -> (f32, f32, f32) {
        let f = |a: Vector3f, b: Vector3f, c: Vector3f| {
            (x * (a.y - b.y) + y * (b.x - a.x) + a.x * b.y - b.x * a.y)
                / (c.x * (a.y - b.y) + c.y * (b.x - a.x) + a.x * b.y - b.x * a.y)
        };
        let alpha = f(v[1], v[2], v[0]);
        let beta = f(v[2], v[0], v[1]);
        let gamma = f(v[0], v[1], v[2]);
        (alpha, beta, gamma)
    }

    pub fn rasterize_triangle(&mut self, t: &super::triangle::Triangle) {
        let vs = t.to_vector4();

        #[cfg(feature = "show_print")]
        println!("triangle vertex is  {:?}", vs);
        let (lower_bound, upper_bound) = vs
            .iter()
            .fold(((f32::MAX, f32::MAX), (f32::MIN, f32::MIN)), |(l, u), v| {
                ((l.0.min(v.x), l.1.min(v.y)), (u.0.max(v.x), u.1.max(v.y)))
            });
        let mut v3s = [Default::default(); 3];
        for i in 0..3 {
            v3s[i] = Vector3::new(vs[i].x, vs[i].y, 0f32);
        }
        for i in lower_bound.0 as i32..upper_bound.0 as i32 + 1 {
            for j in lower_bound.1 as i32..upper_bound.1 as i32 + 1 {
                // super sampling
                for (sub_index, (dx, dy)) in
                    [(0.25, 0.25), (0.25, 0.75), (0.75, 0.25), (0.75, 0.75)]
                        .iter()
                        .enumerate()
                {
                    let (x, y) = (i as f32 + dy, j as f32 + dy);
                    if Self::inside_triangle(x, y, &v3s) {
                        #[cfg(feature = "show_print")]
                        println!("inside pos is {}, {}", i, j);
                        let (alpha, beta, gamma) = Self::compute_barycentric2d(x, y, &t.v);
                        let w_reciprocal =
                            1.0 / (alpha / vs[0].w + beta / vs[1].w + gamma / vs[2].w);
                        let mut z_interpolated = alpha * vs[0].z / vs[0].w
                            + beta * vs[1].z / vs[1].w
                            + gamma * vs[2].z / vs[2].w;
                        z_interpolated *= w_reciprocal;

                        let index = (j * self.width + self.width - i) as usize;
                        if self.depth_buf[index][sub_index] < z_interpolated.abs() {
                            self.depth_buf[index][sub_index] = z_interpolated.abs();
                            self.set_pixel(&Vector3::new(i, j, 1), sub_index, &t.get_color());
                        }
                    }
                }
            }
        }
    }
}
