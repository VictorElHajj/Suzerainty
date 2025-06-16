#![feature(slice_as_array)]

mod debug_ui;
mod hex_sphere;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::camera::ScalingMode,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::{
    debug_ui::DebugUIPlugin,
    hex_sphere::{HexSphereConfig, HexSpherePlugin},
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            WireframePlugin::default(),
            PanOrbitCameraPlugin,
            FrameTimeDiagnosticsPlugin::default(),
            DebugUIPlugin,
            HexSpherePlugin {
                config: HexSphereConfig { subdivisions: 8 },
            },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_wireframe)
        .insert_resource(ClearColor(LinearRgba::BLACK.into()))
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
