use bevy::{
    color::Color,
    math::{Vec2, Vec3},
};

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
