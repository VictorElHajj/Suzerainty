use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        camera::ScalingMode, mesh::Indices, render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
    text::FontSmoothing,
    window::PrimaryWindow,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use hexglobe::projection::globe::ExactGlobe;

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
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle_wireframe, draw_picking))
        .run();
}

#[derive(Component)]
struct MainCamera;

enum Tile {
    Pentagon { adjecencies: [usize; 5] },
    Hexagon { adjecencies: [usize; 6] },
}

impl Tile {
    pub fn adjecencies(&self) -> impl Iterator<Item = &usize> {
        match &self {
            Tile::Pentagon { adjecencies } => adjecencies.iter(),
            Tile::Hexagon { adjecencies } => adjecencies.iter(),
        }
    }
}

#[derive(Resource)]
struct SphereInfo {
    tiles: Vec<Tile>,
    /// List of (normal, tile_index) tuples sorted by normals in (x then y then z) order
    normals: Vec<(Vec3, usize)>,
}

impl SphereInfo {
    pub fn find_closest_normal(&self, normal: Vec3) -> &Vec3 {
        &self
            .normals
            .iter()
            .max_by(|a, b| normal.dot(a.0).partial_cmp(&normal.dot(b.0)).unwrap())
            .unwrap()
            .0
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Create and save a handle to the mesh.
    pub const SUBDIVISIONS: u32 = 10;
    pub const TILES: usize = ExactGlobe::<SUBDIVISIONS>::FACES;
    let globe = ExactGlobe::<SUBDIVISIONS>::new();
    let centroids = globe.centroids(None);
    let vertices = globe.mesh_vertices(&centroids);
    let faces = globe.mesh_faces();
    let triangles = globe.mesh_triangles(&faces);
    let normals = globe.mesh_normals(&vertices);

    println!("Face amount: {:?}", globe.count_faces());

    // All the vertices in each face share the same normal, so just take the first

    let mut face_normals: Vec<(Vec3, usize)> = Vec::with_capacity(TILES);
    let mut tiles: Vec<Tile> = Vec::with_capacity(TILES);
    for i in 0..TILES {
        match faces[i] {
            hexglobe::projection::globe::MeshFace::Pentagon(vertex_indices) => {
                face_normals.push((normals[vertex_indices[0] as usize].into(), i));
                tiles.push(Tile::Pentagon {
                    adjecencies: [0; 5],
                });
            }
            hexglobe::projection::globe::MeshFace::Hexagon(vertex_indices) => {
                face_normals.push((normals[vertex_indices[0] as usize].into(), i));
                tiles.push(Tile::Hexagon {
                    adjecencies: [0; 6],
                });
            }
        }
    }

    commands.insert_resource(SphereInfo {
        tiles,
        normals: face_normals,
    });

    let mesh_handle = meshes.add(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_indices(Indices::U32(triangles))
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals),
    );

    // Render the mesh with the custom texture, and add the marker.
    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..Default::default()
        })),
    ));

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
            zoom_lower_limit: 0.5,
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

/// Picks the tile under the cursor
/// This depends on the fact that the camera is orthographic and always pointing at a unit sphere in origin.
fn draw_picking(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Projection, &Transform), With<MainCamera>>,
    sphere_info: Res<SphereInfo>,
    mut gizmos: Gizmos,
) {
    let window = window_query.single().unwrap();
    let aspect_ratio = window.size().x / window.size().y;
    let (camera_projection, camera_translation) = camera_query.single().unwrap();
    if let Some(cursor_pos) = window.cursor_position() {
        if let Projection::Orthographic(orthographic_projection) = camera_projection {
            // [-1, 1] in x and y relative to screen
            let ndc = cursor_pos / window.size() * 2.0 - Vec2::ONE;

            // Adjust for scale and aspect ratio, so that [-1, 1] is the position on the 2d unit circle
            let mouse_pos_circle =
                ndc * orthographic_projection.scale * vec2(aspect_ratio, 1.) / 2.;

            // If inside the circle
            if mouse_pos_circle.length_squared() <= 1.0 {
                // Reconstruct Z from the unit sphere constraint: x² + y² + z² = 1
                let point_camera = Vec3::new(
                    mouse_pos_circle.x,
                    -mouse_pos_circle.y,
                    (1.0 - mouse_pos_circle.x * mouse_pos_circle.x
                        - mouse_pos_circle.y * mouse_pos_circle.y)
                        .sqrt(),
                );

                // Adjust for camera rotation
                let rotation = -camera_translation.rotation;
                let mut point_transform = Transform::from_translation(point_camera);
                point_transform.rotate_around(Vec3::ZERO, rotation);
                let point_world = point_transform.translation;

                let normal = point_world.normalize();

                // Binary search for the closest normal to find the tile/face under the cursor
                let cursor_face_normal = sphere_info.find_closest_normal(normal);

                // Draw the normal as an arrow from the surface point
                gizmos.arrow(
                    Vec3::ZERO,
                    cursor_face_normal * 1.2,
                    Color::LinearRgba(LinearRgba::RED),
                );
            }
        }
    }
}
