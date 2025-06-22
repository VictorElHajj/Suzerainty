use std::f32::consts::PI;

use bevy::math::Vec3;

pub trait GetNormal {
    fn normal(&self) -> Vec3;
}

pub struct Bin<T: Sized + GetNormal> {
    pub normal: Vec3,
    /// Aproximation of how large is bucket is on the sphere
    pub max_geodesic_distance: f32,
    pub items: Vec<T>,
}

/// Creates BINS bins equally across a sphere. Items are inserted with a unit sphere normal and put in the closest bucket.
pub struct SphereBins<const BINS: usize, T: Sized + GetNormal> {
    pub(crate) bins: [Bin<T>; BINS],
}

impl<const BINS: usize, T: Sized + GetNormal> SphereBins<BINS, T> {
    pub fn new() -> Self {
        let golden_angle = PI * (3. - f32::sqrt(5.));
        let offset: f32 = 2. / BINS as f32;
        let indices: [usize; BINS] = core::array::from_fn(|i| i);
        let bins = indices.map(|i| {
            let y = i as f32 * offset - 1. + offset / 2.;
            let r = (1. - y * y).sqrt();
            let phi = i as f32 * golden_angle;
            let x = f32::cos(phi) * r;
            let z = f32::sin(phi) * r;
            Bin::<T> {
                normal: Vec3::new(x, y, z),
                items: Vec::new(),
                max_geodesic_distance: f32::acos(1. - 2. / BINS as f32),
            }
        });
        return SphereBins { bins };
    }

    /// item is put in bin with closest normal
    pub fn insert(&mut self, item: T) {
        let closest_bin = self
            .bins
            .iter_mut()
            .max_by(|a, b| {
                item.normal()
                    .dot(a.normal)
                    .partial_cmp(&item.normal().dot(b.normal))
                    .unwrap()
            })
            .unwrap();
        closest_bin.items.push(item);
    }

    /// Returns an iterator with references for all items within the radius, across one or multiple bins
    pub fn get_within(&self, normal: Vec3, radius: f32) -> impl Iterator<Item = &T> {
        self.bins
            .iter()
            .filter(move |bin| {
                // Get sphere distance between input normal and bin normal
                let geodesic_distance = f32::acos(normal.dot(bin.normal));
                // if sphere distance is less than bin size + radius
                geodesic_distance < bin.max_geodesic_distance + radius
            })
            .flat_map(|bin| bin.items.iter())
    }

    pub fn get_closest(&self, normal: Vec3) -> &T {
        self.bins
            .iter()
            .max_by(|a, b| {
                normal
                    .dot(a.normal)
                    .partial_cmp(&normal.dot(b.normal))
                    .unwrap()
            })
            .expect("Sphere Bin had no bins.")
            .items
            .iter()
            .max_by(|a, b| {
                normal
                    .dot(a.normal())
                    .partial_cmp(&normal.dot(b.normal()))
                    .unwrap()
            })
            .expect("Closest bin had no items.")
    }
}
