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
    Line,
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

    pub fn set_pixel(&mut self, point: &Vector3<f32>, color: &Vector3<f32>) {
        // println!("set pixel, point: {:?}, color: {:?}", point, color);
        if point.x < 0.0
            || point.x > self.width as f32
            || point.y < 0.0
            || point.y > self.height as f32
        {
            return;
        }

        let ind = (self.height as f32 - point.y) * self.width as f32 + point.x;
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

        println!(
            "mvp is :\n\nmodel: {:?}\n\nview: {:?}\n\nprojection: {:?}",
            self.model, self.view, self.projection
        );
        let mvp = self.projection * self.view * self.model;

        println!("mvp~~~ is : {:?}", mvp);
        let mut ts = vec![];
        for i in ind {
            let v: [Vector4<f32>; 3] = [
                mvp * buf[i[0]].to_homogeneous(),
                mvp * buf[i[1]].to_homogeneous(),
                mvp * buf[i[2]].to_homogeneous(),
            ];
            println!("v is : {:?}", v);

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
            println!("v is : {:?}", v);
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
        // println!("{:?}", self.frame_buf);

        let mut ret = Vec::with_capacity(self.width as usize * self.height as usize);
        self.frame_buf
            .iter()
            .for_each(|color| color.data.iter().for_each(|f| ret.push(*f)));
        ret
    }

    fn draw_line(&mut self, begin: Vector3<f32>, end: Vector3<f32>) {
        let (mut x1, mut y1) = (begin.x, begin.y);
        let (mut x2, mut y2) = (end.x, end.y);

        let line_color = Vector3::new(255.0, 255.0, 255.0);
        let (dx, dy) = (x2 - x1, y2 - y1);
        let dx1 = dx.abs();
        let dy1 = dy.abs();

        let mut px = 2.0 * dy1 - dx1;
        let mut py = 2.0 * dx1 - dy1;

        if dy1 <= dx1 {
            let (mut x, mut y, range) = if dx >= 0f32 {
                (x1, y1, x2 as i32)
            } else {
                (x2, y2, x1 as i32)
            };

            let point = Vector3::new(x, y, 1f32);
            self.set_pixel(&point, &line_color);

            for _ in 0..range + 1 {
                x += 1f32;
                if px < 0f32 {
                    px += 2f32 * dy1;
                } else {
                    if (dx <= 0f32 && dy <= 0f32) || (dx >= 0f32 && dy >= 0f32) {
                        y += 1f32;
                    } else {
                        y -= 1f32;
                    }
                    px += 2f32 * (dy1 - dx1);
                }
            }
            let point = Vector3::new(x, y, 1f32);
            self.set_pixel(&point, &line_color);
        } else {
            let (mut x, mut y, range) = if dy >= 0f32 {
                (x1, y1, y2 as i32)
            } else {
                (x2, y2, y1 as i32)
            };
            let point = Vector3::new(x, y, 1f32);
            self.set_pixel(&point, &line_color);
            for _ in 0..range + 1 {
                y += 1f32;
                if py <= 0f32 {
                    py += 2f32 * dx1;
                } else {
                    if (dx <= 0f32 && dy <= 0f32) || (dx >= 0f32 && dy >= 0f32) {
                        x += 1f32;
                    } else {
                        x -= 1f32;
                    }
                    py += 2f32 * (dx1 - dy1);
                }
                let point = Vector3::new(x, y, 1f32);
                self.set_pixel(&point, &line_color);
            }
        }
    }
    // fn draw_line(&mut self, begin: Vector3<f32>, end: Vector3<f32>) {
    //     let (mut x1, mut y1) = (begin.x, begin.y);
    //     let (mut x2, mut y2) = (end.x, end.y);

    //     let line_color = Vector3::new(255.0, 255.0, 255.0);
    //     let (dx, dy) = (x2 - x1, y2 - y1);
    //     let dx1 = dx.abs();
    //     let dy1 = dy.abs();

    //     let mut px = 2.0 * dy1 - dx1;
    //     let mut py = 2.0 * dx1 - dy1;

    //     let (mut x, mut y, range, rev, p, d1, d2) = if dy1 <= dx1 {
    //         if dx >= 0.0 {
    //             (x1, y1, x2 as i32, true, &mut px, dy1, dx1)
    //         } else {
    //             (x2, y2, x1 as i32, true, &mut px, dy1, dx1)
    //         }
    //     } else {
    //         if dy >= 0.0 {
    //             (x1, y1, y2 as i32, false, &mut py, dx1, dy1)
    //         } else {
    //             (x2, y2, y1 as i32, false, &mut py, dx1, dy1)
    //         }
    //     };
    //     let point = Vector3::new(x, y, 1.0);
    //     self.set_pixel(&point, &line_color);

    //     for _ in 0..range + 1 {
    //         let (m1, m2) = if rev {
    //             (&mut x, &mut y)
    //         } else {
    //             (&mut y, &mut x)
    //         };
    //         *m1 += 1.0;
    //         if *p < 0.0 {
    //             *p += 2.0 * d1;
    //         } else {
    //             if (dx < 0.0 && dy < 0.0) || (dx > 0.0 && dy > 0.0) {
    //                 *m2 += 1.0;
    //             } else {
    //                 *m2 -= 1.0;
    //             }
    //             *p += 2.0 * (d1 - d2);
    //         }

    //         let point = Vector3::new(x, y, 1.0);
    //         self.set_pixel(&point, &line_color);
    //     }
    // }

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
