use std::collections::{HashMap, HashSet};

use bevy::{
    ecs::resource::Resource,
    math::{EulerRot, Quat, Vec2, Vec3},
};
use rand::Rng;

use crate::{
    particle_sphere::ParticleSphere,
    plate::{Plate, PlateType},
};

pub const OCEANIC_PARTICLE_MASS: f32 = 1.;
pub const OCEANIC_PARTICLE_HEIGHT: f32 = 0.98;
pub const CONTINENTAL_PARTICLE_MASS: f32 = 5.;
pub const CONTINENTAL_PARTICLE_HEIGHT: f32 = 1.02;

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
    /// Spring constant used for particle links
    pub spring_constant: f32,
    // Dampener coefficient for the spring forces, used to dampen oscillations
    pub dampener_coefficient: f32,
    /// Modifier to the force applies by the plate rotational axis to plate particles.
    pub plate_force_modifier: f32,
    /// The rate at which the plate axis of rotation drifts in position
    pub plate_rotation_drift_rate: f32,
    pub timestep: f32,
    pub iterations: usize,
    // Friction between plate particles and mantle
    pub friction_coefficient: f32,
}

struct PlateBuilder {
    plate: Plate,
    tile_to_point_mass: HashMap<usize, usize>,
}

impl PlateBuilder {
    fn new(plate: Plate) -> Self {
        Self {
            plate,
            tile_to_point_mass: HashMap::new(),
        }
    }
    fn add_point_mass(
        &mut self,
        tile_index: usize,
        point_mass: soft_sphere::PointMass,
        particle_sphere: &ParticleSphere,
        config: &TectonicsConfiguration,
    ) {
        let point_mass_index = self.plate.shape.point_masses.len();
        self.plate.shape.point_masses.push(point_mass);
        self.tile_to_point_mass.insert(tile_index, point_mass_index);
        // Add springs to already-added adjacent tiles (if they are in this plate)
        for adj_tile in &particle_sphere.tiles[tile_index].adjacent {
            if let Some(&adj_index) = self.tile_to_point_mass.get(adj_tile) {
                let rest_length = self.plate.shape.point_masses[point_mass_index]
                    .geodesic_distance(&self.plate.shape.point_masses[adj_index]);
                self.plate.shape.springs.push(soft_sphere::Spring {
                    anchor_a: point_mass_index,
                    anchor_b: adj_index,
                    rest_length,
                    spring_constant: config.spring_constant,
                    damping_coefficient: config.dampener_coefficient,
                });
            }
        }
    }
}

#[derive(Resource)]
pub struct Tectonics {
    pub config: TectonicsConfiguration,
    /// Average distance if all particles were spaced out evenly
    pub ideal_distance: f32,
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

        let mut plate_builders: Vec<PlateBuilder> = Vec::new();
        let ideal_distance = f32::acos(1. - 2. / particle_sphere.tiles.len() as f32) * 2.;

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
        let mut available_tiles: HashSet<usize> = (0..particle_sphere.tiles.len()).collect();
        available_tiles.remove(&starting_tile);
        let mut adjacent_tiles = vec![starting_tile];

