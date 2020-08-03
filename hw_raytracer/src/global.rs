use rand;
use std::cmp::Ordering;

use nalgebra::{Vector2, Vector3};
pub use std::rc::Rc;
pub use std::cell::RefCell;
pub type Vector2f = Vector2<f32>;
pub type Vector3f = Vector3<f32>;

pub const PI: f32 = 3.14159265358979323846;
pub const INFINITY: f32 = f32::MAX;

pub fn clamp(low: f32, hi: f32, v: f32) -> f32 {
    low.max(hi.min(v))
}

pub fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
    match b * b - 4f32 * a * c {
        0f32 => Some((-0.5 * b / a, -0.5 * b / a)),
        discr if discr < 0f32 => None,
        discr => {
            let q = if b > 0f32 {
                -0.5 * (b + discr.sqrt())
            } else {
                -0.5 * (b - discr.sqrt())
            };
            let mut x0 = q / a;
            let mut x1 = c / q;
            if x0 > x1 {
                std::mem::swap(&mut x0, &mut x1);
            }
            Some((x0, x1))
        }
    }
}

pub enum MaterialType {
    DiffuseAndGlossy,
    ReflectionAndRefraction,
    Reflection,
}

impl Default for MaterialType {
    fn default() -> Self {
        MaterialType::DiffuseAndGlossy
    }
}

pub fn get_random_float() -> f32 {
    rand::random::<f32>()
}

pub fn update_progress(progress: f32) {
    let bar_width = 70;

    print!("[");
    for i in 0..bar_width {
        match i.cmp(&((bar_width as f32 * progress) as i32)) {
            Ordering::Less => print!("="),
            Ordering::Equal => print!(">"),
            Ordering::Greater => print!(" "),
        }
        print!("] {} %\r", (progress * 100f32) as i32);
    }
}
