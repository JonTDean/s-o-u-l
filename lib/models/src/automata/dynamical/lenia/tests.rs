use super::*;
use glam::UVec3;
use simulation_kernel::{
    grid::DenseGrid,
    stepper::dense::{parallel::step_dense_dyn_parallel, step_dense},
};

#[test]
fn serial_and_parallel_models_match_reference_over_many_ticks() {
    let size = [7, 5, 3];
    let mut expected: Vec<u8> = (0..105).map(|i| ((i * 73 + 19) % 256) as u8).collect();
    let mut serial = DenseGrid::blank(UVec3::from_array(size));
    for (index, cell) in serial.voxels.iter_mut().enumerate() {
        cell.state = CellState::from(expected[index]);
        cell.memory = serde_json::json!({"index": index});
    }
    let mut parallel = serial.clone();
    let params = LeniaParams::default();
    let json = serde_json::to_value(params).unwrap();
    for tick in 1..=12 {
        expected = simulation_kernel::core::lenia::step(&expected, size, params).unwrap();
        step_dense(&mut serial, &LeniaRule, &json);
        step_dense_dyn_parallel(&mut parallel, &LeniaRule, &json);
        for (index, (a, b)) in serial.voxels.iter().zip(&parallel.voxels).enumerate() {
            assert_eq!(
                a.state,
                CellState::from(expected[index]),
                "tick {tick}, cell {index}"
            );
            assert_eq!(a.state, b.state);
            assert_eq!(a.memory, serde_json::json!({"index": index}));
            assert_eq!(a.memory, b.memory);
        }
    }
}

#[test]
fn invalid_growth_parameters_leave_model_state_unchanged() {
    let mut grid = DenseGrid::blank(UVec3::ONE);
    grid.voxels[0].state = CellState::Alive(180);
    step_dense(
        &mut grid,
        &LeniaRule,
        &serde_json::json!({"mu": 0.3, "sigma": 0.0, "dt": 0.05}),
    );
    assert_eq!(grid.voxels[0].state, CellState::Alive(180));
}
