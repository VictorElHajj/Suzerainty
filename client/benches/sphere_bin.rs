use criterion::{Criterion, criterion_group, criterion_main};
use rand::SeedableRng;
use suz_sim::{
    particle_sphere::{ParticleSphere, ParticleSphereConfig},
    tectonics::{Tectonics, TectonicsConfiguration},
};

const ITERATIONS: usize = 100;

fn tectonics_benchmark(c: &mut Criterion) {
    let tectonics_config = TectonicsConfiguration {
        major_plate_fraction: 0.5,
        major_tile_fraction: 0.75,
        plate_goal: 10,
        continental_rate: 0.4,
        min_plate_size: 15,
        particle_force_radius: 0.20,
        repulsive_force_modifier: 0.01,
        link_spring_constant: 0.002,
        plate_force_modifier: 0.02,
        plate_rotation_drift_rate: 0.001,
        timestep: 0.3,
        iterations: 500,
        friction_coefficient: 0.5,
    };
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    let particle_sphere = ParticleSphere::from_config(ParticleSphereConfig { subdivisions: 32 });
    let mut tectonics = Tectonics::from_config(tectonics_config, &particle_sphere, &mut rng);
    c.bench_function("Tectonics particle simulation", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                tectonics.simulate(&mut rng);
            }
        });
    });
}

criterion_group!(benches, tectonics_benchmark);
criterion_main!(benches);
