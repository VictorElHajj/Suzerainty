use bevy::prelude::*;

use crate::{
    hex_sphere::{CurrentMousePick, HexSphere, MousePickInfo},
    sphere_bins::SphereBins,
    states::SimulationState,
};

#[derive(Resource, Clone, Copy)]
pub struct TectonicsConfiguration {}

pub struct TectonicsPlugin {
    pub config: TectonicsConfiguration,
}
impl Plugin for TectonicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config)
            .add_systems(OnEnter(SimulationState::Tectonics), setup)
            .add_systems(Update, draw_bins);
    }
}

#[derive(Resource)]
struct TestBins(SphereBins<500, Vec3>);

fn setup(mut commands: Commands, hex_sphere: Res<HexSphere>) {
    let mut sphere_bins = SphereBins::<500, Vec3>::new();
    for tile in hex_sphere.tiles.iter() {
        sphere_bins.insert(tile.normal, tile.normal);
    }
    commands.insert_resource(TestBins(sphere_bins));
}

fn draw_bins(mut gizmos: Gizmos, bins: Res<TestBins>, current_mouse_pick: Res<CurrentMousePick>) {
    if let Some(MousePickInfo { tile, normal }) = &current_mouse_pick.0 {
        let radius = 0.05;
        gizmos.circle(
            Isometry3d {
                rotation: Quat::from_rotation_arc(Vec3::Z, *normal),
                translation: (normal * tile.height).into(),
            },
            radius,
            LinearRgba::BLUE,
        );
        for bin in &bins.0.bins {
            gizmos.arrow(bin.normal, bin.normal * 1.1, LinearRgba::RED);
        }
        for vec in bins.0.get_within(*normal, radius) {
            let geodesic_distance = f32::acos(normal.dot(*vec));
            if geodesic_distance > radius {
                gizmos.arrow(*vec, vec * 1.05, LinearRgba::WHITE);
            } else {
                let distance_fraction = geodesic_distance / radius;
                gizmos.arrow(
                    *vec,
                    vec * (1.15 - distance_fraction / 10.),
                    LinearRgba::new(distance_fraction, 1.0, 0.0, 1.0),
                );
            }
        }
    }
}
