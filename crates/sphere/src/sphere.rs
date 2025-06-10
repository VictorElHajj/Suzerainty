// use hexglobe::projection::globe::ExactGlobe;

// enum Tile {
//     Pentagon { index: usize },
//     Hexagon { index: usize },
// }

// impl Tile {
//     pub fn index(&self) -> &usize {
//         match self {
//             Tile::Pentagon { index, .. } => index,
//             Tile::Hexagon { index, .. } => index,
//         }
//     }
// }

// // A wrapper around HexSphere
// pub struct Sphere<const N: u32> {
//     globe: ExactGlobe<N>,
// }

// impl<const N: u32> Sphere<N> {
//     // Returns the amount of faces
//     pub fn tile_count(&self) -> usize {
//         self.globe.count_faces()
//     }

//     // Returns an iterator of adjacent tile indices
//     pub fn adjacenct(&self, face: usize) -> impl Iterator<Item = &usize> {
//         [0usize].iter()
//     }
// }
