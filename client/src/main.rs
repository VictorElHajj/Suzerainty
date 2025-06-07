use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin}, prelude::*, render::{
        mesh::Indices,
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    }
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use hexglobe::projection::globe::ExactGlobe;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            WireframePlugin::default(),
            PanOrbitCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle_wireframe))
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Create and save a handle to the mesh.
    let mesh_handle: Handle<Mesh> = meshes.add(create_mesh());
    
    // Render the mesh with the custom texture, and add the marker.
    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(Color::WHITE)),
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
        Camera3d::default(),
        PanOrbitCamera {
            focus: Transform::IDENTITY.translation,
            radius: Some(2.),
            zoom_lower_limit: 1.1,
            zoom_upper_limit: Some(10.),
            allow_upside_down: false,
            pan_sensitivity: 0.,
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn create_mesh() -> Mesh {
    let globe = ExactGlobe::<1>::new();
    let centroids = globe.centroids(None);
    let vertices = globe.mesh_vertices(&centroids);
    let faces = globe.mesh_faces();
    let triangles = globe.mesh_triangles(&faces);
    let normals = globe.mesh_normals(&vertices);
    
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vertices
        )
        .with_inserted_indices(Indices::U32(
            triangles
        ))
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            normals
        )
}

fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
    }
}
