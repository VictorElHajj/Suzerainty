use client::{
    sphere_bins::SphereBins,
    tectonics::{
        ParticleSphere, ParticleSphereConfig, TectonicsConfiguration, particle::PlateParticle,
        setup, simulate,
    },
};
use criterion::{Criterion, criterion_group, criterion_main};
use rand::SeedableRng;

const BIN_COUNT: usize = 60;
const ITERATIONS: usize = 100;

fn tectonics_benchmark(c: &mut Criterion) {
    let mut particle_bins = SphereBins::<BIN_COUNT, PlateParticle>::new();
    let mut plates = Vec::new();
    let tectonics_config = TectonicsConfiguration {
        major_plate_fraction: 0.5,
        major_tile_fraction: 0.75,
        plate_goal: 10,
        continental_rate: 0.4,
        min_plate_size: 15,
        particle_force_radius: 0.20,
        repulsive_force_modifier: 0.01,
        attractive_force: 0.002,
        plate_force_modifier: 0.02,
        plate_rotation_drift_rate: 0.001,
        timestep: 0.3,
        iterations: 500,
        friction_coefficient: 0.5,
    };
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    let particle_sphere = ParticleSphere::from_config(ParticleSphereConfig { subdivisions: 32 });
    setup(
        &tectonics_config,
        &particle_sphere,
        &mut particle_bins,
        &mut plates,
        &mut rng,
    );
    c.bench_function("Tectonics paricle simulation", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                simulate(&mut particle_bins, &mut plates, &tectonics_config, &mut rng)
            }
        });
    });
}

criterion_group!(benches, tectonics_benchmark);
criterion_main!(benches);
