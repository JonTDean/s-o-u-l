use super::*;
use engine_core::automata::channel::Channels;
use models::workbench::{catalog::Kind, session::Session};
use simulation_kernel::core::{
    boundary::Boundary,
    lenia::continuous::{Kernel, Shape},
};
use ui::lab::{
    chrome::drawers::Comparison,
    state::{Drawer, LabState},
};
pub(super) fn configure(world: &mut World, owners: &[(Entity, ChannelId, LeniaSimulation)]) {
    let mut bench = world.resource_mut::<Workbench>();
    for s in &mut bench.sessions {
        let id = s.channel;
        *s = Session::new(Kind::Life);
        s.channel = id;
        let mut settings = s.settings.clone();
        settings.world.width = 32;
        settings.world.height = 16;
        settings.world.boundary = Boundary::Periodic;
        settings.world.seed = Some(vec![0; 512]);
        s.configure(settings, true).unwrap();
        s.paint(&[[0, 0], [1, 0], [2, 0]], 255, true).unwrap();
        s.undo_paint(false).unwrap();
        s.undo_paint(true).unwrap();
    }
    bench.sessions[1].paint(&[[0, 0]], 0, true).unwrap();
    for (i, (entity, _, _)) in owners.iter().enumerate() {
        let mut s = world.get_mut::<LeniaSimulation>(*entity).unwrap();
        s.kernel = Some(Kernel {
            shape: if i == 0 {
                Shape::Gaussian
            } else {
                Shape::Shell
            },
            ..Default::default()
        });
        s.paused = true;
        s.reset();
        s.target_tick = 3;
    }
    let mut lab = world.resource_mut::<LabState>();
    lab.drawer = Some(Drawer::Compare);
}
pub(super) fn select(world: &mut World, ids: [ChannelId; 2]) {
    world.resource_mut::<Channels>().selected = Some(ids[0]);
    let mut c = world.resource_mut::<Comparison>();
    c.left = Some(ids[0]);
    c.right = Some(ids[1]);
    c.capture.result = None;
    c.capture.request = Some(true);
}
pub(super) fn capture(world: &mut World, v: &mut Verification, name: &str) {
    use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
    v.captured
        .store(false, std::sync::atomic::Ordering::Release);
    let signal = v.captured.clone();
    let mut save = save_to_disk(v.output.join(name));
    world.spawn(Screenshot::primary_window()).observe(
        move |trigger: Trigger<ScreenshotCaptured>| {
            save(trigger);
            signal.store(true, std::sync::atomic::Ordering::Release);
        },
    );
}
