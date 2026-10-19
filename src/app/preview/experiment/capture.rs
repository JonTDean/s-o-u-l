use super::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
pub(super) fn request(world: &mut World, v: &Verification) {
    let signal = v.captured.clone();
    let mut save = save_to_disk(v.output.join("experiment.png"));
    world.spawn(Screenshot::primary_window()).observe(
        move |trigger: Trigger<ScreenshotCaptured>| {
            save(trigger);
            signal.store(true, std::sync::atomic::Ordering::Release);
        },
    );
}
