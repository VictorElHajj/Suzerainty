use std::collections::HashSet;

use bevy::{
    color::LinearRgba,
    ecs::resource::Resource,
    math::{EulerRot, Quat, Vec2, Vec3},
};
use rand::Rng;
use rayon::iter::ParallelIterator;

use crate::{
    particle_sphere::ParticleSphere,
    plate::{Plate, PlateParticle, PlateType},
    sphere_bins::{GetNormal, SphereBins},
};

const OCEANIC_PARTICLE_MASS: f32 = 1.;
const OCEANIC_PARTICLE_HEIGHT: f32 = 0.98;
const CONTINENTAL_PARTICLE_MASS: f32 = 5.;
const CONTINENTAL_PARTICLE_HEIGHT: f32 = 1.02;

pub const BIN_COUNT: usize = 60;

#[derive(Clone, Copy)]
pub struct TectonicsConfiguration {
    /// How many plates the simulation tries to create
    pub plate_goal: usize,
    /// How many major compared to minor plates
    pub major_plate_fraction: f32,
    /// [0,1] Ratio of total tiles assigned to major plates vs minor plates
    pub major_tile_fraction: f32,
    /// [0,1] Ratio of plates that are continental vs oceanic
    pub continental_rate: f32,
    /// Smallest amount of particles allowed on a plate, if fewer the plate is merged with another
    pub min_plate_size: usize,
    /// Radius which describes the maximum distance at which particles interact
    pub particle_force_radius: f32,
    /// Modifier to the plate particle repulsive force, is 4x to particles of other plates
    pub repulsive_force_modifier: f32,
    /// Modifier to the plate particle attractive force, only works on particles of same plate
    pub attractive_force: f32,
    /// Modifier to the force applies by the plate rotational axis to plate particles.
    pub plate_force_modifier: f32,
    /// The rate at which the plate axis of rotation drifts in position
    pub plate_rotation_drift_rate: f32,
    pub timestep: f32,
    pub iterations: usize,
    // Friction between plate particles and mantle
    pub friction_coefficient: f32,
}

#[derive(Resource)]
pub struct Tectonics {
    pub config: TectonicsConfiguration,
    pub particles: SphereBins<BIN_COUNT, PlateParticle>,
    pub plates: Vec<Plate>,
}

impl Tectonics {
    pub fn from_config(
        config: TectonicsConfiguration,
        particle_sphere: &ParticleSphere,
        rng: &mut rand::rngs::StdRng,
    ) -> Self {
        assert!((0.0..=1.0).contains(&config.major_tile_fraction));
        assert!((0.0..=1.0).contains(&config.major_plate_fraction));
        assert!((0.0..=1.0).contains(&config.continental_rate));

        let mut particles = SphereBins::<BIN_COUNT, PlateParticle>::new();
        let mut plates = Vec::new();

        let mut generated_majors = 0;
        let mut generated_minors = 0;

        let tile_count = particle_sphere.tiles.len();
        let major_tile_count: usize = (tile_count as f32 * config.major_tile_fraction
            / (config.plate_goal as f32 / 2.)
            / config.major_plate_fraction) as usize;
        let minor_tile_count: usize = (tile_count as f32 * (1. - config.major_tile_fraction)
            / (config.plate_goal as f32 / 2.)
            / (1. - config.major_plate_fraction)) as usize;

        let starting_tile = rng.random_range(0..particle_sphere.tiles.len());
        let mut global_surrounding_unvisited_tiles = Vec::<usize>::new();
        let mut next_surrounding_unvisited_tiles = vec![starting_tile];
        let mut added_tiles = HashSet::<usize>::new();
        added_tiles.insert(starting_tile);

        while added_tiles.len() < particle_sphere.tiles.len() {
            let plate_color = LinearRgba::new(rng.random(), rng.random(), rng.random(), 1.).into();
            let plate_type =
                if (added_tiles.len() as f32 / tile_count as f32) < config.continental_rate {
                    PlateType::Continental
                } else {
                    PlateType::Oceanic
                };
            let plate = Plate {
                plate_type: plate_type.clone(),
                color: plate_color,
                axis_of_rotation: Vec3::new(
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                )
                .normalize(),
                drift_direction: Vec2::new(
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                )
                .normalize(),
            };
            // Generate minor plate if we have more major plates than minor plates but there are still minor plates left to generate
            let tiles_to_take = if (generated_majors as f32 / generated_minors as f32)
                > config.major_plate_fraction
            {
                generated_minors += 1;
                minor_tile_count
            } else {
                generated_majors += 1;
                major_tile_count
            };

            // Temp particle list, if resulting plate has too few particles merge with closest
            let mut temp_particle_vec = Vec::<PlateParticle>::new();

            // Add random adjacent tile, add thats tile to the surrounding unvisited tiles
            for _ in 0..tiles_to_take {
                if next_surrounding_unvisited_tiles.is_empty() {
                    break;
                }
                // Chose tile, remember it has been used and remove from adjacent unvisited
                let random_adjacent_tile_index: usize =
                    rng.random_range(0..next_surrounding_unvisited_tiles.len());
                added_tiles.insert(next_surrounding_unvisited_tiles[random_adjacent_tile_index]);
                let chosen_tile = &particle_sphere.tiles
                    [next_surrounding_unvisited_tiles.swap_remove(random_adjacent_tile_index)];

                // Create particle from chosen tile
                temp_particle_vec.push(PlateParticle {
                    position: chosen_tile.normal,
                    height: if plate_type == PlateType::Continental {
                        CONTINENTAL_PARTICLE_HEIGHT
                    } else {
                        OCEANIC_PARTICLE_HEIGHT
                    },
                    plate_index: plates.len(),
                    mass: if plate_type == PlateType::Continental {
                        CONTINENTAL_PARTICLE_MASS
                    } else {
                        OCEANIC_PARTICLE_MASS
                    },
                    velocity: Vec3::ZERO,
                    acceleration: Vec3::ZERO,
                    id: chosen_tile.index,
                });

                // Update univisted tiles with new adjacents
                next_surrounding_unvisited_tiles.extend(
                    chosen_tile
                        .adjacent
                        .iter()
                        .filter(|index| !added_tiles.contains(*index)),
                );
            }

            if temp_particle_vec.len() >= config.min_plate_size {
                plates.push(plate);
                for particle in temp_particle_vec {
                    particles.insert(particle);
                }
            } else if !temp_particle_vec.is_empty() {
                // Find closest existing plate
                let closest_plate_index = particles
                    .get_closest(temp_particle_vec[0].normal())
                    .plate_index;
                for particle in temp_particle_vec {
                    particles.insert(PlateParticle {
                        position: particle.position,
                        height: if plates[closest_plate_index].plate_type == PlateType::Continental
                        {
                            CONTINENTAL_PARTICLE_HEIGHT
                        } else {
                            OCEANIC_PARTICLE_HEIGHT
                        },
                        plate_index: closest_plate_index,
                        mass: if plates[closest_plate_index].plate_type == PlateType::Continental {
                            CONTINENTAL_PARTICLE_MASS
                        } else {
                            OCEANIC_PARTICLE_MASS
                        },
                        velocity: Vec3::ZERO,
                        acceleration: Vec3::ZERO,
                        id: particle.id,
                    });
                }
            }

            // Add remaining unvisited to global unvisited, update to remove used ones.
            global_surrounding_unvisited_tiles.extend(&next_surrounding_unvisited_tiles);
            global_surrounding_unvisited_tiles.retain(|index| !added_tiles.contains(index));
            // Pick a new starting point for the global unvisited, if there are tiles left
            if !(added_tiles.len() == particle_sphere.tiles.len()) {
                next_surrounding_unvisited_tiles = vec![
                    global_surrounding_unvisited_tiles
                        [rng.random_range(0..global_surrounding_unvisited_tiles.len())],
                ];
            }
        }
        Tectonics {
            config,
            particles,
            plates,
        }
    }

