use rayon::prelude::*;
use std::f32::consts::PI;

use bevy::{platform::collections::HashSet, prelude::*};
use rand::Rng;

use crate::{
    GlobalRng,
    debug_ui::DebugDiagnostics,
    hex_sphere::{CurrentMousePick, MousePickInfo},
    sphere_bins::{GetNormal, SphereBins},
    states::SimulationState,
    tectonics::{
        ParticleSphere, ParticleSphereConfig,
        particle::PlateParticle,
        plate::{Plate, PlateType},
        vertex_interpolation::interpolate_vertices,
    },
};

const OCEANIC_PARTICLE_MASS: f32 = 1.;
const OCEANIC_PARTICLE_HEIGHT: f32 = 0.98;
const CONTINENTAL_PARTICLE_MASS: f32 = 5.;
const CONTINENTAL_PARTICLE_HEIGHT: f32 = 1.02;

#[derive(Resource, Clone, Copy)]
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
pub struct TectonicsIteration(pub usize);

pub struct TectonicsPlugin {
    pub tectonics_config: TectonicsConfiguration,
    pub particle_config: ParticleSphereConfig,
}
impl Plugin for TectonicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.tectonics_config)
            .insert_resource(ParticleSphere::from_config(self.particle_config))
            .insert_resource(TectonicsIteration(0))
            .add_systems(OnEnter(SimulationState::Tectonics), setup)
            .add_systems(
                Update,
                (
                    draw_particles,
                    draw_bins,
                    simulate.run_if(in_state(SimulationState::Tectonics)),
                    interpolate_vertices.run_if(in_state(SimulationState::Tectonics)),
                ),
            );
    }
}

pub const BIN_COUNT: usize = 60;
#[derive(Resource)]
pub struct PlateParticles(
    pub crate::sphere_bins::SphereBins<BIN_COUNT, super::particle::PlateParticle>,
);

#[derive(Resource)]
struct Plates(Vec<Plate>);

// This should be the square root of the particle count
// const BIN_COUNT: usize = 60;

#[derive(Resource)]
struct TectonicsStartTime(std::time::Instant);

