use super::*;
use engine_core::{
    automata::channel::{ChannelAction, ChannelCommand},
    events::AutomataCommand,
};
use models::workbench::operations::channels::ConfigureSession;
pub(super) fn update(world: &mut World) {
    world.resource_scope(|world, mut v: Mut<Verification>| {
        v.frames += 1;
        assert!(v.frames < 6000, "rule verification timeout at {}", v.phase);
        if v.frames < v.wait {
            return;
        }
        match v.phase {
            0 if v.frames > 60 => {
                world.send_event(AutomataCommand::Clear);
                world.resource_mut::<Files>().load = Some(fixtures::project());
                world.resource_mut::<LabState>().drawer = None;
                v.phase = 1;
                v.wait = v.frames + 60;
            }
            1 => {
                let ids: Vec<_> = world
                    .resource::<Workbench>()
                    .sessions
                    .iter()
                    .map(|s| s.channel)
                    .collect();
                assert_eq!(ids.len(), 3);
                for (i, id) in ids.iter().enumerate() {
                    world.send_event(ConfigureSession {
                        id: *id,
                        settings: Box::new(fixtures::settings(i)),
                        reset: true,
                    });
                }
                world.resource_mut::<Channels>().selected = Some(ids[2]);
                world.resource_mut::<LabState>().inspector_width = 420.0;
                v.phase = 2;
                v.wait = v.frames + 30;
            }
            2 => {
                let mut expected =
                    project::Project::capture(world.resource::<Workbench>(), vec![]).unwrap();
                for (i, s) in expected.sessions.iter_mut().enumerate() {
                    assert_eq!(s.settings, fixtures::settings(i));
                    for _ in 0..4 {
                        s.step();
                    }
                }
                v.expected = Some(serde_json::to_value(&expected.sessions).unwrap());
                let ids: Vec<_> = world
                    .resource::<Workbench>()
                    .sessions
                    .iter()
                    .map(|s| s.channel)
                    .collect();
                for id in ids {
                    for _ in 0..4 {
                        world.send_event(ChannelCommand {
                            id,
                            action: ChannelAction::Step,
                        });
                    }
                }
                v.phase = 3;
                v.wait = v.frames + 30;
            }
            3 => {
                let bench = world.resource::<Workbench>();
                assert_eq!(
                    serde_json::to_value(&bench.sessions).unwrap(),
                    *v.expected.as_ref().unwrap()
                );
                let ids: Vec<_> = bench.sessions.iter().map(|s| s.channel).collect();
                let mut compare = world.resource_mut::<Comparison>();
                compare.left = Some(ids[0]);
                compare.right = Some(ids[2]);
                compare.capture.request = Some(false);
                let mut files = world.resource_mut::<Files>();
                files.path = v.output.join("project.json").to_string_lossy().into();
                files.request = Some(Request::Save);
                v.phase = 4;
                v.wait = v.frames + 30;
            }
            _ => super::exchange::run(world, &mut v),
        }
    });
}
