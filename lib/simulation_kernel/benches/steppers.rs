use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use simulation_kernel::{
    core::lenia::{self, continuous},
    stepper::{dense, sparse},
};
use std::{hint::black_box, time::Duration};
mod fixtures;
fn benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("life_step");
    group
        .sample_size(10)
        .warm_up_time(Duration::from_millis(200))
        .measurement_time(Duration::from_secs(1));
    for side in [64, 128] {
        for percent in [1, 10, 50] {
            let (d, s) = fixtures::grids(side, percent);
            for backend in ["dense", "dense_parallel", "sparse", "sparse_parallel"] {
                group.bench_function(
                    BenchmarkId::new(backend, format!("{side}-{percent}pct")),
                    |b| {
                        b.iter_batched_ref(
                            || (d.clone(), s.clone()),
                            |(d, s)| {
                                let size = glam::UVec3::new(side, side, 1);
                                match backend {
                                    "dense" => dense::step_dense(
                                        d,
                                        &fixtures::Life,
                                        &serde_json::Value::Null,
                                    ),
                                    "dense_parallel" => dense::parallel::step_dense_dyn_parallel(
                                        d,
                                        &fixtures::Life,
                                        &serde_json::Value::Null,
                                    ),
                                    "sparse" => sparse::step_sparse_bounded(
                                        s,
                                        &fixtures::Life,
                                        &serde_json::Value::Null,
                                        size,
                                    ),
                                    _ => sparse::parallel::step_sparse_bounded_parallel(
                                        s,
                                        &fixtures::Life,
                                        &serde_json::Value::Null,
                                        size,
                                    ),
                                }
                                black_box((d, s));
                            },
                            BatchSize::SmallInput,
                        );
                    },
                );
            }
        }
    }
    group.finish();
    let field = continuous::orbium::seed([64, 64, 1]);
    c.bench_function("orbium_xy_step", |b| {
        b.iter(|| {
            black_box(
                continuous::step(
                    &field,
                    [64, 64, 1],
                    continuous::orbium::KERNEL,
                    continuous::orbium::PARAMS,
                )
                .unwrap(),
            )
        })
    });
    let bytes = lenia::seed([16, 16, 8], lenia::LeniaSeed::Blob);
    c.bench_function("quantized_xy_step", |b| {
        b.iter(|| {
            black_box(lenia::step(&bytes, [16, 16, 8], lenia::LeniaParams::default()).unwrap())
        })
    });
    let field: Vec<_> = bytes.iter().map(|v| *v as f32 / 255.0).collect();
    c.bench_function("continuous_xyz_step", |b| {
        b.iter(|| {
            black_box(
                continuous::step(
                    &field,
                    [16, 16, 8],
                    continuous::Kernel::default(),
                    lenia::LeniaParams::default(),
                )
                .unwrap(),
            )
        })
    });
}
criterion_group!(benches, benchmarks);
criterion_main!(benches);
