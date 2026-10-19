use crate::workbench::{
    catalog::Kind,
    experiment::config::Pattern,
    session::{Model, Session},
};
#[test]
fn seeded_patterns_reproduce_and_live_rules_keep_state() {
    let mut a = Session::new(Kind::Life);
    let mut settings = a.settings.clone();
    settings.pattern = Pattern::Noise;
    settings.random_seed = 12;
    a.configure(settings.clone(), true).unwrap();
    let mut b = Session::new(Kind::Life);
    b.configure(settings, true).unwrap();
    let bytes = |s: &Session| {
        let Model::Grid(g) = &s.model else { panic!() };
        g.bytes()
    };
    assert_eq!(bytes(&a), bytes(&b));
    let mut changed = b.settings.clone();
    changed.random_seed += 1;
    b.configure(changed, true).unwrap();
    assert_ne!(bytes(&a), bytes(&b));
    let initial = bytes(&a);
    let mut live = a.settings.clone();
    live.birth = 1;
    live.survival = 0;
    a.configure(live, false).unwrap();
    assert_eq!(initial, bytes(&a));
    a.step();
    assert_ne!(initial, bytes(&a));
}
#[test]
fn agent_population_edits_require_reset_and_controls_are_finite() {
    for kind in [Kind::Boids, Kind::Ants] {
        let mut session = Session::new(kind);
        session.step();
        let mut settings = session.settings.clone();
        settings.agent_count = 8;
        assert!(session.configure(settings.clone(), false).is_err());
        assert_eq!(session.ticks, 1);
        session.configure(settings, true).unwrap();
        assert_eq!(session.ticks, 0);
        match &session.model {
            Model::Boids(f) => assert_eq!(f.boids.len(), 8),
            Model::Ants(a) => assert_eq!(a.colony.ants.len(), 8),
            _ => panic!(),
        }
        let mut invalid = session.settings.clone();
        invalid.evaporation = f32::INFINITY;
        assert!(session.configure(invalid, false).is_err());
    }
}
