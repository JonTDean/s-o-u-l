use super::*;
use engine_core::automata::channel::{Channels, metrics::Metrics};
use models::workbench::{experiment::export::write_bundle, project};
use ui::lab::chrome::drawers::{Comparison, Files, Request};
pub(super) fn compare(world: &World, v: &Verification, restored: bool) -> bool {
    let c = world.resource::<Comparison>();
    assert!(
        !c.status.contains("cancelled") && !c.status.contains("timed out"),
        "{}",
        c.status
    );
    let Some(pair) = &c.capture.result else {
        return false;
    };
    for frame in pair {
        let kernel = frame.stamp.kernel.unwrap();
        let params = frame.stamp.params.unwrap();
        let mut expected = if restored {
            v.saved
                .iter()
                .find(|s| s.continuous.as_ref().unwrap().kernel == kernel)
                .unwrap()
                .continuous
                .as_ref()
                .unwrap()
                .values
                .clone()
        } else {
            simulation_kernel::core::lenia::seed(frame.field.size, Default::default())
                .iter()
                .map(|v| *v as f32 / 255.0)
                .collect()
        };
        for _ in 0..if restored { 1 } else { 4 } {
            expected = simulation_kernel::core::lenia::continuous::step(
                &expected,
                frame.field.size,
                kernel,
                params,
            )
            .unwrap();
        }
        let error = expected
            .iter()
            .zip(&frame.field.values)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max);
        assert!(error < 0.00001, "native GPU max error {error}");
        assert_eq!(frame.stamp.tick, if restored { 5 } else { 4 });
    }
    assert!(pair[0].field.compare(&pair[1].field).unwrap().maximum > 0.0);
    true
}
pub(super) fn csv(world: &mut World, v: &Verification, name: &str) {
    world.resource_mut::<Comparison>().folder = v.output.join(name).to_string_lossy().into_owned();
    ui::lab::chrome::drawers::compare::export::write(
        world.resource::<Comparison>(),
        world.resource::<Channels>(),
        world.resource::<Metrics>(),
    )
    .unwrap();
}
pub(super) fn save(world: &mut World, v: &Verification) {
    let mut files = world.resource_mut::<Files>();
    files.path = v.output.join("project.json").to_string_lossy().into_owned();
    files.request = Some(Request::Save);
}
pub(super) fn load(world: &mut World, v: &mut Verification) -> bool {
    let files = world.resource::<Files>();
    if files.request.is_some() || files.save.is_some() {
        return false;
    }
    assert!(files.status.starts_with("Saved"), "{}", files.status);
    let project = project::read(std::path::Path::new(&files.path)).unwrap();
    assert_eq!(project.sessions.len(), 2);
    assert_eq!(project.lenia.len(), 2);
    for s in &project.sessions {
        assert_eq!(s.settings.world.width, 32);
        assert!(s.settings.world.seed.is_some());
    }
    let c = world.resource::<Comparison>();
    for frame in c.capture.result.as_ref().unwrap() {
        let saved = project
            .lenia
            .iter()
            .find(|s| s.continuous.as_ref().unwrap().kernel == frame.stamp.kernel.unwrap())
            .unwrap();
        assert_eq!(
            saved.continuous.as_ref().unwrap().values,
            frame.field.values
        );
    }
    write_bundle(&project, &v.output.join("animation"), 2, 100).unwrap();
    v.saved = project.lenia;
    world.resource_mut::<Files>().request = Some(Request::Load);
    true
}

pub(super) fn mass(world: &World) {
    let c = world.resource::<Comparison>();
    let metrics = world.resource::<Metrics>();
    for frame in c.capture.result.as_ref().unwrap() {
        let sample = metrics
            .get(frame.stamp.id)
            .unwrap()
            .latest
            .expect("opt-in GPU mass sample");
        assert_eq!(
            (sample.epoch, sample.tick),
            (frame.stamp.epoch, frame.stamp.tick)
        );
        let expected: f64 = frame.field.values.iter().map(|v| *v as f64).sum();
        assert!(
            (sample.value - expected).abs() < 1e-9,
            "mass must retain float density precision"
        );
    }
}
