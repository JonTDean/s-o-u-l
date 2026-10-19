use crate::workbench::{
    catalog::Kind,
    plugin::Workbench,
    project::Project,
    session::{Model, Session},
};
#[test]
fn every_family_continues_after_roundtrip() {
    for kind in Kind::ALL {
        let mut bench = Workbench::default();
        bench.spawn(kind);
        for _ in 0..3 {
            bench.sessions[0].step();
        }
        let project = Project::capture(&bench, Vec::new()).unwrap();
        let bytes = serde_json::to_vec(&project).unwrap();
        let mut restored: Project = serde_json::from_slice(&bytes).unwrap();
        restored.validate().unwrap();
        for _ in 0..15 {
            let original = &mut bench.sessions[0];
            let loaded = &mut restored.sessions[0];
            original.step();
            loaded.step();
            assert_eq!(original.ticks, loaded.ticks, "{kind:?}");
            if let (
                Model::Symbolic(crate::workbench::symbolic::machine::Symbolic::Finite(a)),
                Model::Symbolic(crate::workbench::symbolic::machine::Symbolic::Finite(b)),
            ) = (&original.model, &loaded.model)
            {
                assert_eq!(a.states, b.states);
                assert_eq!(a.cursor, b.cursor);
                assert_eq!(a.result, b.result);
            } else {
                assert_eq!(
                    serde_json::to_value(&original.model).unwrap(),
                    serde_json::to_value(&loaded.model).unwrap(),
                    "{kind:?}"
                );
            }
        }
    }
}
#[test]
fn live_edit_reset_and_fractional_clock_contract() {
    let mut s = Session::new(Kind::Life);
    s.paused = false;
    s.elapse(0.04);
    let mut loaded: Session = serde_json::from_value(serde_json::to_value(&s).unwrap()).unwrap();
    s.elapse(0.07);
    loaded.elapse(0.07);
    assert_eq!(s.ticks, 1);
    assert_eq!(s.elapsed, loaded.elapsed);
    let before = serde_json::to_value(&s).unwrap();
    let mut invalid = s.settings.clone();
    invalid.rate_hz = f32::NAN;
    assert!(s.configure(invalid, false).is_err());
    assert_eq!(before, serde_json::to_value(&s).unwrap());
    let mut seed = s.settings.clone();
    seed.random_seed += 1;
    assert!(s.configure(seed.clone(), false).is_err());
    assert_eq!(before, serde_json::to_value(&s).unwrap());
    s.configure(seed, true).unwrap();
    assert_eq!(s.ticks, 0);
    assert!(s.paused);
}
