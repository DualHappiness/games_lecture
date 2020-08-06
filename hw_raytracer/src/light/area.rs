use crate::global::*;
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct AreaLight {
    pub base: super::Light,
    pub length: f32,
    pub normal: Vector3f,
    pub u: Vector3f,
    pub v: Vector3f,
}

impl Deref for AreaLight {
    type Target = super::Light;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for AreaLight {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl AreaLight {
    pub fn new(p: &Vector3f, i: &Vector3f) -> Self {
        Self {
            normal: -Vector3f::y(),
            u: Vector3f::x(),
            v: Vector3f::z(),
            length: 100f32,
            base: super::Light::new(p.clone(), i.clone()),
            ..Self::default()
        }
    }
}
