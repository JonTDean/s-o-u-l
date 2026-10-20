use super::*;
use crate::app::preview::automata::flow::stage_check;
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
    let stage = stage_check::stage_pixels(world).expect("the lab publishes a stage");
    let (output, file) = (
        verification.output.clone(),
        path.file_name().unwrap().to_string_lossy().into_owned(),
    );
    // Phase 8 is captured after Clear: the stage must hold no specimen pixels.
    let populated = verification.phase != 8;
    let mut save = save_to_disk(path.clone());
    world.spawn(Screenshot::primary_window()).observe(
        move |trigger: Trigger<ScreenshotCaptured>| {
            let row = stage_check::record(&output, &file, &trigger.event().0, stage);
            let warm = row["warm_pixels"].as_u64().unwrap_or(0);
            assert_eq!(
                warm > 0,
                populated,
                "{file}: warm stage pixels {warm} (populated: {populated})"
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
