use super::*;
use models::workbench::experiment::{config::authored::storage, export};
pub(super) fn run(world: &mut World, v: &mut Verification) {
    match v.phase {
        4 => {
            let saved = project::read(&v.output.join("project.json")).unwrap();
            assert_eq!(
                serde_json::to_value(&saved.sessions).unwrap(),
                *v.expected.as_ref().unwrap()
            );
            let program = saved.sessions[2].settings.program.as_ref().unwrap();
            let path = v.output.join("rules.json");
            storage::write(&path, program).unwrap();
            assert_eq!(storage::read(&path).unwrap(), *program);
            export::write_bundle(&saved, &v.output.join("animation"), 3, 100).unwrap();
            let mut single = project::read(&v.output.join("project.json")).unwrap();
            single.sessions = vec![single.sessions.remove(2)];
            single.selected = 0;
            library::create(&v.output.join("Energy specimen.json"), &single).unwrap();
            library::copy(
                &v.output.join("Energy specimen.json"),
                &v.output.join("Energy copy.json"),
            )
            .unwrap();
            let compare = world.resource::<Comparison>();
            let pair = compare
                .capture
                .result
                .as_ref()
                .expect("completed comparison");
            assert_eq!((pair[0].stamp.tick, pair[1].stamp.tick), (4, 4));
            assert!(pair[0].field.compare(&pair[1].field).unwrap().changed > 0);
            world.resource_mut::<Comparison>().folder =
                v.output.join("comparison").to_string_lossy().into();
            ui::lab::chrome::drawers::compare::export::write(
                world.resource::<Comparison>(),
                world.resource::<Channels>(),
                world.resource::<engine_core::automata::channel::metrics::Metrics>(),
            )
            .unwrap();
            capture(world, v, "energy.png");
            v.phase = 5;
        }
        5 if v.captured.load(std::sync::atomic::Ordering::Acquire) => {
            let id = world.resource::<Workbench>().sessions[1].channel;
            world.resource_mut::<Channels>().selected = Some(id);
            v.phase = 6;
            v.wait = v.frames + 60;
        }
        6 => {
            capture(world, v, "composition.png");
            v.phase = 7;
        }
        7 if v.captured.load(std::sync::atomic::Ordering::Acquire) => {
            world.resource_mut::<Files>().request = Some(Request::Load);
            v.phase = 8;
            v.wait = v.frames + 60;
        }
        8 => {
            let bench = world.resource::<Workbench>();
            assert_eq!(
                serde_json::to_value(&bench.sessions).unwrap(),
                *v.expected.as_ref().unwrap()
            );
            let mut expected = project::Project::capture(bench, vec![]).unwrap();
            let ids: Vec<_> = bench.sessions.iter().map(|s| s.channel).collect();
            for session in &mut expected.sessions {
                session.step();
            }
            v.expected = Some(serde_json::to_value(&expected.sessions).unwrap());
            for id in ids {
                world.send_event(engine_core::automata::channel::ChannelCommand {
                    id,
                    action: engine_core::automata::channel::ChannelAction::Step,
                });
            }
            v.phase = 9;
            v.wait = v.frames + 30;
        }
        9 => {
            assert_eq!(
                serde_json::to_value(&world.resource::<Workbench>().sessions).unwrap(),
                *v.expected.as_ref().unwrap()
            );
            for name in ["energy.png", "composition.png"] {
                assert!(std::fs::metadata(v.output.join(name)).unwrap().len() > 1000);
            }
            std::fs::write(v.output.join("SUCCESS"), "Authored and composed energy channels: configure, four steps, comparison, rule interchange, project restore and continuation, specimens, PNG/GIF and native captures passed.\n").unwrap();
            world.send_event(AppExit::Success);
            v.phase = 10;
        }
        _ => {}
    }
}
