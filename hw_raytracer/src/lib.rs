mod global;
mod light;
mod objects;
mod render;
mod scene;

pub use global::*;
pub use light::*;
pub use objects::*;
pub use render::*;
pub use scene::*;

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
