use crate::workbench::{
    catalog::Kind,
    experiment::config::{Settings, energy::Energy},
    session::{Model, Session},
};
use simulation_kernel::core::boundary::Boundary;
fn fixture(boundary: Boundary) -> Session {
    let mut s = Session::new(Kind::Life);
    let mut settings = Settings::for_kind(Kind::Life);
    settings.world.width = 8;
    settings.world.height = 8;
    settings.world.boundary = boundary;
    settings.world.seed = Some(vec![0; 64]);
    settings.energy = Some(Energy {
        capacity: [512; 2],
        initial: [0; 2],
        diffusion: [1000; 2],
        decay: [0; 2],
        recharge: [0; 2],
        birth_cost: [0; 2],
        survival_cost: [0; 2],
    });
    s.configure(settings, true).unwrap();
    s
}
fn grid(s: &mut Session) -> &mut super::GridSession {
    let Model::Grid(grid) = &mut s.model else {
        panic!("grid")
    };
    grid
}
#[test]
fn diffusion_matches_hand_computed_impulse_and_boundary_loss() {
    for (boundary, neighbors) in [
        (Boundary::Periodic, vec![1, 7, 8, 56]),
        (Boundary::Dead, vec![1, 8]),
    ] {
        let mut s = fixture(boundary);
        grid(&mut s).energy.as_mut().unwrap()[0] = [400, 200];
        s.step();
        for (i, pools) in grid(&mut s).energy.as_ref().unwrap().iter().enumerate() {
            assert_eq!(
                *pools,
                if neighbors.contains(&i) {
                    [100, 50]
                } else {
                    [0, 0]
                }
            );
        }
    }
}
#[test]
fn costs_require_both_pools_and_only_successful_candidates_pay() {
    let mut s = fixture(Boundary::Periodic);
    let mut settings = s.settings.clone();
    settings.birth = 1; // every empty cell has zero occupied neighbors
    let energy = settings.energy.as_mut().unwrap();
    energy.initial = [10, 3];
    energy.diffusion = [0; 2];
    energy.birth_cost = [4, 4];
    s.configure(settings.clone(), true).unwrap();
    s.step();
    assert!(grid(&mut s).bytes().iter().all(|v| *v == 0));
    assert!(
        grid(&mut s)
            .energy
            .as_ref()
            .unwrap()
            .iter()
            .all(|p| *p == [10, 3])
    );
    settings.energy.as_mut().unwrap().recharge = [0, 1];
    s.configure(settings, true).unwrap();
    s.step();
    assert!(grid(&mut s).bytes().iter().all(|v| *v == 255));
    assert!(
        grid(&mut s)
            .energy
            .as_ref()
            .unwrap()
            .iter()
            .all(|p| *p == [6, 0])
    );
}
#[test]
fn reset_only_configuration_paint_and_undo_preserve_pool_contract() {
    let mut s = fixture(Boundary::Periodic);
    grid(&mut s).energy.as_mut().unwrap()[0] = [400, 200];
    s.step();
    let before = serde_json::to_value(&s).unwrap();
    let mut changed = s.settings.clone();
    changed.energy.as_mut().unwrap().decay = [1; 2];
    assert!(s.configure(changed.clone(), false).is_err());
    assert_eq!(serde_json::to_value(&s).unwrap(), before);
    s.paint(&[[2, 2]], 255, true).unwrap();
    assert!(
        grid(&mut s)
            .energy
            .as_ref()
            .unwrap()
            .iter()
            .all(|p| *p == [0, 0])
    );
    s.undo_paint(false).unwrap();
    assert_eq!(serde_json::to_value(&s).unwrap(), before);
    s.undo_paint(true).unwrap();
    s.configure(changed, true).unwrap();
    assert_eq!(s.ticks, 0);
}
#[test]
fn decay_then_recharge_clamps_and_survival_exhausts() {
    let mut s = fixture(Boundary::Periodic);
    let mut settings = s.settings.clone();
    settings.world.seed = Some(vec![255; 64]);
    settings.survival = 1 << 8;
    let e = settings.energy.as_mut().unwrap();
    e.initial = [3; 2];
    e.diffusion = [0; 2];
    e.decay = [2; 2];
    e.recharge = [1; 2];
    e.survival_cost = [2; 2];
    s.configure(settings.clone(), true).unwrap();
    s.step();
    assert!(grid(&mut s).bytes().iter().all(|v| *v == 255));
    assert!(
        grid(&mut s)
            .energy
            .as_ref()
            .unwrap()
            .iter()
            .all(|p| *p == [0, 0])
    );
    s.step();
    assert!(grid(&mut s).bytes().iter().all(|v| *v == 0));
    assert!(
        grid(&mut s)
            .energy
            .as_ref()
            .unwrap()
            .iter()
            .all(|p| *p == [1, 1])
    );
    let e = settings.energy.as_mut().unwrap();
    e.initial = [512; 2];
    e.recharge = [512; 2];
    e.survival_cost = [0; 2];
    s.configure(settings, true).unwrap();
    s.step();
    assert!(
        grid(&mut s)
            .energy
            .as_ref()
            .unwrap()
            .iter()
            .all(|p| *p == [512, 512])
    );
}
