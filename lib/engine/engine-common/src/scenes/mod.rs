//! Scene manager – orchestrates `AppState` ↔ camera activation.
//!
//! ## What it does
//! * **Toggles the world camera** on/off as you move between scenes so the
//!   main menu never renders the (expensive) 3-D world, while the in-game,
//!   editor and pause overlays always do.
//! * **Listens for the **`Escape` **key** in `AppState::InGame` and pushes a
//!   lightweight `PauseMenu` overlay where the player can:
//!   * _Resume_ the simulation,
//!   * open the global _Options_ screen,
//!   * jump into the _Editor_,
//!   * or return to the _Main Menu_.
//! * Exposes a tiny one-frame [`PauseAction`] channel so your egui pause-menu
//!   UI can route button clicks without tight coupling.
//!
//! ## How to use
//! ```rust
//! use engine_common::{controls::camera::CameraPluginBundle, scenes::SceneManagerPlugin};
//!
//! App::new()
//!     .add_plugins(CameraPluginBundle)      // already in your tree
//!     .add_plugins(SceneManagerPlugin)      // ← NEW: drop it in
//!     .run();
//! ```
//!
//! Make sure the `AppState` enum in **engine-core** includes the extra
//! variants used below (`PauseMenu` & `Editor`).  They can be inserted
//! anywhere in the enum—ordering is irrelevant for Bevy’s state machine.

use bevy::prelude::*;
use engine_core::prelude::AppState;

use crate::controls::camera::manager::WorldCamera;  // ← path fixed

pub mod main_menu;
pub mod options;

pub use main_menu::MainMenuPlugin;
pub use options::OptionsScenePlugin;

/* =================================================================== */
/* Plug-in                                                             */
/* =================================================================== */

/// Top-level scene router – installs concrete scenes and handles
/// world-camera visibility & pause logic.
pub struct SceneManagerPlugin;


impl Plugin for SceneManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MainMenuPlugin, OptionsScenePlugin))
            .add_systems(OnEnter(AppState::MainMenu), hide_world_camera)
            .add_systems(OnEnter(AppState::NewScenario), hide_world_camera)
            .add_systems(OnEnter(AppState::LoadScenario), hide_world_camera)
            .add_systems(OnEnter(AppState::Options), hide_world_camera)
            .add_systems(OnEnter(AppState::InGame),  show_world_camera)
            .add_systems(OnEnter(AppState::Paused),  show_world_camera)
            .add_systems(OnExit(AppState::InGame),   hide_world_camera)
            .add_systems(OnExit(AppState::Paused),   hide_world_camera)
            .add_systems(
                Update,
                (
                    in_game_esc_to_pause.run_if(in_state(AppState::InGame)),
                    pause_menu_router.run_if(in_state(AppState::Paused)),
                ),
            );
    }
}

/* =================================================================== */
/* Camera helpers                                                      */
/* =================================================================== */

fn show_world_camera(mut q: Query<(&mut Camera, &mut Visibility), With<WorldCamera>>) {
    if let Ok((mut cam, mut vis)) = q.single_mut() {
        cam.is_active = true;
        *vis          = Visibility::Inherited;
    }
}

fn hide_world_camera(mut q: Query<(&mut Camera, &mut Visibility), With<WorldCamera>>) {
    if let Ok((mut cam, mut vis)) = q.single_mut() {
        cam.is_active = false;
        *vis          = Visibility::Hidden;
    }
}

/* =================================================================== */
/* Pause handling                                                      */
/* =================================================================== */

fn in_game_esc_to_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(AppState::Paused);
    }
}

fn pause_menu_router(
    mut cmd:  Commands,
    mut next: ResMut<NextState<AppState>>,
    action:   Option<Res<PauseAction>>,
) {
    let Some(act) = action else { return };
    match *act {
        PauseAction::Resume   => next.set(AppState::InGame),
        PauseAction::Options  => next.set(AppState::Options),
        PauseAction::MainMenu => next.set(AppState::MainMenu),
    }
    cmd.remove_resource::<PauseAction>();
}

/* =================================================================== */
/* UI → router bridge                                                  */
/* =================================================================== */

/// One-shot command resource emitted by the pause overlay.
#[derive(Resource, Clone, Copy, Debug, Eq, PartialEq)]
pub enum PauseAction {
    /// Return to the running simulation.
    Resume,
    /// Open the global *Options* screen.
    Options,
    /// Jump back to the *Main Menu*.
    MainMenu,
}

impl PauseAction {
    /// Inject the given action as a **one-frame** resource so
    /// `pause_menu_router` can react to it the next frame.
    pub fn send(cmd: &mut Commands, act: Self) {
        cmd.insert_resource(act);
    }
}