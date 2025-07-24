use glam::Vec3;

#[derive(PartialEq)]
pub struct PointMass {
    pub position: Vec3,
    pub velocity: Vec3,
    pub prev_force: Vec3, // Accumulated force in previous update, used for velocity verlet integration
    pub force: Vec3,      // Accumulated force for the next update
    pub mass: f32,
}

impl PointMass {
    pub fn new(position: Vec3, mass: f32) -> Self {
        PointMass {
            position,
            velocity: Vec3::ZERO,
            prev_force: Vec3::ZERO,
            force: Vec3::ZERO,
            mass,
        }
    }
    pub fn geodesic_distance(&self, other: &Self) -> f32 {
        f32::acos(self.position.dot(other.position).clamp(-1., 1.))
    }
}
