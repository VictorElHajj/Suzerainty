use std::num::NonZero;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    window::PrimaryWindow,
};
use noise::NoiseFn;
use subsphere::{Face, Sphere, Vertex, proj::Fuller};

use crate::{
    MainCamera,
    hex_sphere::{Tile, vec_utils},
};

#[derive(Resource)]
pub struct HexSphere {
    /// The [subsphere::HexSphere<Fuller>] [HexSphere] wraps around
    pub subsphere: subsphere::HexSphere<Fuller>,
    /// The modified vertices with interpolated heights
    pub vertices: Vec<[f32; 3]>,
    /// Essentially a wrapper around [subsphere::hex::Face<Fuller>], modified with a central vertex and height
    pub tiles: Vec<Tile>,
}

impl HexSphere {
    /// Returns [Tile] from unit sphere normal
    pub fn tile_at(&self, at: Vec3) -> &Tile {
        &self.tiles[self.subsphere.face_at(vec_utils::vec3_to_f64_3(at)).index()]
    }
}

#[derive(Component)]
struct SphereMeshMarker;

pub struct HexSpherePlugin;
impl Plugin for HexSpherePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, draw_picking);
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Create and save a handle to the mesh.
    // 548 is the smallest number above a million tiles.
    let subdivisions = 548;
    let c = subdivisions % 3;
    let hex_sphere = subsphere::HexSphere::from_kis(subsphere::TriSphere::new(
        subsphere::BaseTriSphere::Icosa,
        subsphere::proj::Fuller,
        NonZero::new(subdivisions).unwrap(),
        c,
    ))
    .unwrap();

    let num_pentagons = 12;
    let num_hexagons = hex_sphere.num_faces() - num_pentagons;
    let num_vertices = num_pentagons * 6 + num_hexagons * 7;
    let num_faces = hex_sphere.num_faces();
    println!("Faces: {:?}", num_faces);

    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
    let mut tiles: Vec<Tile> = Vec::with_capacity(num_faces);
    let mut triangles: Vec<u32> = Vec::with_capacity(num_hexagons * 6 + num_pentagons + 5);
    let mut colors: Vec<[f32; 4]> = vec![[0.; 4]; num_vertices];
    // let mut normals: Vec<[f32; 3]> = vec![[0.; 3]; num_vertices];

    // Temp height generation just to see how it looks like with 3d noise
    let mut tile_heights: Vec<f32> = Vec::with_capacity(hex_sphere.num_faces());
    let noise = noise::Simplex::new(2012);
    for face in hex_sphere.faces() {
        // range [-1, 1]
        let noise_1 = 1. / 3. * noise.get(face.center().pos().map(|f| f * 48.));
        let noise_2 = 2. / 3. * noise.get(face.center().pos().map(|f| f * 8.));
        let noise = noise_1 + noise_2;
        // range [0.99, 1.01]
        let adjusted_noise = 1. + noise as f32 / 100.;
        tile_heights.push(adjusted_noise);
    }

    // Create tiles and mesh
    for (i, face) in hex_sphere.faces().enumerate() {
        // Build triangles, we want each face to be triangular slices around the center point
        let height_color = (tile_heights[i] - 0.99) * 100.;
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
            // normals[index] = face_normal;
        }

        let mut adjacent = face
            .vertices()
            // Need explicit collect or we run into a infinite type recursion for some reason
            .flat_map(|v| v.faces().map(|f| f.index()).collect::<Vec<usize>>())
            .collect::<Vec<usize>>();
        adjacent.sort_unstable();
        adjacent.dedup();

        tiles.push(Tile {
            index: i,
            center: face_center_index,
            vertices: face_vertex_indices[..face_vertex_indices.len() - 1].into(),
            height: tile_heights[i],
            adjacent,
        });
    }

    commands.insert_resource(HexSphere {
        subsphere: hex_sphere,
        tiles,
        vertices: vertices.clone(),
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

    println!("Mesh processing done.");

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

                let tile = &hex_sphere.tiles[hex_sphere
                    .subsphere
                    .face_at(vec_utils::f32_3_to_f64_3(&point_world.into()))
                    .index()];

                // Draw the selected tile
                tile.draw_border(&hex_sphere.vertices, LinearRgba::WHITE.into(), &mut gizmos);
                // Draw connected tiles
                for adjacent_tile in tile
                    .adjacent
                    .iter()
                    .map(|adjacent_index| &hex_sphere.tiles[*adjacent_index])
                {
                    adjacent_tile.draw_border(
                        &hex_sphere.vertices,
                        LinearRgba::GREEN.into(),
                        &mut gizmos,
                    );
                }
            }
        }
    }
}
