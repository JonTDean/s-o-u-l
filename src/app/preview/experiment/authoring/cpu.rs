use super::*;
use engine_core::events::AutomataCommand;
use models::workbench::catalog::scenarios::Scenario;
pub(super) fn run(world: &mut World, v: &mut Verification) {
    match v.phase {
        0 if v.frames > 60 => {
            world.send_event(AutomataCommand::Clear);
            world.resource_mut::<Files>().load = Some(Scenario::Boundaries.project().unwrap());
            let mut lab = world.resource_mut::<LabState>();
            lab.guide.active = Some(Scenario::Boundaries);
            lab.drawer = None;
            v.phase = 1;
            v.wait = v.frames + 30;
        }
        1 => {
            let ids: Vec<_> = world
                .resource::<Workbench>()
                .sessions
                .iter()
                .map(|s| s.channel)
                .collect();
            if ids.len() != 2 || ids.contains(&ChannelId::UNASSIGNED) {
                return;
            }
            world.resource_mut::<Channels>().selected = Some(ids[0]);
            let mut c = world.resource_mut::<Comparison>();
            c.left = Some(ids[0]);
            c.right = Some(ids[1]);
            c.capture.request = Some(true);
            v.phase = 2;
        }
        2 => {
            let c = world.resource::<Comparison>();
            let Some(pair) = &c.capture.result else {
                return;
            };
            assert_eq!(pair[0].stamp.tick, 1);
            assert!(pair[0].field.compare(&pair[1].field).unwrap().maximum > 0.0);
            world.resource_mut::<Comparison>().folder =
                v.output.join("comparison").to_string_lossy().into();
            ui::lab::chrome::drawers::compare::export::write(
                world.resource::<Comparison>(),
                world.resource::<Channels>(),
                world.resource::<engine_core::automata::channel::metrics::Metrics>(),
            )
            .unwrap();
            let session = &world.resource::<Workbench>().sessions[0];
            let id = session.channel;
            v.expected = Some(serde_json::to_value(session).unwrap());
            let mut files = world.resource_mut::<Files>();
            files.path = v.output.join("Life.json").to_string_lossy().into();
            files.request = Some(Request::Specimen(id));
            v.phase = 3;
        }
        3 if v.output.join("Life.json").exists() => finish(world, v),
        12 => {
            let bench = world.resource::<Workbench>();
            assert_eq!(bench.sessions.len(), 1);
            assert_eq!(
                serde_json::to_value(&bench.sessions[0]).unwrap(),
                *v.expected.as_ref().unwrap()
            );
            world.send_event(AutomataCommand::Clear);
            world.resource_mut::<Files>().load = Some(Scenario::Languages.project().unwrap());
            let mut lab = world.resource_mut::<LabState>();
            lab.guide.active = Some(Scenario::Languages);
            lab.guide.step = 1;
            v.phase = 4;
            v.wait = v.frames + 60;
        }
        _ => {}
    }
}
fn finish(world: &mut World, v: &mut Verification) {
    let saved = project::read(&v.output.join("Life.json")).unwrap();
    library::validate(&saved).unwrap();
    assert_eq!(
        serde_json::to_value(&saved.sessions[0]).unwrap(),
        *v.expected.as_ref().unwrap()
    );
    library::copy(
        &v.output.join("Life.json"),
        &v.output.join("Life copy.json"),
    )
    .unwrap();
    models::workbench::experiment::export::write_bundle(
        &saved,
        &v.output.join("animation"),
        2,
        100,
    )
    .unwrap();
    world.resource_mut::<Files>().request = Some(Request::Load);
    v.phase = 12;
    v.wait = v.frames + 60;
}
