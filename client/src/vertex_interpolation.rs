use crate::hex_sphere::{HexSphere, HexSphereMeshHandle};
use crate::tectonics::TectonicsIteration;
use bevy::prelude::*;
use rayon::prelude::*;
use suz_sim::sphere_bins::GetNormal;
use suz_sim::tectonics::Tectonics;

pub fn interpolate_vertices(
    mut meshes: ResMut<Assets<Mesh>>,
    mut hex_sphere: ResMut<HexSphere>,
    tectonics: Res<Tectonics>,
    tectonics_iteration: Res<TectonicsIteration>,
    mesh_handle: Res<HexSphereMeshHandle>,
) {
    if tectonics_iteration.0 % 10 == 0 {
        // 1. For each tile, compute average height from nearby particles, update tile height and center vertex height
        let tile_results: Vec<_> = hex_sphere
            .tiles
            .par_iter()
            .enumerate()
            .map(|(tile_index, tile)| {
                let mut weighted_sum = 0.0;
                let mut weight_total = 0.0;
                let tile_normal = tile.normal;
                let tile_height = tile.height;
                let tile_center = tile.center;
                for particle in tectonics
                    .particles
                    .get_within(tile_normal, tectonics.config.particle_force_radius)
                {
                    let dist = 1.0 - tile_normal.dot(particle.normal()); // geodesic cosine distance
                    let weight = 1.0 / (dist + 0.01); // closer = higher weight, avoid div by zero
                    weighted_sum += particle.height * weight;
                    weight_total += weight;
                }
                let new_height = if weight_total > 0.0 {
                    weighted_sum / weight_total
                } else {
                    tile_height
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
