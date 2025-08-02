//! lib/tests/src/util.rs
//! -----------------------------------------------------------
//! Minimal helpers so the placeholder tests build.
//!
//! Replace these with proper implementations once the
//! hash-world / deterministic-runner utilities land.

use bevy::prelude::*;

/// Bare-bones Bevy app good enough for the sim-loop unit-tests.
pub fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);        // no window/audio needed
    app
}

/// One frame advance (variable-rate).
pub fn step_once(app: &mut App) {
    app.update();
}

/// Dummy world hash (always 0) – replace with a real hash later.
pub fn hash_world(_: &World) -> u64 {
    0
}
