use super::*;
use crate::automata::dynamical::particle::hpp::{N, S};

#[test]
fn hpp_particles_keep_direction_and_head_on_collisions_turn_perpendicularly() {
    for (velocity, direction) in [(N, IVec3::Y), (S, -IVec3::Y), (E, IVec3::X), (W, -IVec3::X)] {
        let mut grid = DenseGrid::blank(UVec3::new(9, 9, 1));
        let origin = IVec3::new(4, 4, 0);
        grid.set_state(origin, CellState::Alive(velocity));
        for tick in 1..=3 {
            step_dense(&mut grid, &HPPRule, &serde_json::Value::Null);
            assert_eq!(
                grid.get(origin + direction * tick).unwrap().state,
                CellState::Alive(velocity)
            );
            assert_eq!(
                grid.voxels
                    .iter()
                    .filter(|c| c.state != CellState::Dead)
                    .count(),
                1
            );
        }
    }
    for incoming in 0..16_u8 {
        let mut grid = DenseGrid::blank(UVec3::new(3, 3, 1));
        for (bit, p) in [
            (N, IVec3::new(1, 0, 0)),
            (S, IVec3::new(1, 2, 0)),
            (E, IVec3::new(0, 1, 0)),
            (W, IVec3::new(2, 1, 0)),
        ] {
            grid.set_state(p, CellState::from(incoming & bit));
        }
        step_dense(&mut grid, &HPPRule, &serde_json::Value::Null);
        let expected = if incoming == N | S {
            E | W
        } else if incoming == E | W {
            N | S
        } else {
            incoming
        };
        assert_eq!(
            grid.get(IVec3::new(1, 1, 0)).unwrap().state,
            CellState::from(expected)
        );
        assert_eq!(incoming.count_ones(), expected.count_ones());
    }
}
