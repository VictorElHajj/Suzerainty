use crate::hex_sphere::{Tile, vec_utils};
use crate::utils::MainCamera;
use crate::{debug_ui::DebugDiagnostics, states::SimulationState};
use bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    render::mesh::{Indices, PrimitiveTopology},
    window::PrimaryWindow,
};
use std::{num::NonZero, time::Instant};
use subsphere::Vertex;
use subsphere::{Face, Sphere, proj::Fuller};

#[derive(Resource)]
pub struct HexSphere {
    /// The [subsphere::HexSphere<Fuller>] [HexSphere] wraps around
    pub subsphere: subsphere::HexSphere<Fuller>,
    /// The modified vertices with interpolated heights
    pub vertices: Vec<[f32; 3]>,
    /// Essentially a wrapper around [subsphere::hex::Face<Fuller>], modified with a central vertex and height
    pub tiles: Vec<Tile>,
    /// For each vertex, the indices of the tiles it is adjacent to
    pub vertices_to_tiles: Vec<Vec<usize>>,
}

impl HexSphere {
    /// Returns [Tile] from unit sphere normal
    pub fn tile_at(&self, at: Vec3) -> &Tile {
        &self.tiles[self.subsphere.face_at(vec_utils::vec3_to_f64_3(at)).index()]
    }
}

#[derive(Component)]
struct SphereMeshMarker;

#[derive(Resource, Clone, Copy)]
pub struct HexSphereConfig {
    pub subdivisions: u32,
}
pub struct HexSpherePlugin {
    pub config: HexSphereConfig,
}
impl Plugin for HexSpherePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config)
            .insert_resource(CurrentMousePick::default())
            .add_systems(OnEnter(SimulationState::MeshGen), setup)
            .add_systems(Update, (mouse_pick, draw_selected));
    }
}

#[derive(Resource)]
pub struct HexSphereMeshHandle(pub Handle<Mesh>);

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut diagnostics: ResMut<DebugDiagnostics>,
    config: Res<HexSphereConfig>,
    mut next_state: ResMut<NextState<SimulationState>>,
) {
    let start = Instant::now();
    // Create and save a handle to the mesh.
    // 548 is the smallest number above a million tiles.
    let c = config.subdivisions % 3;
    let hex_sphere = subsphere::HexSphere::from_kis(subsphere::TriSphere::new(
        subsphere::BaseTriSphere::Icosa,
        subsphere::proj::Fuller,
        NonZero::new(config.subdivisions).unwrap(),
        c,
    ))
    .unwrap();

    let num_pentagons = 12;
    let num_hexagons = hex_sphere.num_faces() - num_pentagons;
    let num_vertices = num_pentagons * 6 + num_hexagons * 7;
    let num_faces = hex_sphere.num_faces();

    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
    let mut vertices_to_tiles: Vec<Vec<usize>> = vec![Vec::new(); num_vertices];
    let mut tiles: Vec<Tile> = Vec::with_capacity(num_faces);
    let mut triangles: Vec<u32> = Vec::with_capacity(num_hexagons * 6 + num_pentagons + 5);
    let mut colors: Vec<[f32; 4]> = vec![[0.; 4]; num_vertices];
    // let mut normals: Vec<[f32; 3]> = vec![[0.; 3]; num_vertices];

    let mut tile_heights: Vec<f32> = Vec::with_capacity(hex_sphere.num_faces());
    for face in hex_sphere.faces() {
        let vec: Vec3 = face.center().pos().map(|f| f as f32).into();
        tile_heights.push(vec.length());
    }

    // Create tiles and mesh
    for (i, face) in hex_sphere.faces().enumerate() {
        // Build triangles, we want each face to be triangular slices around the center point
        let height_color = 1.0;
        let face_color = [height_color, height_color, height_color, 1.0];
        let face_normal = vec_utils::f64_3_to_f32_3(&face.center().pos());
        let face_center = face_normal.map(|f| f * tile_heights[i]);
        let face_vertex_count = if face.is_hex() { 7 } else { 6 };

        // For each face vertex excluding the center, interpolate between adjacent tile centers
        vertices.extend(face.vertices().map(|v| {
            let interpolated_pos: [f32; 3] = v
                .faces()
                .map(|face| {
                    face.center()
                        .pos()
                        .map(|val| val as f32 * tile_heights[face.index()] / 3.)
                })
                .reduce(|acc, e| [acc[0] + e[0], acc[1] + e[1], acc[2] + e[2]])
                .unwrap();
            interpolated_pos
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

        for index in &face_vertex_indices {
            colors[*index] = face_color;
        }

        let mut adjacent = face
            .vertices()
            // Need explicit collect or we run into a infinite type recursion for some reason
            .flat_map(|v| v.faces().map(|f| f.index()).collect::<Vec<usize>>())
            .collect::<Vec<usize>>();
        adjacent.sort_unstable();
        adjacent.dedup();

        vertices_to_tiles[face_center_index] = vec![];
        for (i, vertex) in face.vertices().enumerate() {
            vertices_to_tiles[face_vertex_indices[i]] =
                vertex.faces().map(|f| f.index()).collect::<Vec<usize>>();
        }

        tiles.push(Tile {
            index: i,
            center: face_center_index,
            vertices: face_vertex_indices[..face_vertex_indices.len() - 1].into(),
            height: tile_heights[i],
            adjacent,
            normal: face_normal.into(),
        });
    }

    commands.insert_resource(HexSphere {
        subsphere: hex_sphere,
        tiles,
        vertices: vertices.clone(),
        vertices_to_tiles,
    });

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_indices(Indices::U32(triangles))
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.compute_normals();
    let mesh_handle = meshes.add(mesh);
    commands.insert_resource(HexSphereMeshHandle(mesh_handle.clone()));

    // Render the mesh with the custom texture, and add the marker.
    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            perceptual_roughness: 0.9,
            reflectance: 0.18,
            ..Default::default()
        })),
        SphereMeshMarker,
    ));

    diagnostics.tiles = Some(num_faces);
    diagnostics.subdivisions = Some(config.subdivisions);
    diagnostics.mesh_gen_time = Some(start.elapsed());
    next_state.set(SimulationState::Tectonics)
}

