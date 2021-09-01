use nalgebra::Vector3;
use opencv::{core, imgcodecs, imgproc, prelude::*};

pub struct Texture {
    image_data: Mat,
    pub width: i32,
    pub height: i32,
}

impl Texture {
    pub fn new(name: &str) -> Self {
        let mut image_data = Mat::default().unwrap();
        imgproc::cvt_color(
            &imgcodecs::imread(name, imgcodecs::IMREAD_COLOR).expect("read img error"),
            &mut image_data,
            imgproc::COLOR_RGB2BGR,
            0,
        )
        .expect(&format!("convert color error: {}", name));
        let (width, height) = (image_data.cols(), image_data.rows());

        println!("texture size is {}, {}", width, height);

        Self {
            image_data,
            width,
            height,
        }
    }

    fn raw_get_color(&self, u: i32, v: i32) -> Vector3<f32> {
        let u = if u < 0 { u + self.width } else { u };
        let v = if v < 0 { v + self.height } else { v };
        let color = self
            .image_data
            .at_2d::<core::Vec3b>(v, u)
            .expect("get color wrong");
        Vector3::new(color[0] as f32, color[1] as f32, color[2] as f32)
    }

    pub fn get_color(&self, u: f32, v: f32) -> Vector3<f32> {
        let u = (u * self.width as f32) as i32;
        let v = ((1f32 - v) * self.height as f32) as i32;
        self.raw_get_color(u, v)
    }

    pub fn get_color_bilinear(&self, u: f32, v: f32) -> Vector3<f32> {
        let u = u * self.width as f32;
        let v = (1f32 - v) * self.height as f32;

        let uu = (u + 0.5) as i32 as f32;
        let uv = (v + 0.5) as i32 as f32;
        let lu = (u - 0.5) as i32 as f32;
        let lv = (v - 0.5) as i32 as f32;

        let s = (u - lu) / (uu - lu);
        let t = (v - lv) / (uv - lv);

        let (lu, lv, uu, uv) = (lu as i32, lv as i32, uu as i32, uv as i32);
        let c00 = self.raw_get_color(lu, lv);
        let c01 = self.raw_get_color(uu, lv);
        let c10 = self.raw_get_color(lu, uv);
        let c11 = self.raw_get_color(uu, uv);

        let c0 = c00.lerp(&c01, s);
        let c1 = c10.lerp(&c11, s);

        c0.lerp(&c1, t)
    }

    pub fn get_color_raw(&self, u: i32, v: i32) -> Vector3<f32> {
        self.raw_get_color(u, v)
    }
}
