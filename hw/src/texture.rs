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
        Self {
            image_data,
            width,
            height,
        }
    }

    pub fn get_color(&self, u: f32, v: f32) -> Vector3<f32> {
        let u = u * self.width as f32;
        let v = (1f32 - v) * self.height as f32;
        let color = self
            .image_data
            .at_2d::<core::Vec3b>(u as i32, v as i32)
            .expect("get color wrong");
        Vector3::new(color[0] as f32, color[1] as f32, color[2] as f32)
    }
}
