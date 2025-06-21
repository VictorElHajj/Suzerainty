#![feature(slice_as_array)]

mod debug_ui;
mod hex_sphere;
mod states;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::camera::ScalingMode,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::{
    debug_ui::{DebugDiagnostics, DebugUIPlugin},
    hex_sphere::{HexSphereConfig, HexSpherePlugin},
    states::SimulationState,
};

fn main() {
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
                diagnostics: DebugDiagnostics::seed(rand::random::<u32>()),
            },
            HexSpherePlugin {
                config: HexSphereConfig { subdivisions: 512 },
            },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_wireframe)
        .insert_resource(ClearColor(LinearRgba::BLACK.into()))
        .init_state::<SimulationState>()
        .run();
}

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
