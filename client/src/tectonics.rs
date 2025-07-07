use std::f32::consts::PI;
use suz_sim::{
    particle_sphere::{ParticleSphere, ParticleSphereConfig},
    tectonics::{Tectonics, TectonicsConfiguration},
};

use bevy::prelude::*;

use crate::{
    GlobalRng, debug_ui::DebugDiagnostics, states::SimulationState,
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
            .add_systems(OnExit(SimulationState::Tectonics), interpolate_vertices)
            .add_systems(
                Update,
                (
                    draw_particles,
                    simulate_system.run_if(in_state(SimulationState::Tectonics)),
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
    commands.insert_resource(tectonics);
    commands.insert_resource(particle_sphere);
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
    for (i, particle) in tectonics.particles.iter().enumerate() {
        gizmos.cross(
            Isometry3d {
                translation: (particle.position * particle.height * 1.05).into(),
                rotation: Quat::from_rotation_arc(Vec3::Z, particle.position),
            },
            16. * PI / particle_sphere.tiles.len() as f32,
            tectonics.plates[particle.plate_index].color.with_alpha(0.4),
        );
        let plate_color = tectonics.plates[particle.plate_index].color;
        for other_particle in tectonics.links[&i]
            .iter()
            .map(|o| &tectonics.particles.items[*o])
        {
            gizmos.line(
                particle.position * 1.05,
                other_particle.position * 1.05,
                plate_color.with_alpha(0.1),
            );
        }
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
