#![feature(slice_as_array)]
use std::f32::consts::PI;

use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        camera::ScalingMode,
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
    text::FontSmoothing,
    window::PrimaryWindow,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use hexglobe::projection::globe::ExactGlobe;
use rand::Rng;
use rustc_hash::FxHashMap;

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

// TODO, break out adjecencies itno enum to keep shared stuff in Tile struct?
// Also keep a list of vertices by index
pub enum Tile {
    Pentagon {
        /// Face indices
        adjecencies: [usize; 5],
        /// Vertice indices
        vertices: [usize; 5],
        normal: Vec3,
    },
    Hexagon {
        /// Face indices
        adjecencies: [usize; 6],
        /// Vertice indices
        vertices: [usize; 6],
        normal: Vec3,
    },
}

impl Tile {
    pub fn adjecencies(&self) -> impl Iterator<Item = &usize> {
        match &self {
            Tile::Pentagon { adjecencies, .. } => adjecencies.iter(),
            Tile::Hexagon { adjecencies, .. } => adjecencies.iter(),
        }
    }
    pub fn normal(&self) -> &Vec3 {
        match &self {
            Tile::Pentagon { normal, .. } => normal,
            Tile::Hexagon { normal, .. } => normal,
        }
    }

    pub fn vertices(&self) -> impl Iterator<Item = &usize> {
        match &self {
            Tile::Pentagon { vertices, .. } => vertices.iter(),
            Tile::Hexagon { vertices, .. } => vertices.iter(),
        }
    }
}

#[derive(Resource)]
pub struct SphereInfo<const BIN_COUNT: usize> {
    pub tiles: Vec<Tile>,
    /// Divides sphere into BIN_COUNT bins of aprox equal distance for faster search
    normal_map: FxHashMap<usize, Vec<IndexedNormal>>,
    fibonacci_sphere_points: [IndexedNormal; BIN_COUNT],
}

#[derive(Clone, Copy)]
pub struct IndexedNormal {
    index: usize,
    normal: Vec3,
}

impl<const BIN_COUNT: usize> SphereInfo<BIN_COUNT> {
    fn find_closest_normal_internal<'a>(
        normals: impl Iterator<Item = &'a IndexedNormal>,
        normal: Vec3,
    ) -> &'a IndexedNormal {
        &normals
            .max_by(|a, b| {
                normal
                    .dot(a.normal)
                    .partial_cmp(&normal.dot(b.normal))
                    .unwrap()
            })
            .unwrap()
    }

    fn create_fibonacci_points() -> [IndexedNormal; BIN_COUNT] {
        let golden_angle = PI * (3. - f32::sqrt(5.));
        let offset = 2. / BIN_COUNT as f32;
        let mut points = [IndexedNormal {
            index: 0,
            normal: Vec3::ZERO,
        }; BIN_COUNT];
        for i in 0..BIN_COUNT {
            let y = i as f32 * offset - 1. + offset / 2.;
            let r = (1. - y * y).sqrt();
            let phi = i as f32 * golden_angle;
            let x = f32::cos(phi) * r;
            let z = f32::sin(phi) * r;
            points[i] = IndexedNormal {
                index: i,
                normal: Vec3::new(x, y, z),
            };
        }
        points
    }

    pub fn new(tile_count: usize) -> Self {
        SphereInfo::<BIN_COUNT> {
            tiles: Vec::with_capacity(tile_count),
            normal_map: default(),
            fibonacci_sphere_points: Self::create_fibonacci_points(),
        }
    }

    pub fn add_tile<const SIDES: usize>(
        &mut self,
        tile_index: usize,
        normal: Vec3,
        adjacencies: &FxHashMap<usize, Vec<usize>>,
        vertex_indices: &[u32; SIDES],
    ) {
        let tile_adjacencies = adjacencies.get(&tile_index).unwrap();
        assert!(
            tile_adjacencies.len() == SIDES,
            "Face has {} adjacencies but expected {}",
            tile_adjacencies.len(),
            SIDES
        );
        match SIDES {
            5 => self.tiles.push(Tile::Pentagon {
                adjecencies: *tile_adjacencies.as_array().unwrap(),
                vertices: *vertex_indices.map(|i| i as usize).as_array().unwrap(),
                normal,
            }),
            6 => self.tiles.push(Tile::Hexagon {
                adjecencies: *tile_adjacencies.as_array().unwrap(),
                vertices: *vertex_indices.map(|i| i as usize).as_array().unwrap(),
                normal,
            }),
            _ => panic!("add_tile only supports pentagons and hexagons"),
        }
    }

    pub fn add_normal(&mut self, indexed_normal: IndexedNormal) {
        let bin = Self::find_closest_normal_internal(
            self.fibonacci_sphere_points.iter(),
            indexed_normal.normal,
        )
        .index;
        match self.normal_map.get_mut(&bin) {
            Some(vec) => vec.push(indexed_normal),
            None => {
                self.normal_map.insert(bin, vec![indexed_normal]);
            }
        }
    }

    pub fn find_closest_normal(&self, normal: Vec3) -> &IndexedNormal {
        let bin =
            Self::find_closest_normal_internal(self.fibonacci_sphere_points.iter(), normal).index;
        Self::find_closest_normal_internal(self.normal_map[&bin].iter(), normal)
    }
}

