#![feature(slice_as_array)]

mod debug_ui;
mod hex_sphere;
mod sphere_bins;
mod states;
mod tectonics;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::camera::ScalingMode,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use rand::SeedableRng;

use crate::{
    debug_ui::{DebugDiagnostics, DebugUIPlugin},
    hex_sphere::{HexSphereConfig, HexSpherePlugin},
    states::SimulationState,
    tectonics::{TectonicsConfiguration, TectonicsPlugin},
};

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
            WireframePlugin::default(),
            PanOrbitCameraPlugin,
            FrameTimeDiagnosticsPlugin {
                max_history_length: 60,
                smoothing_factor: 0.1,
            },
            DebugUIPlugin {
                diagnostics: DebugDiagnostics::seed(seed),
            },
            HexSpherePlugin {
                config: HexSphereConfig { subdivisions: 32 },
            },
            TectonicsPlugin {
                config: TectonicsConfiguration {
                    major_plate_fraction: 0.5,
                    major_tile_fraction: 0.75,
                    plate_goal: 10,
                    continental_rate: 0.3,
                    min_plate_size: 15,
                    particle_force_radius: 0.20,
                    repulsive_force_modifier: 0.01,
                    attractive_force_modifier: 10.,
                    plate_force_modifier: 0.01,
                },
            },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_wireframe)
        .insert_resource(ClearColor(LinearRgba::BLACK.into()))
        .insert_resource(GlobalRng(rand::rngs::StdRng::seed_from_u64(seed)))
        .init_state::<SimulationState>()
        .run();
}

#[derive(Resource)]
struct GlobalRng(rand::rngs::StdRng);

#[derive(Component)]
struct MainCamera;

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

fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
    }
}
