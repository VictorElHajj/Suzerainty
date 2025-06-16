use bevy::{color::Color, gizmos::gizmos::Gizmos};

/// A helper for the modified faces with a central vertex
pub struct Tile {
    /// Index to [subsphere::hex::Face<Fuller>] (same index in wrapper and subsphere)
    pub index: usize,
    /// Index to the central vertex in HexSphere.vertices
    pub center: usize,
    /// Indices to corner vertices in HexSphere.vertices
    pub vertices: Vec<usize>,
    /// Height of the tile center
    pub height: f32,
    /// Indices to adjacent tiles
    pub adjacent: Vec<usize>,
}

impl Tile {
    pub fn draw_border(&self, vertices: &Vec<[f32; 3]>, color: Color, gizmos: &mut Gizmos) {
        gizmos.linestrip(
            self.vertices
                .iter()
                .chain(std::iter::once(&self.vertices[0]))
                .map(|vertex_index| vertices[*vertex_index].map(|val| val * 1.01).into()),
            color,
        );
    }
}
