use std::f32;

use bevy::{
    math::{
        Rect, Vec2, Vec3,
        primitives::{Plane3d, RegularPolygon},
    },
    render::mesh::Meshable,
};

use crate::Tile;

/// A [Goldberg polyhedron](https://en.wikipedia.org/wiki/Goldberg_polyhedron)
/// generated from the [dual](https://en.wikipedia.org/wiki/Dual_polyhedron)
/// of a icosphere, itself generated from a subdivided icosahedron.
///
/// Will always contain exactly 12 pentagons and the remaining faces will be hexagons
pub struct HexSphere<const SUBDIVISIONS: u32> {
    pub vertices: Vec<Vec3>,
    pub tiles: Vec<Tile>,
}

impl<const SUBDIVISIONS: u32> HexSphere<SUBDIVISIONS> {
    pub fn new() -> Self {
        // First, we need to create a icosahedron using 3 perpinduclar rectangles
        let width = 1.;
        let height = width / f32::consts::PHI;
        let half_size = Vec2::new(height, width);
        let icosahedron = RegularPolygon::new(1., 12).mesh();
        // The corner points of the rectangles form the vertices.
        HexSphere {
            vertices: vec![],
            tiles: vec![],
        }
    }
}
