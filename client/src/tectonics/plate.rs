use bevy::{color::Color, math::Vec3};

#[derive(PartialEq, Clone, Copy)]
pub enum PlateType {
    Oceanic,
    Continental,
}

pub struct Plate {
    pub plate_type: PlateType,
    pub color: Color,
    /// All particles within this plate will have a constant force applied to make the particles rotate around this axis
    /// TODO: This should very over time
    pub axis_of_rotation: Vec3,
}