fn setup(
    mut commands: Commands,
    particle_sphere: Res<ParticleSphere>,
    tectonics_config: Res<TectonicsConfiguration>,
    mut rng: ResMut<GlobalRng>,
) {
    commands.insert_resource(TectonicsStartTime(std::time::Instant::now()));
    assert!((0.0..=1.0).contains(&tectonics_config.major_tile_fraction));
    assert!((0.0..=1.0).contains(&tectonics_config.major_plate_fraction));
    assert!((0.0..=1.0).contains(&tectonics_config.continental_rate));
    let mut generated_majors = 0;
    let mut generated_minors = 0;
    let mut plates = Vec::<Plate>::new();
    let mut particle_bins = SphereBins::<BIN_COUNT, PlateParticle>::new();

    let tile_count = particle_sphere.tiles.len();
    let major_tile_count: usize = (tile_count as f32 * tectonics_config.major_tile_fraction
        / (tectonics_config.plate_goal as f32 / 2.)
        / tectonics_config.major_plate_fraction) as usize;
    let minor_tile_count: usize = (tile_count as f32 * (1. - tectonics_config.major_tile_fraction)
        / (tectonics_config.plate_goal as f32 / 2.)
        / (1. - tectonics_config.major_plate_fraction)) as usize;

    let starting_tile = rng.0.random_range(0..particle_sphere.tiles.len());
    let mut global_surrounding_unvisited_tiles = Vec::<usize>::new();
    let mut next_surrounding_unvisited_tiles = vec![starting_tile];
    let mut added_tiles = HashSet::<usize>::new();
    added_tiles.insert(starting_tile);

    while added_tiles.len() < particle_sphere.tiles.len() {
        let plate_color =
            LinearRgba::new(rng.0.random(), rng.0.random(), rng.0.random(), 0.1).into();
        let plate_type =
            if (added_tiles.len() as f32 / tile_count as f32) < tectonics_config.continental_rate {
                PlateType::Continental
            } else {
                PlateType::Oceanic
            };
        let plate = Plate {
            plate_type: plate_type.clone(),
            color: plate_color,
            axis_of_rotation: Vec3::new(
                rng.0.random_range(-1.0..1.0),
                rng.0.random_range(-1.0..1.0),
                rng.0.random_range(-1.0..1.0),
            )
            .normalize(),
            drift_direction: Vec2::new(
                rng.0.random_range(-1.0..1.0),
                rng.0.random_range(-1.0..1.0),
            )
            .normalize(),
        };
        // Generate minor plate if we have more major plates than minor plates but there are still minor plates left to generate
        let tiles_to_take = if (generated_majors as f32 / generated_minors as f32)
            > tectonics_config.major_plate_fraction
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
            let random_adjacent_tile_index: usize = rng
                .0
                .random_range(0..next_surrounding_unvisited_tiles.len());
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

        if temp_particle_vec.len() >= tectonics_config.min_plate_size {
            plates.push(plate);
            for particle in temp_particle_vec {
                particle_bins.insert(particle);
            }
        } else if !temp_particle_vec.is_empty() {
            // Find closest existing plate
            let closest_plate_index = particle_bins
                .get_closest(temp_particle_vec[0].normal())
                .plate_index;
            for particle in temp_particle_vec {
                particle_bins.insert(PlateParticle {
                    position: particle.position,
                    height: if plates[closest_plate_index].plate_type == PlateType::Continental {
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
                global_surrounding_unvisited_tiles[rng
                    .0
                    .random_range(0..global_surrounding_unvisited_tiles.len())],
            ];
        }
    }
    commands.insert_resource(PlateParticles(particle_bins));
    commands.insert_resource(Plates(plates));
}

fn draw_particles(
    mut gizmos: Gizmos,
    plate_particles: Res<PlateParticles>,
    plates: Res<Plates>,
    particle_sphere: Res<ParticleSphere>,
) {
    for plate in &plates.0 {
        gizmos.arrow(
            plate.axis_of_rotation,
            plate.axis_of_rotation * 1.1,
            plate.color,
        );
    }
    for particle in plate_particles
        .0
        .bins
        .iter()
        .flat_map(|bin| bin.items.iter())
    {
        gizmos.cross(
            Isometry3d {
                translation: (particle.position * particle.height).into(),
                rotation: Quat::from_rotation_arc(Vec3::Z, particle.position),
            },
            16. * PI / particle_sphere.tiles.len() as f32,
            plates.0[particle.plate_index].color,
        );
    }
}

// Each particle will be forced to have the velocity matching rotation around the ownings plate axis of rotation
// Then we adjust that velocity depending on other particles
fn simulate(
    mut plate_particles: ResMut<PlateParticles>,
    mut plates: ResMut<Plates>,
    tectonics_config: Res<TectonicsConfiguration>,
    mut rng: ResMut<GlobalRng>,
    mut tectonics_iteration: ResMut<TectonicsIteration>,
    tectonics_start_time: Res<TectonicsStartTime>,
    mut debug_diagnostics: ResMut<DebugDiagnostics>,
    mut next_state: ResMut<NextState<SimulationState>>,
) {
    if tectonics_iteration.0 < tectonics_config.iterations {
        // 1. Calculate acceleration for each particle
        let new_particle_accelerations: Vec<Vec3> = plate_particles
            .0
            .par_iter()
            .map(|particle| {
                let plate_force = plates.0[particle.plate_index]
                    .axis_of_rotation
                    .cross(particle.position)
                    * tectonics_config.plate_force_modifier
                    // We make this force mass independent so oceanic and continental plates move equally
                    * particle.mass;
                let friction_force = if particle.velocity.length() > 0. {
                    -particle.velocity * particle.mass * tectonics_config.friction_coefficient
                } else {
                    Vec3::ZERO
                };

                let mut interaction_force = Vec3::ZERO;
                for other_particle in plate_particles
                    .0
                    .get_within(particle.position, tectonics_config.particle_force_radius)
                {
                    if particle.id == other_particle.id {
                        continue;
                    }
                    let geodesic_distance =
                        f32::acos(particle.position.dot(other_particle.position));
                    let repulsive_force = if particle.plate_index == other_particle.plate_index {
                        1. / (geodesic_distance / tectonics_config.repulsive_force_modifier).powi(2)
                    } else {
                        1. / (geodesic_distance / tectonics_config.repulsive_force_modifier).powi(2)
                            * 2.
                    };
                    let attraction_force = if particle.plate_index == other_particle.plate_index {
                        tectonics_config.attractive_force
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
        for (i, particle) in plate_particles.0.iter_mut().enumerate() {
            let displacement = particle.velocity * tectonics_config.timestep
                + 0.5 * particle.acceleration * tectonics_config.timestep.powi(2);
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
                    * tectonics_config.timestep;
            particle.acceleration = new_particle_accelerations[i];
        }
        // 3. Update the sphere bin datastructure as some particles might leave their current bin
        plate_particles.0.refresh();
        // 4. Randomly modify each plates axis of rotation slightly
        for plate in plates.0.iter_mut() {
            plate.drift_direction = (plate.drift_direction
                + Vec2::new(
                    rng.0.random_range(-1.0..1.0) * tectonics_config.plate_rotation_drift_rate,
                    rng.0.random_range(-1.0..1.0) * tectonics_config.plate_rotation_drift_rate,
                ) * tectonics_config.timestep)
                .normalize();
            plate.axis_of_rotation = Quat::from_euler(
                EulerRot::XYZ,
                plate.drift_direction.x * tectonics_config.plate_rotation_drift_rate,
                plate.drift_direction.y * tectonics_config.plate_rotation_drift_rate,
                0.,
            ) * plate.axis_of_rotation;
        }
        tectonics_iteration.0 += 1;
    } else {
        debug_diagnostics.tectonics_time = Some(tectonics_start_time.0.elapsed());
        next_state.set(SimulationState::Erosion);
    }
}

fn draw_bins(
    mut gizmos: Gizmos,
    // bins: Res<PlateParticles>,
    tectonics_config: Res<TectonicsConfiguration>,
    current_mouse_pick: Res<CurrentMousePick>,
) {
    if let Some(MousePickInfo { tile, normal }) = &current_mouse_pick.0 {
        gizmos.circle(
            Isometry3d {
                rotation: Quat::from_rotation_arc(Vec3::Z, *normal),
                translation: (normal * tile.height).into(),
            },
            tectonics_config.particle_force_radius,
            LinearRgba::BLUE,
        );
        // for particle in bins
        //     .0
        //     .get_within(*normal, tectonics_config.particle_force_radius)
        // {
        //     let geodesic_distance = f32::acos(normal.dot(particle.position));
        //     let distance_fraction = geodesic_distance / tectonics_config.particle_force_radius;
        //     gizmos.arrow(
        //         particle.position,
        //         particle.position * (1.1 - distance_fraction / 10.),
        //         LinearRgba::new(distance_fraction, 1.0, 0.0, 1.0),
        //     );
        // }
    }
}
