use crate::hex_sphere::{HexSphere, HexSphereMeshHandle};
use crate::tectonics::TectonicsIteration;
use bevy::prelude::*;
use kdtree::KdTree;
use rayon::prelude::*;
use suz_sim::tectonics::{CONTINENTAL_HEIGHT, OCEANIC_HEIGHT, Tectonics};
use suz_sim::vec_utils;

pub fn interpolate_vertices(
    mut meshes: ResMut<Assets<Mesh>>,
    mut hex_sphere: ResMut<HexSphere>,
    tectonics: Res<Tectonics>,
    tectonics_iteration: Res<TectonicsIteration>,
    mesh_handle: Res<HexSphereMeshHandle>,
) {
    if tectonics_iteration.0 % 40 == 0 {
        // 1. For each tile, compute average height from nearby point masses, update tile height and center vertex height
        let mut kdtree = KdTree::<f32, (_, f32), [f32; 3]>::new(3);
        for (point_mass, plate_type, spring_compressions) in
            tectonics.plates.iter().flat_map(|plate| {
                plate
                    .shape
                    .par_iter_point_masses_with_springs()
                    .map(|(point_mass, springs)| {
                        (
                            point_mass,
                            plate.plate_type,
                            springs.map(|spring| {
                                let pm_a = &plate.shape.point_masses[spring.anchor_a];
                                let pm_b = &plate.shape.point_masses[spring.anchor_b];
                                let compression: f32 =
                                    spring.rest_length - pm_a.geodesic_distance(&pm_b);
                                compression
                            }),
                        )
                    })
            })
        {
            kdtree
                .add(
                    point_mass.position.into(),
                    (plate_type, spring_compressions.sum::<f32>()),
                )
                .ok();
        }

        // TODO: Compute compression for each point mass once, put in spatial datastructure, should speed this up massively
        let tile_results: Vec<_> = hex_sphere
            .tiles
            .par_iter()
            .enumerate()
            .map(|(tile_index, tile)| {
                let mut weighted_sum = 0.0;
                let mut weight_total = 0.0;
                let tile_normal = tile.normal;
                let tile_center = tile.center;
                let position: [f32; 3] = tile_normal.into();
                for (distance, (plate_type, compression)) in kdtree
                    .within(
                        &position,
                        tectonics.config.vertex_interpolation_radius,
                        &vec_utils::geodesic_distance_arr,
                    )
                    .unwrap()
                {
                    let weight = 1.0 / (distance + 0.01); // closer = higher weight, avoid div by zero
                    let plate_height = match plate_type {
                        suz_sim::plate::PlateType::Oceanic => OCEANIC_HEIGHT,
                        suz_sim::plate::PlateType::Continental => CONTINENTAL_HEIGHT,
                    };
                    weighted_sum += (plate_height + compression) * weight;
                    weight_total += weight;
                }
                let new_height = if weight_total > 0.0 {
                    weighted_sum / weight_total
                } else {
                    OCEANIC_HEIGHT
                };
                let color = if new_height < 1.0 {
                    [0.0, 0.0, 1.0, 1.0] // blue for below 1.0
                } else {
                    [0.0, 1.0, 0.0, 1.0] // green for above
                };
                (tile_index, new_height, color, tile_center, tile_normal)
            })
            .collect();

        // Apply results sequentially to avoid race conditions
        for (tile_index, new_height, color, tile_center, tile_normal) in tile_results {
            hex_sphere.tiles[tile_index].height = new_height;
            hex_sphere.colors[tile_center] = color;
            hex_sphere.vertices[tile_center] = (tile_normal * new_height).into();
            for vertex_index in &hex_sphere.tiles[tile_index].vertices.clone() {
                hex_sphere.colors[*vertex_index] = color;
            }
        }

        // 2. Interpolate corner vertices using vertex_to_tiles (parallel, but collect first)
        let new_vertex_positions: Vec<_> = (0..hex_sphere.vertices_to_tiles.len())
            .into_par_iter()
            .map(|vertex_index| {
                let tile_indices = &hex_sphere.vertices_to_tiles[vertex_index];
                // Center vertex has no adjacent tiles, so we skip it
                if tile_indices.is_empty() {
                    return hex_sphere.vertices[vertex_index];
                }
                let mut sum = Vec3::ZERO;
                for tile_index in tile_indices {
                    let tile = &hex_sphere.tiles[*tile_index];
                    let normal = tile.normal;
                    let height = tile.height;
                    sum += normal * height;
                }
                (sum / 3.).into()
            })
            .collect();
        for (vertex, new_pos) in hex_sphere.vertices.iter_mut().zip(new_vertex_positions) {
            *vertex = new_pos;
        }

        // 3. Update mesh
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            if hex_sphere.vertices.len() == mesh.count_vertices()
                && hex_sphere.colors.len() == mesh.count_vertices()
            {
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, hex_sphere.colors.clone());
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, hex_sphere.vertices.clone());
                mesh.compute_normals();
            } else {
                warn!(
                    "Vertex or color array length does not match mesh vertex count: vertices = {}, mesh = {}",
                    hex_sphere.vertices.len(),
                    mesh.count_vertices()
                );
            }
        }
    }
}