        while available_tiles.len() > 0 || adjacent_tiles.len() > 0 {
            let plate_type = if ((tile_count - available_tiles.len()) as f32 / tile_count as f32)
                < config.continental_rate
            {
                PlateType::Continental
            } else {
                PlateType::Oceanic
            };
            let mut builder = PlateBuilder::new(Plate::random(plate_type, rng));
            let tiles_to_take = if (generated_majors as f32 / generated_minors as f32)
                > config.major_plate_fraction
            {
                generated_minors += 1;
                minor_tile_count
            } else {
                generated_majors += 1;
                major_tile_count
            };

            // Add random adjacent tile, add thats tile to the surrounding unvisited tiles
            for _ in 0..tiles_to_take {
                // No unvisited tiles left
                if adjacent_tiles.is_empty() {
                    break;
                }
                // Chose tile, remember it has been used and remove from adjacent unvisited
                let random_adjacent_tile: usize =
                    adjacent_tiles.swap_remove(rng.random_range(0..adjacent_tiles.len()));
                let mass = if plate_type == PlateType::Continental {
                    CONTINENTAL_PARTICLE_MASS
                } else {
                    OCEANIC_PARTICLE_MASS
                };
                let point_mass = soft_sphere::PointMass::new(
                    particle_sphere.tiles[random_adjacent_tile].normal,
                    mass,
                );
                builder.add_point_mass(random_adjacent_tile, point_mass, particle_sphere, &config);
                adjacent_tiles.extend(
                    particle_sphere.tiles[random_adjacent_tile]
                        .adjacent
                        .iter()
                        .filter(|index| available_tiles.remove(index)),
                );
            }
            if builder.plate.shape.point_masses.len() >= config.min_plate_size {
                plate_builders.push(builder);
            } else if !builder.plate.shape.point_masses.is_empty() {
                // Plate is too small, merge into closest plate
                let closest_plate_builder = plate_builders
                    .iter_mut()
                    .min_by(|pb_a, pb_b| {
                        let closest_point_mass_a = pb_a
                            .plate
                            .shape
                            .point_masses
                            .iter()
                            .map(|point_mass| {
                                point_mass.geodesic_distance(&builder.plate.shape.point_masses[0])
                            })
                            .min_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap();
                        let closest_point_mass_b = pb_b
                            .plate
                            .shape
                            .point_masses
                            .iter()
                            .map(|point_mass| {
                                point_mass.geodesic_distance(&builder.plate.shape.point_masses[0])
                            })
                            .min_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap();
                        closest_point_mass_a
                            .partial_cmp(&closest_point_mass_b)
                            .expect("Failed to compare point mass distances, check for NaN")
                    })
                    .expect("Failed to find closest plate when plate was too small");
                // For each point mass in the too-small plate, add to closest plate and add springs
                for (&tile_index, &pm_index) in builder.tile_to_point_mass.iter() {
                    let point_mass = &builder.plate.shape.point_masses[pm_index];
                    let new_index = closest_plate_builder.plate.shape.point_masses.len();
                    closest_plate_builder
                        .plate
                        .shape
                        .point_masses
                        .push(soft_sphere::PointMass {
                            position: point_mass.position,
                            mass: if closest_plate_builder.plate.plate_type
                                == PlateType::Continental
                            {
                                CONTINENTAL_PARTICLE_MASS
                            } else {
                                OCEANIC_PARTICLE_MASS
                            },
                            velocity: Vec3::ZERO,
                            force: Vec3::ZERO,
                            prev_force: Vec3::ZERO,
                        });
                    closest_plate_builder
                        .tile_to_point_mass
                        .insert(tile_index, new_index);
                    for adj_tile in &particle_sphere.tiles[tile_index].adjacent {
                        if let Some(&adjacent_index) =
                            closest_plate_builder.tile_to_point_mass.get(adj_tile)
                        {
                            let rest_length = closest_plate_builder.plate.shape.point_masses
                                [new_index]
                                .geodesic_distance(
                                    &closest_plate_builder.plate.shape.point_masses[adjacent_index],
                                );
                            closest_plate_builder
                                .plate
                                .shape
                                .springs
                                .push(soft_sphere::Spring {
                                    anchor_a: new_index,
                                    anchor_b: adjacent_index,
                                    rest_length,
                                    spring_constant: config.spring_constant,
                                    damping_coefficient: config.dampener_coefficient,
                                });
                        }
                    }
                }
            }

            // Return adjacent tiles to available tiles, pick a new starting point
            available_tiles.extend(adjacent_tiles.drain(..));
            if available_tiles.len() > 0 {
                let available_tiles_vec: Vec<usize> = available_tiles.iter().cloned().collect();
                let starting_tile =
                    available_tiles_vec[rng.random_range(0..available_tiles_vec.len())];
                available_tiles.remove(&starting_tile);
                adjacent_tiles.push(starting_tile);
            }
        }

        let point_mass_count = plate_builders
            .iter()
            .map(|pb| pb.plate.shape.point_masses.len())
            .sum::<usize>();
        assert!(
            point_mass_count == particle_sphere.tiles.len(),
            "Point mass count {} not same as Particle Tile {} count!",
            point_mass_count,
            particle_sphere.tiles.len()
        );

        Tectonics {
            config,
            plates: plate_builders.drain(..).map(|pb| pb.plate).collect(),
            ideal_distance,
        }
    }

    // Each particle will be forced to have the velocity matching rotation around the ownings plate axis of rotation
    // Then we adjust that velocity depending on other particles
    pub fn simulate(&mut self, rng: &mut rand::rngs::StdRng) {
        // Apply forces and update velocity and position
        for plate in &mut self.plates {
            plate.shape.apply_external_force(|point_mass| {
                let plate_force = plate
                    .axis_of_rotation
                    .cross(point_mass.position)
                    * self.config.plate_force_modifier
                    // We make this force mass independent so oceanic and continental plates move equally
                    * point_mass.mass;
                let friction_force = if point_mass.velocity.length() > 0. {
                    -point_mass.velocity * point_mass.mass * self.config.friction_coefficient
                } else {
                    Vec3::ZERO
                };
                plate_force + friction_force
            });
            plate.shape.apply_spring_forces();
            // TODO: Update and add frame forces to maintain shape
            // TODO: Simulate collisions
            plate.shape.update(self.config.timestep);
        }
        // Randomly modify each plates axis of rotation slightly
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
