use bezier::*;
use opencv::{core, highgui, imgcodecs, imgproc, prelude::*};

fn main() -> opencv::Result<()> {
    let mut window = Mat::default()?;
    imgproc::cvt_color(
        &Mat::new_rows_cols_with_default(700, 700, core::CV_8UC3, core::Scalar::all(0f64))?,
        &mut window,
        imgproc::COLOR_RGB2BGR,
        0,
    )?;
    let window_name = "Bezier Curve";
    highgui::named_window(window_name, highgui::WINDOW_AUTOSIZE)?;
    highgui::set_mouse_callback(window_name, Some(Box::new(mouse_handler)))?;

    let mut key = -1;
    while key != 27 {
        unsafe {
            let scalar = core::Scalar::new(255f64, 255f64, 255f64, 0f64);
            for &point in &CONTROL_POINTS {
                imgproc::circle(&mut window, point, 3, scalar, 3, imgproc::LINE_8, 0)?;
            }

            if CONTROL_POINTS.len() >= 4 {
                // naive_bezier(&CONTROL_POINTS, &mut window)?;
                bezier(&CONTROL_POINTS, &mut window)?;

                imgcodecs::imwrite("my_bezier_curve.png", &window, &core::Vector::new())?;
            }
        }

        highgui::imshow(window_name, &window)?;
        key = highgui::wait_key(20)?;
    }
    Ok(())
}
