use crate::workbench::{
    catalog::Kind,
    plugin::Workbench,
    project::Project,
    session::{Model, Session},
};
use simulation_kernel::core::boundary::Boundary;
fn bytes(s: &Session) -> Vec<u8> {
    let Model::Grid(g) = &s.model else { panic!() };
    g.bytes()
}
#[test]
fn stroke_undo_redo_reset_and_loaded_continuation() {
    let mut s = Session::new(Kind::Life);
    let mut settings = s.settings.clone();
    settings.world.width = 16;
    settings.world.height = 8;
    settings.world.boundary = Boundary::Periodic;
    settings.world.seed = Some(vec![0; 128]);
    s.configure(settings, true).unwrap();
    s.step();
    s.paint(&[[0, 0], [1, 0]], 255, true).unwrap();
    s.paint(&[[2, 0]], 255, false).unwrap();
    let seed = bytes(&s);
    assert_eq!(s.ticks, 0);
    s.undo_paint(false).unwrap();
    assert_eq!(s.ticks, 1);
    assert!(bytes(&s).iter().all(|v| *v == 0));
    s.undo_paint(true).unwrap();
    assert_eq!(bytes(&s), seed);
    s.step();
    assert_eq!(bytes(&s).iter().filter(|v| **v > 0).count(), 3);
    assert_eq!(
        bytes(&s)[7 * 16 + 1],
        255,
        "vertical oscillator wraps top edge"
    );
    let mut bench = Workbench::default();
    bench.sessions.push(s);
    let project = Project::capture(&bench, vec![]).unwrap();
    let mut loaded: Project =
        serde_json::from_slice(&serde_json::to_vec(&project).unwrap()).unwrap();
    loaded.validate().unwrap();
    for _ in 0..10 {
        bench.sessions[0].step();
        loaded.sessions[0].step();
        assert_eq!(bytes(&bench.sessions[0]), bytes(&loaded.sessions[0]));
    }
    loaded.sessions[0].reset().unwrap();
    assert_eq!(bytes(&loaded.sessions[0]), seed);
    assert!(!loaded.sessions[0].edits.can_undo());
}
#[test]
fn invalid_strokes_are_atomic_and_running_worlds_refuse_edits() {
    let mut s = Session::new(Kind::Hpp);
    let before = bytes(&s);
    assert!(s.paint(&[[0, 0], [64, 0]], 1, true).is_err());
    assert!(s.paint(&[[0, 0]], 16, true).is_err());
    assert!(!s.edits.can_undo());
    s.paused = false;
    assert!(s.paint(&[[0, 0]], 1, true).is_err());
    assert_eq!(bytes(&s), before);
}
#[test]
fn old_projects_default_worlds_and_new_world_limits_validate() {
    let mut bench = Workbench::default();
    bench.spawn(Kind::Rule30);
    let mut value = serde_json::to_value(Project::capture(&bench, vec![]).unwrap()).unwrap();
    value["version"] = 1.into();
    value["sessions"][0]["settings"]
        .as_object_mut()
        .unwrap()
        .remove("world");
    value["sessions"][0]["model"]["Grid"]
        .as_object_mut()
        .unwrap()
        .remove("boundary");
    let old: Project = serde_json::from_value(value).unwrap();
    old.validate().unwrap();
    let mut s = Session::new(Kind::Life);
    let mut settings = s.settings.clone();
    settings.world.width = 257;
    assert!(s.configure(settings, true).is_err());
}
