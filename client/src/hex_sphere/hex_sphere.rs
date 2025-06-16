use bevy::prelude::*;
use subsphere::{Face, Sphere, proj::Fuller};

use crate::hex_sphere::{Tile, vec_utils};

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
