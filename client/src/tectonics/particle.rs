use bevy::prelude::*;

use crate::sphere_bins::GetNormal;

pub struct PlateParticle {
    /// Unit sphere normal
    pub position: Vec3,
    pub height: f32,
    /// Index to plate
    pub plate_index: usize,
}

impl GetNormal for PlateParticle {
    fn normal(&self) -> Vec3 {
        self.position
    }
}