#[derive(Resource, Default)]
pub struct CurrentMousePick(pub Option<MousePickInfo>);

pub struct MousePickInfo {
    pub normal: Vec3,
    // Todo, make this a reference and have the tile and hexsphere be global?
    pub tile: Tile,
}

/// Picks the tile under the cursor
/// This depends on the fact that the camera is orthographic and always pointing at a unit sphere in origin.
fn mouse_pick(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Projection, &Transform), With<MainCamera>>,
    hex_sphere: Res<HexSphere>,
    mut gizmos: Gizmos,
    mut current_mouse_pick: ResMut<CurrentMousePick>,
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

                let tile = &hex_sphere.tiles[hex_sphere
                    .subsphere
                    .face_at(vec_utils::f32_3_to_f64_3(&point_world.into()))
                    .index()];

                current_mouse_pick.0 = Some(MousePickInfo {
                    normal: point_world,
                    tile: tile.clone(),
                });

                // Draw the selected tile
                tile.draw_border(&hex_sphere.vertices, LinearRgba::WHITE.into(), &mut gizmos);
                // Draw connected tiles
                // for adjacent_tile in tile
                //     .adjacent
                //     .iter()
                //     .map(|adjacent_index| &hex_sphere.tiles[*adjacent_index])
                // {
                //     adjacent_tile.draw_border(
                //         &hex_sphere.vertices,
                //         LinearRgba::GREEN.into(),
                //         &mut gizmos,
                //     );
                // }
            } else {
                current_mouse_pick.0 = None;
            }
        }
    }
}

fn draw_selected(
    mut gizmos: Gizmos,
    hex_sphere: Res<HexSphere>,
    current_mouse_pick: Res<CurrentMousePick>,
) {
    if let Some(MousePickInfo { tile, .. }) = &current_mouse_pick.0 {
        tile.draw_border(&hex_sphere.vertices, LinearRgba::WHITE.into(), &mut gizmos);
    }
}
