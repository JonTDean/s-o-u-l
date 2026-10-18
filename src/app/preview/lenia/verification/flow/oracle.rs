use super::*;
use engine_gpu::LeniaSnapshot;
use std::io::Write;

pub(super) fn verify(v: &Verification, snapshot: &LeniaSnapshot) {
    assert_eq!(
        (snapshot.epoch, snapshot.tick),
        (v.epoch, v.tick),
        "phase {} clock",
        v.phase
    );
    assert_eq!(snapshot.size, [64, 64, 8]);
    assert_eq!(snapshot.params, v.params);
    assert_eq!(snapshot.densities.len(), v.expected.len());
    if let Some(index) = snapshot
        .densities
        .iter()
        .zip(&v.expected)
        .position(|(actual, expected)| actual != expected)
    {
        panic!(
            "Lenia phase {} cell {index}: GPU {}, CPU {}",
            v.phase, snapshot.densities[index], v.expected[index]
        );
    }
    let alive = snapshot
        .densities
        .iter()
        .filter(|&&value| value > 0)
        .count();
    let sum: u64 = snapshot.densities.iter().map(|&value| value as u64).sum();
    let hash = snapshot
        .densities
        .iter()
        .fold(0xcbf29ce484222325u64, |hash, &byte| {
            (hash ^ byte as u64).wrapping_mul(0x100000001b3)
        });
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(v.output.join("readbacks.csv"))
        .unwrap();
    writeln!(
        file,
        "{},{},{},{alive},{sum},{hash:016x}",
        v.phase, v.epoch, v.tick
    )
    .unwrap();
    info!(
        "Lenia phase {} verified: epoch {}, tick {}, {alive} live cells, checksum {hash:016x}",
        v.phase, v.epoch, v.tick
    );
}

pub(super) fn cleared(v: &Verification) {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(v.output.join("readbacks.csv"))
        .unwrap();
    writeln!(file, "8,,,0,0,").unwrap();
    info!("Lenia clear verified: no live ECS owners or GPU slots");
}
