use bevy::prelude::*;
use rand::Rng;

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
    pub shape: soft_sphere::Shape,
}

impl Plate {
    pub fn random(plate_type: PlateType, rng: &mut rand::rngs::StdRng) -> Self {
        let plate_color = LinearRgba::new(rng.random(), rng.random(), rng.random(), 1.).into();
        Plate {
            plate_type: plate_type.clone(),
            color: plate_color,
            axis_of_rotation: Vec3::new(
                rng.random_range(-1.0..1.0),
                rng.random_range(-1.0..1.0),
                rng.random_range(-1.0..1.0),
            )
            .normalize(),
            drift_direction: Vec2::new(rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0))
                .normalize(),
            shape: soft_sphere::Shape::new(),
        }
    }
}
