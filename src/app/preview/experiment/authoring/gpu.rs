use super::*;
use engine_core::events::AutomataCommand;
pub(super) fn run(world: &mut World, v: &mut Verification) {
    match v.phase {
        4 => {
            let id = world.resource::<Workbench>().sessions[0].channel;
            world.resource_mut::<Channels>().selected = Some(id);
            world.resource_mut::<Workbench>().sessions[0].single_step();
            v.phase = 5;
            v.wait = v.frames + 30;
        }
        5 => {
            let count = world
                .query_filtered::<Entity, With<ui::lab::stage::voxels::SymbolVoxel>>()
                .iter(world)
                .count();
            assert!(count >= 8, "tape, head and stack are voxel entities");
            capture(world, v, "symbolic-voxels.png");
            v.phase = 6;
        }
        6 if v.captured.load(std::sync::atomic::Ordering::Acquire) => {
            world.send_event(AutomataCommand::RestoreLenia(Box::new(
                library::orbium::specimen(),
            )));
            world.resource_mut::<LabState>().guide.active = None;
            v.phase = 7;
            v.wait = v.frames + 60;
        }
        7 => {
            let Some((_, id)) = world
                .query::<(&LeniaSimulation, &ChannelId)>()
                .iter(world)
                .next()
            else {
                return;
            };
            let id = *id;
            world.resource_mut::<Channels>().selected = Some(id);
            world.resource_mut::<LabState>().drawer = None;
            let mut files = world.resource_mut::<Files>();
            files.path = v.output.join("Orbium.json").to_string_lossy().into();
            files.request = Some(Request::Specimen(id));
            v.phase = 8;
        }
        8 if v.output.join("Orbium.json").exists() => {
            let saved = project::read(&v.output.join("Orbium.json")).unwrap();
            library::validate(&saved).unwrap();
            assert_eq!(
                saved.lenia[0].continuous.as_ref().unwrap().values,
                library::orbium::specimen().continuous.unwrap().values
            );
            library::copy(
                &v.output.join("Orbium.json"),
                &v.output.join("Orbium copy.json"),
            )
            .unwrap();
            models::workbench::experiment::export::write_bundle(
                &saved,
                &v.output.join("orbium-animation"),
                2,
                100,
            )
            .unwrap();
            capture(world, v, "published-orbium.png");
            v.phase = 9;
        }
        9 if v.captured.load(std::sync::atomic::Ordering::Acquire) => {
            let (owner, mut sim) = world
                .query::<(Entity, &mut LeniaSimulation)>()
                .iter_mut(world)
                .next()
                .unwrap();
            sim.target_tick = 10;
            world.resource::<engine_gpu::LeniaProbe>().request(owner);
            v.phase = 10;
        }
        10 => {
            let owner = world
                .query_filtered::<Entity, With<LeniaSimulation>>()
                .iter(world)
                .next()
                .unwrap();
            let probe = world.resource::<engine_gpu::LeniaProbe>();
            let Some(Ok(snapshot)) = probe.latest(owner) else {
                probe.request(owner);
                return;
            };
            if snapshot.tick != 10 {
                probe.request(owner);
                return;
            }
            let values = snapshot.continuous.unwrap().values;
            let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/specimens/orbium-reference.json"
            )))
            .unwrap();
            let error = values[4 * 4096..5 * 4096]
                .iter()
                .zip(fixture["states"]["10"].as_array().unwrap())
                .map(|(a, b)| (*a as f64 - b.as_f64().unwrap()).abs())
                .fold(0.0, f64::max);
            assert!(error < 0.002, "native Orbium reference error {error}");
            std::fs::write(
                v.output.join("orbium-error.txt"),
                format!("tick 10 max FFT error {error}\n"),
            )
            .unwrap();
            std::fs::write(v.output.join("SUCCESS"), "Native scenarios, completed comparison CSV, exact CPU/GPU specimens, duplicates, animations, voxel entities and published Orbium passed.\n").unwrap();
            world.send_event(AppExit::Success);
            v.phase = 11;
        }
        _ => {}
    }
}
