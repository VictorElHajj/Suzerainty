use glam::{Quat, Vec3};

use crate::{point_mass::PointMass, spring::Spring};

pub struct Shape {
    pub point_masses: Vec<PointMass>,
    pub springs: Vec<Spring>,
    centroid: Vec3,
    bounding_distance: f32,
}

impl Shape {
    pub fn new() -> Self {
        Shape {
            point_masses: Vec::new(),
            springs: Vec::new(),
            centroid: Vec3::NAN,
            bounding_distance: f32::NAN,
        }
    }
    fn zero_forces(&mut self) {
        for point_mass in &mut self.point_masses {
            point_mass.prev_force = point_mass.force;
            point_mass.force = Vec3::ZERO;
        }
    }

    pub fn apply_spring_forces(&mut self) {
        for spring in &self.springs {
            spring.apply_force(&mut self.point_masses);
        }
    }

    pub fn apply_external_force<F>(&mut self, function: F)
    where
        F: Fn(&PointMass) -> Vec3,
    {
        for point_mass in &mut self.point_masses {
            point_mass.force += function(&point_mass);
        }
    }

    // Integrate forces with velocity verlet integration and update point mass positions
    pub fn update(&mut self, timestep: f32) {
        for point_mass in &mut self.point_masses {
            let old_acc = point_mass.prev_force / point_mass.mass;
            let new_acc = point_mass.force / point_mass.mass;
            let displacement = point_mass.velocity * timestep + 0.5 * old_acc * timestep.powi(2);

            // Project displacement onto tangent plane of point mass
            let tangent_disp =
                displacement - displacement.dot(point_mass.position) * point_mass.position;

            let angle = tangent_disp.length();
            if angle > 0.0 {
                let axis = point_mass.position.cross(tangent_disp).normalize();
                let rot = Quat::from_axis_angle(axis, angle);
                // Normalize to avoid error build up, point masses are constrained to the unit sphere
                point_mass.position = (rot * point_mass.position).normalize();
            }
            point_mass.velocity = point_mass.velocity + (old_acc + new_acc) / 2. * timestep;
        }
        self.zero_forces();
        self.update_centroid();
        self.update_bounding_distance();
    }

    /// Calculate the shapes average point
    pub fn update_centroid(&mut self) {
        self.centroid = Vec3::ZERO;
        for point_mass in &self.point_masses {
            self.centroid += point_mass.position / self.point_masses.len() as f32;
        }
    }

    pub fn update_bounding_distance(&mut self) {
        self.bounding_distance = self
            .point_masses
            .iter()
            .map(|pm| f32::acos(pm.position.dot(self.centroid).clamp(-1., 1.)))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    pub fn within_bounding_spherical_cap(&self, position: Vec3) -> bool {
        f32::acos(position.dot(self.centroid).clamp(-1., 1.)) < self.bounding_distance
    }

    /// Returns an iterator going over each point mass and the springs it is an anchor for.
    pub fn iter_point_masses_with_springs(
        &self,
    ) -> impl Iterator<Item = (&PointMass, impl Iterator<Item = &Spring>)> {
        self.point_masses.iter().enumerate().map(|(i, point_mass)| {
            let springs = self
                .springs
                .iter()
                .filter(move |spring| spring.anchor_a == i || spring.anchor_b == i);
            (point_mass, springs)
        })
    }

    // pub fn apply frame force

    // pub fn get shape/hull from grahams method
}
