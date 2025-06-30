use std::f32::consts::PI;
use suz_sim::{
    particle_sphere::{ParticleSphere, ParticleSphereConfig},
    tectonics::{Tectonics, TectonicsConfiguration},
};

use bevy::prelude::*;

use crate::{
    GlobalRng,
    debug_ui::DebugDiagnostics,
    hex_sphere::{CurrentMousePick, MousePickInfo},
    states::SimulationState,
    vertex_interpolation::interpolate_vertices,
};

#[derive(Resource)]
pub struct TectonicsIteration(pub usize);

#[derive(Resource, Clone, Copy)]
pub struct TectonicsPluginConfig {
    pub tectonics_config: TectonicsConfiguration,
    pub particle_config: ParticleSphereConfig,
}

pub struct TectonicsPlugin {
    pub config: TectonicsPluginConfig,
}
impl Plugin for TectonicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config)
            .insert_resource(TectonicsIteration(0))
            .add_systems(OnEnter(SimulationState::Tectonics), setup)
            .add_systems(
                Update,
                (
                    draw_particles,
                    draw_bins,
                    simulate_system.run_if(in_state(SimulationState::Tectonics)),
                    interpolate_vertices.run_if(in_state(SimulationState::Tectonics)),
                ),
            );
    }
}

#[derive(Resource)]
struct TectonicsStartTime(std::time::Instant);

fn setup(config: Res<TectonicsPluginConfig>, mut commands: Commands, mut rng: ResMut<GlobalRng>) {
    let particle_sphere = ParticleSphere::from_config(config.particle_config);
    let tectonics = Tectonics::from_config(config.tectonics_config, &particle_sphere, &mut rng.0);
    commands.insert_resource(TectonicsStartTime(std::time::Instant::now()));
    commands.insert_resource(particle_sphere);
    commands.insert_resource(tectonics);
}

fn draw_particles(
    mut gizmos: Gizmos,
    tectonics: Res<Tectonics>,
    particle_sphere: Res<ParticleSphere>,
) {
    for plate in &tectonics.plates {
        gizmos.arrow(
            plate.axis_of_rotation,
            plate.axis_of_rotation * 1.1,
            plate.color,
        );
    }
    for particle in tectonics.particles.iter() {
        gizmos.cross(
            Isometry3d {
                translation: (particle.position * particle.height).into(),
                rotation: Quat::from_rotation_arc(Vec3::Z, particle.position),
            },
            16. * PI / particle_sphere.tiles.len() as f32,
            tectonics.plates[particle.plate_index].color,
        );
    }
}

fn simulate_system(
    tectonics_start_time: Res<TectonicsStartTime>,
    mut tectonics: ResMut<Tectonics>,
    mut rng: ResMut<GlobalRng>,
    mut tectonics_iteration: ResMut<TectonicsIteration>,
    mut debug_diagnostics: ResMut<DebugDiagnostics>,
    mut next_state: ResMut<NextState<SimulationState>>,
) {
    if tectonics_iteration.0 < tectonics.config.iterations {
        tectonics.simulate(&mut rng.0);
        tectonics_iteration.0 += 1;
    } else {
        debug_diagnostics.tectonics_time = Some(tectonics_start_time.0.elapsed());
        next_state.set(SimulationState::Erosion);
    }
}

fn draw_bins(
    mut gizmos: Gizmos,
    // bins: Res<PlateParticles>,
    tectonics_config: Res<TectonicsPluginConfig>,
    current_mouse_pick: Res<CurrentMousePick>,
) {
    if let Some(MousePickInfo { tile, normal }) = &current_mouse_pick.0 {
        gizmos.circle(
            Isometry3d {
                rotation: Quat::from_rotation_arc(Vec3::Z, *normal),
                translation: (normal * tile.height).into(),
            },
            tectonics_config.tectonics_config.particle_force_radius,
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
