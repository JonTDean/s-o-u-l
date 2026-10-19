use super::*;
use crate::automata::classical::regular::wolfram_1d::rules::{rule30::Rule30, rule110::Rule110};
use crate::automata::dynamical::particle::life::Life;
use simulation_kernel::stepper::dense::parallel::step_dense_dyn_parallel;

#[test]
fn elementary_rules_match_all_eight_binary_neighborhoods_in_three_dimensions() {
    for (number, rule) in [
        (30_u8, &Rule30 as &dyn AutomatonRule<D = Dim>),
        (110, &Rule110),
    ] {
        for pattern in 0..8 {
            let mut grid = DenseGrid::blank(UVec3::splat(3));
            for x in 0..3 {
                grid.set_state(
                    IVec3::new(x, 1, 1),
                    CellState::from(if pattern & (1 << (2 - x)) != 0 {
                        255
                    } else {
                        0
                    }),
                );
            }
            // Distractors outside the XY row must not influence the rule.
            grid.set_state(IVec3::new(0, 1, 0), CellState::Alive(255));
            step_dense(&mut grid, rule, &serde_json::Value::Null);
            let expected = CellState::from(if number & (1 << pattern) != 0 { 255 } else { 0 });
            assert_eq!(
                grid.get(IVec3::ONE).unwrap().state,
                expected,
                "rule {number}, pattern {pattern}"
            );
        }
    }
}

#[test]
fn life_blinker_period_and_glider_displacement_are_correct() {
    let mut grid = DenseGrid::blank(UVec3::new(9, 9, 1));
    for x in 3..=5 {
        grid.set_state(IVec3::new(x, 4, 0), CellState::Alive(255));
    }
    let original = grid.voxels.clone();
    step_dense(&mut grid, &Life, &serde_json::Value::Null);
    for y in 3..=5 {
        assert_eq!(
            grid.get(IVec3::new(4, y, 0)).unwrap().state,
            CellState::Alive(255)
        );
    }
    assert_eq!(
        grid.voxels
            .iter()
            .filter(|c| c.state != CellState::Dead)
            .count(),
        3
    );
    step_dense(&mut grid, &Life, &serde_json::Value::Null);
    assert!(
        grid.voxels
            .iter()
            .zip(original)
            .all(|(a, b)| a.state == b.state)
    );
    let mut session = GridSession::new(Kind::Life);
    let initial: Vec<_> = session
        .grid
        .iter()
        .filter(|(_, c)| c.state != CellState::Dead)
        .map(|(p, _)| p)
        .collect();
    for _ in 0..4 {
        session.step();
    }
    assert_eq!(session.bytes().iter().filter(|&&v| v != 0).count(), 5);
    for p in initial {
        assert_eq!(
            session.grid.get(p + IVec3::new(1, 1, 0)).unwrap().state,
            CellState::Alive(255)
        );
    }
}

#[test]
fn cellular_histories_and_parallel_updates_preserve_snapshot_semantics() {
    for kind in [Kind::Rule30, Kind::Rule110, Kind::Life, Kind::Hpp] {
        let mut session = GridSession::new(kind);
        let mut parallel = session.grid.clone();
        let rule: &(dyn AutomatonRule<D = Dim> + Sync) = match kind {
            Kind::Rule30 => &Rule30,
            Kind::Rule110 => &Rule110,
            Kind::Life => &Life,
            _ => &HPPRule,
        };
        for _ in 0..80 {
            session.step();
            step_dense_dyn_parallel(&mut parallel, rule, &serde_json::Value::Null);
            assert!(
                session
                    .grid
                    .voxels
                    .iter()
                    .zip(&parallel.voxels)
                    .all(|(a, b)| a.state == b.state)
            );
        }
        if matches!(kind, Kind::Rule30 | Kind::Rule110) {
            assert_eq!(session.history.len(), 64);
        }
    }
}
