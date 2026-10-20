//! Reproducible finite-domain timings; payload estimates exclude allocator overhead.
use super::*;
use crate::{
    core::{
        cell::{CellCtx, CellOutcome, CellState},
        dim::Dimensionality,
    },
    grid::DenseGrid,
};
use glam::IVec3;
use serde_json::{Value, json};
use std::time::Instant;
mod rule;
use rule::Life;
#[test]
#[ignore = "measurement benchmark: set SOUL_BENCH_REPORT to retain JSON"]
fn sparse_benchmark() {
    let mut rows = Vec::new();
    for side in [64, 128] {
        for percent in [1, 10, 50] {
            let size = UVec3::new(side, side, 1);
            let mut dense = DenseGrid::blank(size);
            let mut sparse = SparseGrid::default();
            let mut seed = 0x534f554cu64;
            for y in 0..side {
                for x in 0..side {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    if (seed >> 32) % 100 < percent {
                        let p = IVec3::new(x as i32, y as i32, 0);
                        dense.set_state(p, CellState::Alive(255));
                        sparse.set_state(p, CellState::Alive(255));
                    }
                }
            }
            for backend in [
                "dense",
                "dense_parallel",
                "sparse_bounded",
                "sparse_bounded_parallel",
            ] {
                let mut samples = Vec::new();
                let mut occupied = 0;
                for _ in 0..5 {
                    let mut d = dense.clone();
                    let mut s = sparse.clone();
                    let start = Instant::now();
                    for _ in 0..10 {
                        match backend {
                            "dense" => {
                                crate::stepper::dense::step_dense(&mut d, &Life, &Value::Null)
                            }
                            "dense_parallel" => {
                                crate::stepper::dense::parallel::step_dense_dyn_parallel(
                                    &mut d,
                                    &Life,
                                    &Value::Null,
                                )
                            }
                            "sparse_bounded" => {
                                step_sparse_bounded(&mut s, &Life, &Value::Null, size)
                            }
                            _ => parallel::step_sparse_bounded_parallel(
                                &mut s,
                                &Life,
                                &Value::Null,
                                size,
                            ),
                        }
                    }
                    samples.push(start.elapsed().as_micros() as u64);
                    occupied = if backend.starts_with("dense") {
                        d.voxels
                            .iter()
                            .filter(|c| c.state != CellState::Dead)
                            .count()
                    } else {
                        s.map.len()
                    };
                    std::hint::black_box((&d, &s));
                }
                samples.sort_unstable();
                rows.push(json!({"side":side,"initial_percent":percent,"backend":backend,"ticks":10,"median_us":samples[2],"samples_us":samples,"final_occupied":occupied,"dense_cell_payload_bytes":dense.voxels.len()*std::mem::size_of::<crate::core::cell::Cell>(),"sparse_final_entry_payload_bytes":occupied*std::mem::size_of::<(IVec3,crate::core::cell::Cell)>()}));
            }
        }
    }
    let report = json!({"seed":0x534f554cu64,"rayon_threads":rayon::current_num_threads(),"boundary":"finite dead; complete domain for spontaneous births","memory_note":"payload estimates exclude HashMap capacity, allocator and temporary frontier buffers","measurements":rows});
    let bytes = serde_json::to_vec_pretty(&report).unwrap();
    if let Some(path) = std::env::var_os("SOUL_BENCH_REPORT") {
        std::fs::write(path, &bytes).unwrap();
    }
    println!("{}", String::from_utf8(bytes).unwrap());
}
