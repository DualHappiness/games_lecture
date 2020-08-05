mod global;
mod light;
mod objects;
mod render;
mod scene;
mod ray;
mod intersection;
mod material;
mod bound;
mod bvh;
mod sah;

pub mod obj_loader;

pub use ray::*;
pub use global::*;
pub use light::*;
pub use objects::*;
pub use render::*;
pub use scene::*;
pub use intersection::*;
pub use material::*;
pub use bound::*;
pub use bvh::*;
pub use sah::*;

#[macro_export]
macro_rules! vector3 {
    {$($v:expr),*} => {{
        let v = vec![$($v),*];
        nalgebra::Vector3::from_column_slice(&v.as_slice()[0..3])
    }};
}

#[macro_export]
macro_rules! vector2 {
    {$($v:expr),*} => {{
        let v = vec![$($v),*];
        nalgebra::Vector2::from_column_slice(&v.as_slice()[0..2])
    }};
}
