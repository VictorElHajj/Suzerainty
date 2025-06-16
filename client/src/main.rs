#![feature(slice_as_array)]

pub mod hex_sphere;

use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::camera::ScalingMode,
    text::FontSmoothing,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::hex_sphere::{HexSphereConfig, HexSpherePlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            WireframePlugin::default(),
            PanOrbitCameraPlugin,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: 20.0,
                        font: default(),
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: Color::WHITE,
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                },
            },
            HexSpherePlugin {
                config: HexSphereConfig { subdivisions: 548 },
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
