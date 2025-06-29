use bevy::prelude::*;

use crate::sphere_bins::GetNormal;

#[derive(PartialEq)]
pub struct PlateParticle {
    /// Unit sphere normal
    pub position: Vec3,
    pub height: f32,
    /// Index to plate
    pub plate_index: usize,
    pub mass: f32,
    /// Velicity
    pub velocity: Vec3,
    /// Acceleration
    pub acceleration: Vec3,
    pub id: usize,
}

impl GetNormal for PlateParticle {
    #[inline]
    fn normal(&self) -> Vec3 {
        self.position
    }
}