const SPHERE_BIN_COUNT: usize = 3;

#[derive(Component)]
struct SphereMeshMarker;

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Create and save a handle to the mesh.
    pub const SUBDIVISIONS: u32 = 160;
    pub const TILES: usize = ExactGlobe::<SUBDIVISIONS>::FACES;
    let globe = ExactGlobe::<SUBDIVISIONS>::new();
    let centroids = globe.centroids(None);
    let vertices = globe.mesh_vertices(&centroids);
    let faces = globe.mesh_faces();
    let triangles = globe.mesh_triangles(&faces);
    let normals = globe.mesh_normals(&vertices);
    let mut edges = globe.adjacency();
    // Despite assurances this contains duplicate edges
    edges.sort_unstable();
    // This will not remove duplicates like (2, 191) and (191, 2)
    edges.dedup();
    // Remove final duplicates
    let edges: Vec<(usize, usize)> = edges
        .iter()
        .filter(|(a, b)| {
            if edges.binary_search(&(*a, *b)).is_ok() && edges.binary_search(&(*b, *a)).is_ok() {
                // This has to only filter out one of the occurances
                a > b
            } else {
                true
            }
        })
        .cloned()
        .collect();
    let mut edge_map = FxHashMap::<usize, Vec<usize>>::default();
    for (a, b) in edges.iter() {
        match edge_map.get_mut(a) {
            Some(bin) => bin.push(*b),
            None => {
                edge_map.insert(*a, vec![*b]);
            }
        }
        match edge_map.get_mut(b) {
            Some(bin) => bin.push(*a),
            None => {
                edge_map.insert(*b, vec![*a]);
            }
        }
    }

    let mut colors: Vec<[f32; 4]> = Vec::new();

    println!("Face amount: {:?}", globe.count_faces());

    let mut sphere_info = SphereInfo::<SPHERE_BIN_COUNT>::new(TILES);

    let mut rng = rand::rng();

    for i in 0..TILES {
        // Per face random color
        let face_color = vec![[rng.random(), rng.random(), rng.random(), 1.0]];
        let face_color_cycle = face_color.iter().cycle();
        // Sort out face normals and tiles
        // All the vertices in each face share the same normal, so just take the first
        match faces[i] {
            hexglobe::projection::globe::MeshFace::Pentagon(vertex_indices) => {
                colors.append(&mut face_color_cycle.take(5).cloned().collect());
                let normal = normals[vertex_indices[0] as usize].into();
                sphere_info.add_tile::<5>(i, normal, &edge_map, &vertex_indices);
                sphere_info.add_normal(IndexedNormal {
                    index: i,
                    normal: normal,
                });
            }
            hexglobe::projection::globe::MeshFace::Hexagon(vertex_indices) => {
                colors.append(&mut face_color_cycle.take(6).cloned().collect());
                let normal = normals[vertex_indices[0] as usize].into();
                sphere_info.add_tile::<6>(i, normal, &edge_map, &vertex_indices);
                sphere_info.add_normal(IndexedNormal {
                    index: i,
                    normal: normal,
                });
            }
        }
    }

    commands.insert_resource(sphere_info);

    let mesh_handle = meshes.add(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_indices(Indices::U32(triangles))
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors),
    );

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
    sphere_info: Res<SphereInfo<SPHERE_BIN_COUNT>>,
    sphere_mesh_query: Query<&Mesh3d, With<SphereMeshMarker>>,
    mut gizmos: Gizmos,
    meshes: ResMut<Assets<Mesh>>,
) {
    let window = window_query.single().unwrap();
    let aspect_ratio = window.size().x / window.size().y;
    let (camera_projection, camera_translation) = camera_query.single().unwrap();
    let sphere_mesh = meshes.get(sphere_mesh_query.single().unwrap()).unwrap();
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
                let cursor_tile_index = sphere_info.find_closest_normal(normal).index;
                let cursor_tile = &sphere_info.tiles[cursor_tile_index];

                // Draw the selected tile
                // TODO: Will this perform a copy from the render world each frame? Should I just keep my own copy of the vertices in sphere info and sync as needed?
                if let Some(VertexAttributeValues::Float32x3(positions)) =
                    sphere_mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                {
                    // Draw selected tile
                    gizmos.linestrip(
                        cursor_tile
                            .vertices()
                            .chain(std::iter::once(cursor_tile.vertices().next().unwrap()))
                            .map(|i| {
                                let v: Vec3 = positions[*i].into();
                                v * 1.001
                            }),
                        Color::LinearRgba(LinearRgba::WHITE),
                    );
                    // Draw connected tiles
                    for index in cursor_tile.adjecencies() {
                        let adjacent_tile = &sphere_info.tiles[*index];
                        gizmos.linestrip(
                            adjacent_tile
                                .vertices()
                                .chain(std::iter::once(adjacent_tile.vertices().next().unwrap()))
                                .map(|i| {
                                    let v: Vec3 = positions[*i].into();
                                    v * 1.001
                                }),
                            Color::LinearRgba(LinearRgba::GREEN),
                        );
                    }
                }
            }
        }
    }
}