    // Each particle will be forced to have the velocity matching rotation around the ownings plate axis of rotation
    // Then we adjust that velocity depending on other particles
    pub fn simulate(&mut self, rng: &mut rand::rngs::StdRng) {
        // 1. Calculate acceleration for each particle
        let new_particle_accelerations: Vec<Vec3> = self
            .particles
            .par_iter()
            .map(|particle| {
                let plate_force = self.plates[particle.plate_index]
                    .axis_of_rotation
                    .cross(particle.position)
                    * self.config.plate_force_modifier
                    // We make this force mass independent so oceanic and continental plates move equally
                    * particle.mass;
                let friction_force = if particle.velocity.length() > 0. {
                    -particle.velocity * particle.mass * self.config.friction_coefficient
                } else {
                    Vec3::ZERO
                };

                let mut interaction_force = Vec3::ZERO;
                for other_particle in self
                    .particles
                    .get_within(particle.position, self.config.particle_force_radius)
                {
                    if particle.id == other_particle.id {
                        continue;
                    }
                    let geodesic_distance =
                        f32::acos(particle.position.dot(other_particle.position));
                    let repulsive_force = if particle.plate_index == other_particle.plate_index {
                        1. / (geodesic_distance / self.config.repulsive_force_modifier).powi(2)
                    } else {
                        1. / (geodesic_distance / self.config.repulsive_force_modifier).powi(2) * 2.
                    };
                    let attraction_force = if particle.plate_index == other_particle.plate_index {
                        self.config.attractive_force
                    } else {
                        0.
                    };
                    interaction_force += (repulsive_force - attraction_force)
                        * (particle.position - other_particle.position);
                    if particle.plate_index != other_particle.plate_index {
                        // TODO this is where we do plate interactions
                    }
                }
                (plate_force + interaction_force + friction_force) / particle.mass
            })
            .collect();
        // 2. Apply forces and update velocity and position
        // We used a Velocity Verlet integration
        for (i, particle) in self.particles.iter_mut().enumerate() {
            let displacement = particle.velocity * self.config.timestep
                + 0.5 * particle.acceleration * self.config.timestep.powi(2);
            // Project displacement onto tangent plane of current position
            let tangent_disp =
                displacement - particle.position * displacement.dot(particle.position);
            let angle = tangent_disp.length();
            if angle > 0.0 {
                let axis = particle.position.cross(tangent_disp).normalize();
                let rot = Quat::from_axis_angle(axis, angle);
                particle.position = rot * particle.position;
            }
            particle.velocity = particle.velocity
                + (particle.acceleration + new_particle_accelerations[i]) / 2.
                    * self.config.timestep;
            particle.acceleration = new_particle_accelerations[i];
        }
        // 3. Update the sphere bin datastructure as some particles might leave their current bin
        self.particles.refresh();
        // 4. Randomly modify each plates axis of rotation slightly
        for plate in self.plates.iter_mut() {
            plate.drift_direction = (plate.drift_direction
                + Vec2::new(
                    rng.random_range(-1.0..1.0) * self.config.plate_rotation_drift_rate,
                    rng.random_range(-1.0..1.0) * self.config.plate_rotation_drift_rate,
                ) * self.config.timestep)
                .normalize();
            plate.axis_of_rotation = Quat::from_euler(
                EulerRot::XYZ,
                plate.drift_direction.x * self.config.plate_rotation_drift_rate,
                plate.drift_direction.y * self.config.plate_rotation_drift_rate,
                0.,
            ) * plate.axis_of_rotation;
        }
    }
}
