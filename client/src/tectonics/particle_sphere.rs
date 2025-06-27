use bevy::prelude::*;
use subsphere::{Face, Sphere, Vertex, proj::Fuller};

use crate::hex_sphere::vec_utils;

#[derive(Resource, Clone, Copy)]
pub struct ParticleSphereConfig {
    pub subdivisions: u32,
}

#[derive(Clone)]
pub struct ParticleTile {
    /// Index to [subsphere::hex::Face<Fuller>] (same index in wrapper and subsphere)
    pub index: usize,
    /// Indices to adjacent tiles
    pub adjacent: Vec<usize>,
    /// Tile face normal
    pub normal: Vec3,
}

#[derive(Resource)]
pub struct ParticleSphere {
    pub subsphere: subsphere::HexSphere<Fuller>,
    pub tiles: Vec<ParticleTile>,
}

impl ParticleSphere {
    pub fn from_config(config: ParticleSphereConfig) -> Self {
        let c = config.subdivisions % 3;
        let hex_sphere = subsphere::HexSphere::from_kis(subsphere::TriSphere::new(
            subsphere::BaseTriSphere::Icosa,
            subsphere::proj::Fuller,
            std::num::NonZero::new(config.subdivisions).unwrap(),
            c,
        ))
        .unwrap();
        let mut tiles: Vec<ParticleTile> = Vec::with_capacity(hex_sphere.num_faces());
        for (i, face) in hex_sphere.faces().enumerate() {
            let face_normal = vec_utils::f64_3_to_f32_3(&face.center().pos());
            let mut adjacent = face
                .vertices()
                .flat_map(|v| v.faces().map(|f| f.index()).collect::<Vec<usize>>())
                .collect::<Vec<usize>>();
            adjacent.sort_unstable();
            adjacent.dedup();
            tiles.push(ParticleTile {
                index: i,
                adjacent,
                normal: face_normal.into(),
            });
        }
        ParticleSphere {
            subsphere: hex_sphere,
            tiles,
        }
    }
}
