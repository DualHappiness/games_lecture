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

extern crate nalgebra as na;
use na::{Matrix4, Point3, Vector3, Vector4};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Rasterizer {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    projection: Matrix4<f32>,

    pos_buf: HashMap<usize, Vec<Point3<f32>>>,
    ind_buf: HashMap<usize, Vec<Vector3<usize>>>,

    frame_buf: Vec<Vector3<f32>>,
    depth_buf: Vec<f32>,

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
        ret.frame_buf.resize((width * height) as usize, na::zero());
        ret.depth_buf.resize((width * height) as usize, na::zero());
        ret
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

    pub fn set_pixel(&mut self, point: &Vector3<i32>, color: &Vector3<f32>) {
        if point.x < 0 || point.x > self.width || point.y < 0 || point.y > self.height {
            return;
        }

        let ind = (self.height - point.y) * self.width + point.x;
        #[cfg(feature = "show_print")]
        println!(
            "set pixel, {:?} point: {:?}, color: {:?}",
            ind as usize, point, color
        );
        self.frame_buf[ind as usize] = color.clone();
    }

    pub fn clear(&mut self, buff: Buffers) {
        if (buff.clone() & Buffers::Color) == Buffers::Color {
            self.frame_buf
                .iter_mut()
                .for_each(|f| *f = Vector3::new(0.0, 0.0, 0.0));
        }
        if (buff.clone() & Buffers::Depth) == Buffers::Depth {
            self.depth_buf
                .iter_mut()
                .for_each(|d| *d = std::f32::INFINITY);
        }
    }

    pub fn draw(&mut self, pos_buf_id: PosBufID, ind_buf_id: IndBufID, r#type: Primitive) {
        assert_eq!(r#type, Primitive::Triangle);

        let buf = &self.pos_buf[&pos_buf_id.pos_id];
        let ind = &self.ind_buf[&ind_buf_id.ind_id];

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

            let v: Vec<Vector4<f32>> = v
                .iter()
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
            (0..3).for_each(|i| t.set_vertex(i, Vector3::from_column_slice(&v[i].as_slice()[..3])));
            t.set_color(0, 255f32, 0f32, 0f32).expect("set wrong color");
            t.set_color(1, 0f32, 255f32, 0f32).expect("set wrong color");
            t.set_color(2, 0f32, 0f32, 255f32).expect("set wrong color");

            ts.push(t);
        }
        ts.iter().for_each(|t| self.rasterize_wireframe(&t));
    }

    pub fn frame_buffer(&mut self) -> Vec<f32> {
        let mut ret = Vec::with_capacity(self.width as usize * self.height as usize);
        self.frame_buf
            .iter()
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
        self.set_pixel(&point, &line_color);

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
            self.set_pixel(&point, &line_color);
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
}
