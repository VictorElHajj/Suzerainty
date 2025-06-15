use bevy::math::Vec3;

/// A hex sphere tile is either a pentagon or hexagon.
pub enum Tile {
    Pentagon {
        /// Indices to other [Tile]s contained in the parent [HexSphere].
        adjecencies: [usize; 5],
        /// Indices to the vertices that represent this [Tile] in the parent [HexSphere] mesh.
        vertices: [usize; 5],
        normal: Vec3,
    },
    Hexagon {
        /// Indices to other [Tile]s contained in the parent [HexSphere].
        adjecencies: [usize; 6],
        /// Indices to the vertices that represent this [Tile] in the parent [HexSphere] mesh.
        vertices: [usize; 6],
        normal: Vec3,
    },
}

impl Tile {
    /// Indices to other [Tile]s contained in the parent [HexSphere].
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

    /// Indices to the vertices that represent this [Tile] in the parent [HexSphere] mesh.
    pub fn vertices(&self) -> impl Iterator<Item = &usize> {
        match &self {
            Tile::Pentagon { vertices, .. } => vertices.iter(),
            Tile::Hexagon { vertices, .. } => vertices.iter(),
        }
    }
}
