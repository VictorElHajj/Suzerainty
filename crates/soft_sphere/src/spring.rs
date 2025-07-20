use crate::point_mass::PointMass;

pub struct Spring {
    /// Index to PointMass
    pub anchor_a: usize,
    /// Index to PointMass
    pub anchor_b: usize,
    pub rest_length: f32,
    pub spring_constant: f32,
    pub damping_coefficient: f32,
}

impl Spring {
    /// Calculate the spring-dampener system force on [self]
    pub fn apply_force(&self, point_masses: &mut Vec<PointMass>) {
        let point_a = &point_masses[self.anchor_a];
        let point_b = &point_masses[self.anchor_b];

        let distance = point_a.geodesic_distance(&point_b);
        if distance == 0.0 {
            return;
        }

        let direction = (point_a.position - point_b.position) / distance;
        let relative_velocity = point_a.velocity - point_b.velocity;
        let velocity_towards = relative_velocity.dot(direction);

        let force = (-self.spring_constant * (distance - self.rest_length)
            - self.damping_coefficient * velocity_towards)
            * direction;

        // Project force onto point_a tangent plane
        let force_on_a = force - force.dot(point_a.position) * point_a.position;
        let force_on_b = (-force) - (-force).dot(point_b.position) * point_b.position;

        point_masses[self.anchor_a].force += force_on_a;
        point_masses[self.anchor_b].force += force_on_b;
    }
}
