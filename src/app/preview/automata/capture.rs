use super::verification::Verification;
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(super) fn request(world: &mut World, verification: &mut Verification, name: &str) {
    let path = verification
        .output
        .join(format!("{}.png", name.replace(':', "-")));
    if path.exists() {
        std::fs::remove_file(&path).expect("remove stale screenshot");
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

pub(super) fn finished(verification: &Verification) -> bool {
    let Some((path, signal)) = &verification.capture else {
        return false;
    };
    if !signal.load(Ordering::Acquire) {
        return false;
    }
    assert!(std::fs::metadata(path).expect("saved screenshot").len() > 1000);
    true
}
