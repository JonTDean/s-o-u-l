use super::*;
#[test]
fn scenarios_are_repeatable_and_paused() {
    for scenario in Scenario::ALL {
        let a = scenario.project().unwrap();
        let b = scenario.project().unwrap();
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&b).unwrap()
        );
        assert_eq!(a.sessions.len(), 2);
        assert!(a.sessions.iter().all(|s| s.paused && s.ticks == 0));
    }
}
#[test]
fn scenarios_reveal_their_documented_outcomes() {
    let mut boundaries = Scenario::Boundaries.project().unwrap();
    for s in &mut boundaries.sessions {
        s.step();
    }
    assert_ne!(
        serde_json::to_value(&boundaries.sessions[0].model).unwrap(),
        serde_json::to_value(&boundaries.sessions[1].model).unwrap()
    );
    let mut languages = Scenario::Languages.project().unwrap();
    for s in &mut languages.sessions {
        for _ in 0..16 {
            s.step();
        }
    }
    for (session, accepted) in languages.sessions.iter().zip([true, false]) {
        let crate::workbench::session::Model::Symbolic(machine) = &session.model else {
            panic!()
        };
        assert_eq!(machine.result(), Some(accepted));
    }
}
