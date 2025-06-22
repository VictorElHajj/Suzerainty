use bevy::color::Color;

pub enum PlateType {
    Oceanic,
    Continental,
}

pub struct Plate {
    pub plate_type: PlateType,
    pub color: Color,
}
