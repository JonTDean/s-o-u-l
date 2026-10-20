use super::{flow::stage_check, verification::Verification};
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
    let stage = stage_check::stage_pixels(world).expect("the lab publishes a stage");
    let (output, file) = (
        verification.output.clone(),
        path.file_name().unwrap().to_string_lossy().into_owned(),
    );
    let mut save = save_to_disk(path.clone());
    world.spawn(Screenshot::primary_window()).observe(
        move |trigger: Trigger<ScreenshotCaptured>| {
            let row = stage_check::record(&output, &file, &trigger.event().0, stage);
            assert!(
                row["warm_pixels"].as_u64() > Some(0),
                "{file}: no specimen pixels in the stage: {row}"
            );
            assert_eq!(
                row["panels_clear"], true,
                "{file}: a panel overlaps the stage: {row}"
            );
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
