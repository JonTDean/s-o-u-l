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
