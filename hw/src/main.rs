use nalgebra::{Vector3, Vector4};
use opencv::{core, highgui, imgcodecs, imgproc, prelude::*};
use opencv_learn::{obj_loader::Loader, rasterizer, texture::Texture, triangle::Triangle, *};
use std::default::Default;
use std::env;

const SIZE: i32 = 700;

fn draw_image(
    angle: &f32,
    eye_pos: &Vector3<f32>,
    r: &mut rasterizer::Rasterizer,
    triangle_list: &Vec<triangle::Triangle>,
) -> Mat {
    r.clear(rasterizer::Buffers::Color | rasterizer::Buffers::Depth);

    r.set_model(&opencv_learn::get_model_matrix(*angle));
    r.set_view(&opencv_learn::get_view_matrix(*eye_pos));
    r.set_projection(&opencv_learn::get_projection_matrix(
        45f32, 1f32, 0.1f32, 50f32,
    ));

    r.draw_triangles(&triangle_list.iter().map(|t| t).collect());
    let mut buf = r.frame_buffer();
    let ptr = buf.as_mut_ptr() as *mut std::ffi::c_void;
    let mut ret = Mat::default().unwrap();
    unsafe {
        let image =
            Mat::new_rows_cols_with_data(SIZE, SIZE, core::CV_32FC3, ptr, core::Mat_AUTO_STEP)
                .expect("build image fail");
        let mut temp = Mat::default().unwrap();
        image
            .convert_to(&mut temp, core::CV_8UC3, 1f64, 0f64)
            .expect("convert err");
        imgproc::cvt_color(&temp, &mut ret, imgproc::COLOR_RGB2BGR, 0).expect("cvt color error");
    }
    ret
}

type Vector3f = Vector3<f32>;
fn main() {
    let args: Vec<_> = env::args().collect();

    let mut command_line = false;
    let mut filename = "output.png";

    let mut triangles = vec![];
    let obj_path = "./models/spot/".to_owned();
    // load obj file
    {
        let mut loader: Loader = Default::default();
        loader
            .load_file(&(obj_path.clone() + "spot_triangulated_good.obj"))
            .expect("load file err");
        for mesh in &loader.loaded_meshes {
            let mut index = 0;
            while index < mesh.vertices.len() {
                let mut t: Triangle = Default::default();
                for j in 0..3 {
                    let vert = mesh.vertices[index + j];
                    t.set_vertex(
                        j,
                        Vector4::new(vert.position.x, vert.position.y, vert.position.z, 1f32),
                    );
                    t.set_normal(j, vert.normal);
                    t.set_tex_coord(j, vert.texture_coordinates);
                }
                triangles.push(t);
                index += 3;
            }
        }
    }

    let mut angle = 140f32;
    let mut r = rasterizer::Rasterizer::new(SIZE, SIZE);

    let texture_path = "hmap.jpg";
    r.set_texture(Texture::new(&(obj_path.clone() + texture_path)));

    let mut active_shader: fn(&shader::FragmentShaderPayload) -> Vector3f = phone_fragment_shader;
    if args.len() >= 2 {
        command_line = true;
        filename = &args[1];
        if args.len() >= 3 {
            match &args[2][..] {
                "texture" => {
                    println!("Rasterizing using the texture shader");
                    active_shader = texture_fragment_shader;
                    r.set_texture(Texture::new(&(obj_path.clone() + "spot_texture_low.png")));
                }
                "normal" => {
                    println!("Rasterizing using the normal shader");
                    active_shader = normal_fragment_shader;
                }
                "phong" => {
                    println!("Rasterizing using the phong shader");
                    active_shader = phone_fragment_shader;
                }
                "bump" => {
                    println!("Rasterizing using the bump shader");
                    active_shader = bump_fragment_shader;
                }
                "displacement" => {
                    println!("Resterizing using the displacement shader");
                    active_shader = displacement_fragment_shader;
                }
                arg => println!("error shader argument {}", arg),
            }
        }
    }

    r.set_fragment_shader(&active_shader);
    r.set_vertex_shader(&vertex_shader);

    let eye_pos = Vector3::new(0f32, 0f32, 10f32);
    let mut key = 0 as u8;
    // let mut frame_count = 0;

    if command_line {
        let image = draw_image(&angle, &eye_pos, &mut r, &triangles);
        imgcodecs::imwrite(filename, &image, &core::Vector::new()).unwrap();
    } else {
        while key != 27 {
            let image = draw_image(&angle, &eye_pos, &mut r, &triangles);

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
