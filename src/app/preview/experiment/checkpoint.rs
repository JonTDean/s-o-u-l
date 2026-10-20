use super::*;
use engine_core::automata::lenia::LeniaSimulation;
use models::workbench::{
    experiment::export::write_bundle,
    plugin::Workbench,
    project::{self, Project},
};
use ui::lab::chrome::drawers::{Files, Pending};
pub(super) fn save(world: &mut World, v: &Verification, owner: Entity, sim: LeniaSimulation) {
    let project = Project::capture(world.resource::<Workbench>(), Vec::new()).unwrap();
    let mut files = world.resource_mut::<Files>();
    files.path = v.output.join("project.json").to_string_lossy().into_owned();
    files.save = Some(Pending {
        specimen: false,
        project,
        owners: vec![(owner, sim)],
        path: files.path.clone().into(),
        started: std::time::Instant::now(),
    });
}
pub(super) fn load(world: &mut World, v: &Verification) -> bool {
    let files = world.resource::<Files>();
    if files.save.is_some() {
        return false;
    }
    assert!(files.status.starts_with("Saved"), "{}", files.status);
    let mut project = project::read(std::path::Path::new(&files.path)).unwrap();
    assert_eq!(project.lenia[0].densities, v.expected);
    assert_eq!(project.lenia[0].tick, 4);
    assert_eq!(project.sessions[0].ticks, 12);
    assert_eq!(project.sessions[1].settings.elementary, 90);
    write_bundle(&project, &v.output.join("export"), 8, 100).unwrap();
    world.send_event(AutomataCommand::Clear);
    for saved in project.lenia.drain(..) {
        world.send_event(AutomataCommand::RestoreLenia(Box::new(saved)));
    }
    world.resource_mut::<Files>().load = Some(project);
    true
}
