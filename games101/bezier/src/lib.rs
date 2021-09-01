use opencv::{core, highgui, prelude::*};

pub static mut CONTROL_POINTS: Vec<core::Point2i> = vec![];

pub fn mouse_handler(event: i32, x: i32, y: i32, _flags: i32) {
    unsafe {
        if event == highgui::EVENT_LBUTTONDOWN {
            println!(
                "Left button of the mouse is clicked - position ({}, {})",
                x, y
            );
            CONTROL_POINTS.push(core::Point2i::new(x, y));
        }
    }
}

fn to_point2d(o: &core::Point2i) -> core::Point2d {
    core::Point2d::from_vec2(core::Vec2::from([o.x as f64, o.y as f64]))
}

pub fn naive_bezier(points: &Vec<core::Point2i>, window: &mut core::Mat) -> opencv::Result<()> {
    let p0 = to_point2d(&points[0]);
    let p1 = to_point2d(&points[1]);
    let p2 = to_point2d(&points[2]);
    let p3 = to_point2d(&points[3]);

    let mut t = 0f64;
    while t < 1.0 {
        let point = p0 * (1.0 - t).powi(3)
            + p1 * 3.0 * t * (1.0 - t).powi(2)
            + p2 * 3.0 * t.powi(2) * (1.0 - t)
            + p3 * t.powi(3);
        window.at_2d_mut::<core::Vec3b>(point.y as i32, point.x as i32)?[2] = 255;
        t += 0.001;
    }
    Ok(())
}

fn recursive_bezier(points: &Vec<core::Point2d>, t: f64) -> opencv::Result<core::Point2d> {
    if points.len() == 1 {
        return Ok(points[0]);
    }
    let points = points
        .iter()
        .zip(points.iter().skip(1))
        .map(|(&p1, &p2)| p1 * (1.0 - t) + p2 * t)
        .collect();
    recursive_bezier(&points, t)
}

fn to_vec3d(c: &core::Vec3b) -> core::Vec3d {
    core::Vec3d::from([c[0] as f64, c[1] as f64, c[2] as f64])
}

fn mul(c: core::Vec3d, s: f64) -> core::Vec3d {
    core::Vec3d::from([c[0] * s, c[1] * s, c[2] * s])
}

fn add(c1: core::Vec3d, c2: core::Vec3d) -> core::Vec3d {
    core::Vec3d::from([c1[0] + c2[0], c1[1] + c2[1], c1[2] + c2[2]])
}

fn bilinear(points: Vec<core::Point2d>, window: &mut Mat) -> opencv::Result<()> {
    for point in points {
        let lu = (point.x - 0.5) as i32;
        let lv = (point.y - 0.5) as i32;

        let s = (point.x - lu as f64) / 2.0;
        let t = (point.y - lv as f64) / 2.0;

        let c00: core::Vec3d = to_vec3d(window.at_2d(lu, lv)?);
        let c01: core::Vec3d = to_vec3d(window.at_2d(lu + 1, lv)?);
        let c10: core::Vec3d = to_vec3d(window.at_2d(lu, lv + 1)?);
        let c11: core::Vec3d = to_vec3d(window.at_2d(lu + 1, lv + 1)?);

        let c0 = add(mul(c00, 1.0 - s), mul(c01, s));
        let c1 = add(mul(c10, 1.0 - s), mul(c11, s));

        let c = add(mul(c0, 1.0 - t), mul(c1, t));

        let p = window.at_2d_mut::<core::Vec3b>(point.x as i32, point.y as i32)?;
        *p = core::Vec3b::from([c[0] as u8, c[1] as u8, c[2] as u8]);
    }

    Ok(())
}

pub fn bezier(points: &Vec<core::Point2i>, window: &mut Mat) -> opencv::Result<()> {
    let points = points.iter().map(to_point2d).collect();

    let mut bezier_points = vec![];
    let mut t = 0f64;
    while t <= 1.0 {
        let point = recursive_bezier(&points, t)?;
        window.at_2d_mut::<core::Vec3b>(point.y as i32, point.x as i32)?[1] = 255;
        bezier_points.push(point);
        t += 0.0001;
    }

    // println!("points len is {}", bezier_points.len());
    bilinear(bezier_points, window)?;
    Ok(())
}
