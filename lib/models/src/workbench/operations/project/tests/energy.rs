//! Full checkpoint continuation includes both pools, both rules and fractional time.
use crate::workbench::{
    catalog::Kind,
    experiment::config::{
        authored::{
            Rule,
            composition::{Blend, Layer, Program},
        },
        energy::Energy,
    },
    plugin::Workbench,
    project::Project,
    session::Model,
};
#[test]
fn composed_energy_project_continues_and_rejects_corrupt_fields() {
    let mut bench = Workbench::default();
    bench.spawn(Kind::Life);
    let s = &mut bench.sessions[0];
    let mut settings = s.settings.clone();
    settings.program = Some(Program {
        primary: Rule::life(8, 12),
        secondary: Some(Layer {
            rule: Rule::copy_west(),
            blend: Blend::Union,
        }),
    });
    settings.energy = Some(Energy::default());
    s.configure(settings, true).unwrap();
    s.paused = false;
    s.elapse(0.25);
    let mut restored = Project::capture(&bench, vec![]).unwrap();
    for _ in 0..15 {
        bench.sessions[0].elapse(0.11);
        restored.sessions[0].elapse(0.11);
        assert_eq!(
            serde_json::to_value(&bench.sessions[0]).unwrap(),
            serde_json::to_value(&restored.sessions[0]).unwrap()
        );
    }
    let snapshot = serde_json::to_value(&restored).unwrap();
    let Model::Grid(grid) = &mut restored.sessions[0].model else {
        panic!("grid")
    };
    grid.energy.as_mut().unwrap()[0] = [u16::MAX; 2];
    assert!(restored.validate().is_err());
    restored = serde_json::from_value(snapshot.clone()).unwrap();
    let Model::Grid(grid) = &mut restored.sessions[0].model else {
        panic!("grid")
    };
    grid.energy.as_mut().unwrap().pop();
    assert!(restored.validate().is_err());
    restored = serde_json::from_value(snapshot).unwrap();
    restored.sessions[0].settings.energy = None;
    assert!(restored.validate().is_err());
}
#[test]
fn old_projects_default_to_no_program_or_energy() {
    let mut bench = Workbench::default();
    bench.spawn(Kind::Life);
    let mut value = serde_json::to_value(Project::capture(&bench, vec![]).unwrap()).unwrap();
    let settings = value["sessions"][0]["settings"].as_object_mut().unwrap();
    settings.remove("program");
    settings.remove("energy");
    value["sessions"][0]["model"]["Grid"]
        .as_object_mut()
        .unwrap()
        .remove("energy");
    let restored: Project = serde_json::from_value(value).unwrap();
    restored.validate().unwrap();
    assert!(restored.sessions[0].settings.program.is_none());
    assert!(restored.sessions[0].settings.energy.is_none());
}
