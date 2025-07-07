use std::f32::consts::PI;

use bevy::math::Vec3;
use rayon::prelude::*;

use crate::vec_utils::geodesic_distance;

pub trait Binnable: Sized + Send + Sync {
    fn normal(&self) -> Vec3;
    fn id(&self) -> usize;
}

pub struct Bin {
    pub normal: Vec3,
    /// Aproximation of how large is bucket is on the sphere
    pub max_geodesic_distance: f32,
    /// Indices to SphereBins items
    pub indices: Vec<usize>,
}

/// Creates BINS bins equally across a sphere. Items are inserted with a unit sphere normal and put in the closest bucket.
pub struct SphereBins<const BINS: usize, T: Binnable> {
    pub(crate) bins: [Bin; BINS],
    /// A list of T's with unique ids, sorted by id
    pub items: Vec<T>,
}

impl<const BINS: usize, T: Binnable> SphereBins<BINS, T> {
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
            Bin {
                normal: Vec3::new(x, y, z),
                indices: Vec::new(),
                max_geodesic_distance: f32::acos(1. - 2. / BINS as f32),
            }
        });
        return SphereBins {
            bins,
            items: Vec::new(),
        };
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
        self.items.push(item);
        closest_bin.indices.push(self.items.len() - 1);
    }

    /// Returns an iterator with references for all items within the radius, across one or multiple bins
    pub fn get_within(&self, normal: Vec3, radius: f32) -> impl Iterator<Item = &T> {
        self.bins
            .iter()
            .filter(move |bin| {
                // if sphere distance is less than bin size + radius
                geodesic_distance(normal, bin.normal) < bin.max_geodesic_distance + radius
            })
            .flat_map(|bin| bin.indices.iter())
            .filter_map(move |index| {
                let item = &self.items[*index];
                let geodesic_distance = f32::acos(normal.dot(item.normal()));
                if geodesic_distance <= radius {
                    Some(item)
                } else {
                    None
                }
            })
    }

    /// Returns a iterator over all items
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    /// Returns a rayon parallel iterator over all items
    pub fn par_iter(&self) -> impl ParallelIterator<Item = &T> {
        self.items.par_iter()
    }

    /// Returns a rayon parallel mutable iterator over all items
    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T> {
        self.items.par_iter_mut()
    }

    /// Returns mutable iterator over all items
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }

    /// Returns item with normal closest to input normal
    pub fn get_closest(&self, normal: Vec3) -> &T {
        self.bins
            .iter()
            .filter(move |bin| {
                // Get sphere distance between input normal and bin normal
                let geodesic_distance = f32::acos(normal.dot(bin.normal));
                // if sphere distance is less than bin size + radius
                geodesic_distance < bin.max_geodesic_distance * 2.
            })
            .flat_map(|bin| bin.indices.iter().map(|index| &self.items[*index]))
            .max_by(|a, b| {
                normal
                    .dot(a.normal())
                    .partial_cmp(&normal.dot(b.normal()))
                    .expect(&format!(
                        "Failed to compare {:?} with {:?}",
                        a.normal(),
                        b.normal()
                    ))
            })
            .unwrap()
    }

    /// Checks all items, if any item is further away from the normal than the maximum expected bucket size, remove and re-add.
    pub fn refresh(&mut self) {
        for bin in self.bins.iter_mut() {
            bin.indices.clear();
        }
        // Re-assign each item to the closest bin
        for (idx, item) in self.items.iter().enumerate() {
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
            closest_bin.indices.push(idx);
        }
    }
}
