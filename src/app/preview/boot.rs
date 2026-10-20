//! Portable native boot gate: require live channels and a completed window capture.
use bevy::{
    diagnostic::FrameCount,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
};
use engine_core::automata::channel::Channels;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
#[derive(Resource)]
struct Boot {
    output: PathBuf,
    requested: bool,
    done: Arc<AtomicBool>,
    started: std::time::Instant,
}
pub(super) fn install(app: &mut App) {
    let Some(output) = std::env::var_os("SOUL_BOOT_VERIFY") else {
        return;
    };
    let output = PathBuf::from(output);
    std::fs::create_dir(&output).expect("fresh boot verification directory");
    app.insert_resource(Boot {
        output,
        requested: false,
        done: default(),
        started: std::time::Instant::now(),
    })
    .add_systems(Last, verify);
}
fn verify(
    mut commands: Commands,
    frames: Res<FrameCount>,
    channels: Res<Channels>,
    mut boot: ResMut<Boot>,
    mut exit: EventWriter<AppExit>,
) {
    assert!(
        boot.started.elapsed().as_secs() < 120,
        "native boot timed out"
    );
    if boot.done.load(Ordering::Acquire) {
        let screenshot = boot.output.join("boot.png");
        assert!(std::fs::metadata(screenshot).unwrap().len() > 1000);
        std::fs::write(
            boot.output.join("SUCCESS"),
            "Native window rendered; channels published; screenshot saved.\n",
        )
        .unwrap();
        exit.write(AppExit::Success);
        return;
    }
    if frames.0 < 120 || channels.rows.is_empty() || boot.requested {
        return;
    }
    boot.requested = true;
    let done = boot.done.clone();
    let mut save = save_to_disk(boot.output.join("boot.png"));
    commands.spawn(Screenshot::primary_window()).observe(
        move |trigger: Trigger<ScreenshotCaptured>| {
            save(trigger);
            done.store(true, Ordering::Release);
        },
    );
}
