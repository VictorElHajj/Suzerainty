use std::f32::consts::PI;

use bevy::{math::NormedVectorSpace, platform::collections::HashSet, prelude::*};
use rand::Rng;

use crate::{
    GlobalRng,
    hex_sphere::{CurrentMousePick, HexSphere, MousePickInfo},
    sphere_bins::{GetNormal, SphereBins},
    states::SimulationState,
    tectonics::{
        particle::PlateParticle,
        plate::{Plate, PlateType},
    },
};

const OCEANIC_PARTICLE_MASS: f32 = 2.;
const CONTINENTAL_PARTICLE_MASS: f32 = 3.;

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
    pub attractive_force_modifier: f32,
    /// Modifier to the force applies by the plate rotational axis to plate particles.
    pub plate_force_modifier: f32,
}

pub struct TectonicsPlugin {
    pub config: TectonicsConfiguration,
}
impl Plugin for TectonicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config)
            .add_systems(OnEnter(SimulationState::Tectonics), setup)
            .add_systems(
                Update,
                (draw_particles, draw_bins, update_particle_velocities),
            );
    }
}

#[derive(Resource)]
struct PlateParticles(SphereBins<100, PlateParticle>);

#[derive(Resource)]
struct Plates(Vec<Plate>);

fn setup(
    mut commands: Commands,
    hex_sphere: Res<HexSphere>,
    tectonics_config: Res<TectonicsConfiguration>,
    mut rng: ResMut<GlobalRng>,
) {
    assert!((0.0..=1.0).contains(&tectonics_config.major_tile_fraction));
    assert!((0.0..=1.0).contains(&tectonics_config.major_plate_fraction));
    assert!((0.0..=1.0).contains(&tectonics_config.continental_rate));
    let mut generated_majors = 0;
    let mut generated_minors = 0;
    let mut plates = Vec::<Plate>::new();
    let mut particle_bins = SphereBins::<100, PlateParticle>::new();

    // There is a bit of heuristics here, like the magic / 2. This is not a perfect technique.
    let tile_count = hex_sphere.tiles.len();
    let major_tile_count: usize = (tile_count as f32 * tectonics_config.major_tile_fraction
        / (tectonics_config.plate_goal as f32 / 2.)
        / tectonics_config.major_plate_fraction) as usize;
    let minor_tile_count: usize = (tile_count as f32 * (1. - tectonics_config.major_tile_fraction)
        / (tectonics_config.plate_goal as f32 / 2.)
        / (1. - tectonics_config.major_plate_fraction)) as usize;

    // Pick a random tile to seed the selection, will always be continent 0, which is also always continental and not oceanic
    let starting_tile = rng.0.random_range(0..hex_sphere.tiles.len());
    let mut global_surrounding_unvisited_tiles = Vec::<usize>::new();
    let mut next_surrounding_unvisited_tiles = vec![starting_tile];
    let mut added_tiles = HashSet::<usize>::new();
    added_tiles.insert(starting_tile);

    while added_tiles.len() < hex_sphere.tiles.len() {
        let plate_color = LinearRgba::rgb(rng.0.random(), rng.0.random(), rng.0.random()).into();
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
            let chosen_tile = &hex_sphere.tiles
                [next_surrounding_unvisited_tiles.swap_remove(random_adjacent_tile_index)];

            // Create particle from chosen tile
            temp_particle_vec.push(PlateParticle {
                position: chosen_tile.normal,
                height: 1.0,
                plate_index: plates.len(),
                mass: if plate_type == PlateType::Continental {
                    CONTINENTAL_PARTICLE_MASS
                } else {
                    OCEANIC_PARTICLE_MASS
                },
                velocity: plate.axis_of_rotation.cross(chosen_tile.normal) * 0.005,
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
                // TODO: Height and velocity should use the new plates initial values rather than keeping the old
                particle_bins.insert(PlateParticle {
                    position: particle.position,
                    height: particle.height,
                    plate_index: closest_plate_index,
                    mass: if plates[closest_plate_index].plate_type == PlateType::Continental {
                        CONTINENTAL_PARTICLE_MASS
                    } else {
                        OCEANIC_PARTICLE_MASS
                    },
                    velocity: plates[closest_plate_index]
                        .axis_of_rotation
                        .cross(particle.position),
                });
            }
        }

        // Add remaining unvisited to global unvisited, update to remove used ones.
        global_surrounding_unvisited_tiles.extend(&next_surrounding_unvisited_tiles);
        global_surrounding_unvisited_tiles.retain(|index| !added_tiles.contains(index));
        // Pick a new starting point for the global unvisited, if there are tiles left
        if !(added_tiles.len() == hex_sphere.tiles.len()) {
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
    hex_sphere: Res<HexSphere>,
) {
    for plate in &plates.0 {
        gizmos.arrow(
            plate.axis_of_rotation,
            plate.axis_of_rotation * 1.5,
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
                translation: particle.position.into(),
                rotation: Quat::from_rotation_arc(Vec3::Z, particle.position),
            },
            16. * PI / hex_sphere.tiles.len() as f32,
            plates.0[particle.plate_index].color,
        );
    }
}

// Each particle will be forced to have the velocity matching rotation around the ownings plate axis of rotation
// Then we adjust that velocity depending on other particles
fn update_particle_velocities(
    mut plate_particles: ResMut<PlateParticles>,
    plates: Res<Plates>,
    hex_sphere: Res<HexSphere>,
    tectonics_config: Res<TectonicsConfiguration>,
    mut gizmos: Gizmos,
) {
    let new_velocities: Vec<Vec3> = plate_particles
        .0
        .iter()
        .map(|particle| {
            // TODO: Multiply by velocity
            let plate_velocity = plates.0[particle.plate_index]
                .axis_of_rotation
                .cross(particle.position);
            let mut acceleration = Vec3::ZERO;
            for other_particle in plate_particles
                .0
                .get_within(particle.position, tectonics_config.particle_force_radius)
            {
                if particle == other_particle {
                    continue;
                }
                let geodesic_distance = f32::acos(particle.position.dot(other_particle.position));
                let repulsive_force = if particle.plate_index == other_particle.plate_index {
                    1. / (geodesic_distance / tectonics_config.repulsive_force_modifier).powi(2)
                } else {
                    1. / (geodesic_distance / tectonics_config.repulsive_force_modifier).powi(2)
                        * 4.
                };
                let attraction_force = if particle.plate_index == other_particle.plate_index {
                    0.
                } else {
                    0.
                };
                acceleration += (repulsive_force - attraction_force)
                    * (particle.position - other_particle.position)
                    / particle.mass;
                if particle.plate_index != other_particle.plate_index {
                    // TODO this is where we do plate interactions
                }
            }
            // Assumes timestep = 1s
            gizmos.arrow(
                particle.position,
                particle.position
                    + acceleration
                    + plate_velocity * tectonics_config.plate_force_modifier,
                plates.0[particle.plate_index].color,
            );
            plate_velocity * tectonics_config.plate_force_modifier + acceleration
        })
        .collect();
    for (i, particle) in plate_particles.0.iter_mut().enumerate() {
        particle.velocity = new_velocities[i];
        particle.position = (particle.position + particle.velocity).normalize();
    }
    plate_particles.0.refresh();
}

fn draw_bins(
    mut gizmos: Gizmos,
    bins: Res<PlateParticles>,
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
