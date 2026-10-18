use super::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use std::sync::atomic::Ordering;

pub(super) fn request(world: &mut World, verification: &mut Verification) {
    let path = verification
        .output
        .join(format!("lenia-phase-{}.png", verification.phase));
    if path.exists() {
        std::fs::remove_file(&path).expect("remove previous capture");
    }
    let finished = Arc::new(AtomicBool::new(false));
    let signal = finished.clone();
    let mut save = save_to_disk(path.clone());
    world.spawn(Screenshot::primary_window()).observe(
        move |trigger: Trigger<ScreenshotCaptured>| {
            save(trigger);
            signal.store(true, Ordering::Release);
        },
    );
    verification.capture = Some((path, finished));
}

pub(super) fn finished(verification: &mut Verification) -> bool {
    let Some((path, signal)) = &verification.capture else {
        return true;
    };
    if !signal.load(Ordering::Acquire) {
        return false;
    }
    assert!(
        std::fs::metadata(path)
            .expect("Lenia screenshot saved")
            .len()
            > 1_000
    );
    verification.capture = None;
    true
}
