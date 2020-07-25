use nalgebra::{Point3, Vector3};
use opencv::{core, highgui, imgcodecs, prelude::*};
use std::env;

mod rasterizer;
mod triangle;

const SIZE: i32 = 700;

fn draw_image(
    angle: &f32,
    eye_pos: &Vector3<f32>,
    r: &mut rasterizer::Rasterizer,
    pos_id: rasterizer::PosBufID,
    ind_id: rasterizer::IndBufID,
    col_id: rasterizer::ColBufID,
) -> Mat {
    r.clear(rasterizer::Buffers::Color | rasterizer::Buffers::Depth);

    r.set_model(&opencv_learn::get_model_matrix(*angle));
    r.set_view(&opencv_learn::get_view_matrix(*eye_pos));
    r.set_projection(&opencv_learn::get_projection_matrix(
        120f32, 1f32, 5f32, 10f32,
    ));

    r.draw(pos_id, ind_id, col_id, rasterizer::Primitive::Triangle);
    let mut buf = r.frame_buffer();
    let ptr = buf.as_mut_ptr() as *mut std::ffi::c_void;
    let mut ret = Mat::default().unwrap();
    unsafe {
        let image =
            Mat::new_rows_cols_with_data(SIZE, SIZE, core::CV_32FC3, ptr, core::Mat_AUTO_STEP)
                .expect("build image fail");
        image
            .convert_to(&mut ret, core::CV_8UC3, 1f64, 0f64)
            .expect("convert err");
    }
    ret
}

type Vector3f = Vector3<f32>;
fn main() {
    let args: Vec<_> = env::args().collect();

    let mut command_line = false;
    let mut filename = "output.png";

    let mut angle = 0f32;

    if args.len() >= 3 {
        command_line = true;
        angle = args[2].parse().unwrap();

        if args.len() >= 4 {
            filename = &args[3];
        }
    }

    let mut r = rasterizer::Rasterizer::new(SIZE, SIZE);

    let eye_pos = Vector3::new(0f32, 0f32, 5f32);

    let points = vec![
        Point3::new(2f32, 0f32, -2f32),
        Point3::new(0f32, 2f32, -2f32),
        Point3::new(-2f32, 0f32, -2f32),
        Point3::new(3.5, -1f32, -5f32),
        Point3::new(2.5, 1.5, -5f32),
        Point3::new(-1f32, 0.5, -5f32),
    ];
    let ind = vec![
        Vector3::new(0, 1, 2), 
        Vector3::new(3, 4, 5),
        ];

    let pos_id = r.load_positions(&points);
    let ind_id = r.load_indices(&ind);

    let colors = vec![
        Vector3f::new(217.0, 238.0, 185.0),
        Vector3f::new(217.0, 238.0, 185.0),
        Vector3f::new(217.0, 238.0, 185.0),
        Vector3f::new(185.0, 217.0, 238.0),
        Vector3f::new(185.0, 217.0, 238.0),
        Vector3f::new(185.0, 217.0, 238.0),
    ];
    let col_id = r.load_colors(&colors);
    let mut key = 0 as u8;
    // let mut frame_count = 0;

    if command_line {
        let image = draw_image(&angle, &eye_pos, &mut r, pos_id, ind_id, col_id);
        imgcodecs::imwrite(filename, &image, &core::Vector::new()).unwrap();
    } else {
        while key != 27 {
            let image = draw_image(
                &angle,
                &eye_pos,
                &mut r,
                pos_id.clone(),
                ind_id.clone(),
                col_id.clone(),
            );

            highgui::imshow("show image", &image).unwrap();
            key = highgui::wait_key(0).unwrap() as u8;

            // frame_count += 1;
            if key == b'a' {
                angle += 10f32;
            }

            if key == b'd' {
                angle -= 10f32;
            }
        }
    }
}
