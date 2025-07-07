use crate::sphere_bins::Binnable;
use bevy::prelude::*;

#[derive(PartialEq, Clone, Copy)]
pub enum PlateType {
    Oceanic,
    Continental,
}

pub struct Plate {
    pub plate_type: PlateType,
    pub color: Color,
    pub axis_of_rotation: Vec3,
    pub drift_direction: Vec2,
}

#[derive(PartialEq, Clone, Copy)]
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

impl Binnable for PlateParticle {
    #[inline]
    fn normal(&self) -> Vec3 {
        self.position
    }

    #[inline]
    fn id(&self) -> usize {
        self.id
    }
}
