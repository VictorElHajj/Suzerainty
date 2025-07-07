#![feature(slice_as_array)]

use crate::{
    debug_ui::{DebugDiagnostics, DebugUIPlugin},
    hex_sphere::{HexSphereConfig, HexSpherePlugin},
    states::SimulationState,
    tectonics::{TectonicsPlugin, TectonicsPluginConfig},
};
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*, render::camera::ScalingMode};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use rand::SeedableRng;
use suz_sim::{particle_sphere::ParticleSphereConfig, tectonics::TectonicsConfiguration};

mod debug_ui;
mod hex_sphere;
mod states;
mod tectonics;
mod vertex_interpolation;

fn main() {
    let seed = rand::random::<u64>();
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Suzerainty".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            PanOrbitCameraPlugin,
            FrameTimeDiagnosticsPlugin {
                max_history_length: 60,
                smoothing_factor: 0.1,
            },
            DebugUIPlugin {
                diagnostics: DebugDiagnostics::seed(seed),
            },
            HexSpherePlugin {
                config: HexSphereConfig { subdivisions: 128 },
            },
            TectonicsPlugin {
                config: TectonicsPluginConfig {
                    tectonics_config: TectonicsConfiguration {
                        major_plate_fraction: 0.3,
                        major_tile_fraction: 0.4,
                        plate_goal: 20,
                        continental_rate: 0.4,
                        min_plate_size: 15,
                        particle_force_radius: 0.20,
                        repulsive_force_modifier: 0.06,
                        link_spring_constant: 10.0,
                        plate_force_modifier: 0.02,
                        plate_rotation_drift_rate: 0.01,
                        timestep: 0.1,
                        iterations: 1000,
                        friction_coefficient: 0.8,
                    },
                    particle_config: ParticleSphereConfig { subdivisions: 32 },
                },
            },
        ))
        .add_systems(Startup, setup)
        .insert_resource(ClearColor(LinearRgba::BLACK.into()))
        .insert_resource(GlobalRng(rand::rngs::StdRng::seed_from_u64(seed)))
        .init_state::<SimulationState>()
        .run();
}

#[derive(Resource)]
pub struct GlobalRng(pub rand::rngs::StdRng);

#[derive(Component)]
pub struct MainCamera;

fn setup(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            // 6 world units per pixel of window height.
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 1.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            focus: Transform::IDENTITY.translation,
            radius: Some(10.),
            zoom_lower_limit: 0.01,
            zoom_upper_limit: Some(10.),
            allow_upside_down: false,
            pan_sensitivity: 0.,
            ..Default::default()
        },
    ));
}
