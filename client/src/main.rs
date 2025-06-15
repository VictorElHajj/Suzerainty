#![feature(slice_as_array)]
use std::num::NonZero;

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
use noise::NoiseFn;
use rand::Rng;
use subsphere::{Face, Sphere, Vertex, proj::Fuller};

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
        .insert_resource(ClearColor(LinearRgba::BLACK.into()))
        .run();
}

#[derive(Component)]
struct MainCamera;

#[derive(Resource)]
pub struct HexSphere(subsphere::HexSphere<Fuller>);

#[derive(Component)]
struct SphereMeshMarker;

#[inline]
fn f64_3_to_f32_3(input: &[f64; 3]) -> [f32; 3] {
    input.map(|p| p as f32)
}
#[inline]
fn f64_3_to_vec3(input: &[f64; 3]) -> Vec3 {
    input.map(|p| p as f32).into()
}
#[inline]
fn f32_3_to_f64_3(input: &[f32; 3]) -> [f64; 3] {
    input.map(|p| p as f64)
}
#[inline]
fn vec3_to_f64_3(input: Vec3) -> [f64; 3] {
    let arr: [f32; 3] = input.into();
    arr.map(|p| p as f64)
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Create and save a handle to the mesh.
    // around 548 or something to get 1 million faces
    let subdivisions = 128;
    let c = subdivisions % 3;
    let sphere = subsphere::HexSphere::from_kis(subsphere::TriSphere::new(
        subsphere::BaseTriSphere::Icosa,
        subsphere::proj::Fuller,
        NonZero::new(subdivisions).unwrap(),
        c,
    ))
    .unwrap();
    commands.insert_resource(HexSphere(sphere));
    let num_pentagons = 12;
    let num_hexagons = sphere.num_faces() - num_pentagons;
    let num_vertices = num_pentagons * 6 + num_hexagons * 7;

    let num_faces = sphere.num_faces();
    println!("Faces: {:?}", sphere.num_faces());

    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
    let mut triangles: Vec<u32> = Vec::with_capacity(num_hexagons * 6 + num_pentagons + 5);
    let mut colors: Vec<[f32; 4]> = vec![[0.; 4]; num_vertices];
    // let mut normals: Vec<[f32; 3]> = vec![[0.; 3]; num_vertices];

    // Temp just to see how it looks like
    let mut tile_heights: Vec<f32> = Vec::with_capacity(sphere.num_faces());
    let noise = noise::Simplex::new(2012);
    for face in sphere.faces() {
        // range [0, 1]
        let normalized_nosie = noise.get(face.center().pos().map(|f| f * 8.)) as f32 + 1. / 2.;
        tile_heights.push(1. + normalized_nosie / 10.);
    }

    let mut rng = rand::rng();
    for (i, face) in sphere.faces().enumerate() {
        // Build triangles, we want each face to be triangular slices around the center point
        let height_color = (tile_heights[i] - 1.) * 10.;
        let face_color = [height_color, height_color, height_color, 1.0];
        let face_normal = f64_3_to_f32_3(&face.center().pos());
        let face_center = face_normal.map(|f| f * tile_heights[i]);
        let face_vertex_count = if face.is_hex() { 7 } else { 6 };

        // For each face vertex excluding the center, interpolate between adjacent tiles
        vertices.extend(face.vertices().map(|v| {
            let interpolated_height: f32 = v.faces().map(|f| tile_heights[f.index()] / 3.).sum();
            // TODO, distance between faces might not be the same, should I use something that adjusts for that, like bilinear interpolation? (Trilinear?)
            f64_3_to_f32_3(&v.pos()).map(|f| f * interpolated_height)
        }));
        vertices.push(face_center);
        let face_center_index: usize = vertices.len() - 1;

        let face_vertex_indices: Vec<usize> =
            (face_center_index + 1 - face_vertex_count..=face_center_index).collect();

        let mut face_triangles: Vec<u32> = face_vertex_indices[..face_vertex_indices.len() - 1]
            .iter()
            .flat_map(move |i| vec![*i as u32, face_center_index as u32, *i as u32])
            .collect();
        face_triangles.rotate_right(1);
        triangles.extend(face_triangles);

        for index in face_vertex_indices {
            colors[index] = face_color;
            // normals[index] = face_normal;
        }
    }

    println!("Face processing done.");

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_indices(Indices::U32(triangles))
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.compute_normals();
    let mesh_handle = meshes.add(mesh);

    // Render the mesh with the custom texture, and add the marker.
    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            ..Default::default()
        })),
        SphereMeshMarker,
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

/// Picks the tile under the cursor
/// This depends on the fact that the camera is orthographic and always pointing at a unit sphere in origin.
fn draw_picking(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Projection, &Transform), With<MainCamera>>,
    hex_sphere: Res<HexSphere>,
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

                let face = hex_sphere.0.face_at(f32_3_to_f64_3(&point_world.into()));

                // Draw the selected tile
                gizmos.linestrip(
                    face.vertices()
                        .chain(std::iter::once(face.vertices().next().unwrap()))
                        .map(|v| f64_3_to_vec3(&v.pos()) * 1.01),
                    Color::LinearRgba(LinearRgba::WHITE),
                );
                // Draw connected tiles
                // for index in cursor_tile.adjecencies() {
                //     let adjacent_tile = &sphere_info.tiles[*index];
                //     gizmos.linestrip(
                //         adjacent_tile
                //             .vertices()
                //             .chain(std::iter::once(adjacent_tile.vertices().next().unwrap()))
                //             .map(|i| {
                //                 let v: Vec3 = sphere_info.vertices[*i].into();
                //                 v * 1.001
                //             }),
                //         Color::LinearRgba(LinearRgba::GREEN),
                //     );
                // }
            }
        }
    }
}
