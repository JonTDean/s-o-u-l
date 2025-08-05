//! Scene manager – orchestrates `AppState` ↔ camera activation.
//!
//! This top-level router plugs in the concrete scene plug-ins for the *main*
//! menu, *options* menu, and the *scenario* flows (new / load).  It also takes
//! care of toggling the expensive 3-D world camera and of the in-game *Escape*
//! key → pause overlay logic.  Everything is strictly data-driven, so the
//! entire module remains deterministic and therefore amenable to headless CI
//! test builds.

use bevy::prelude::*;
use engine_core::prelude::AppState;

use crate::controls::camera::manager::WorldCamera;

pub mod main_menu;
pub mod options;
pub mod scenarios;

pub use main_menu::MainMenuPlugin;
pub use options::OptionsScenePlugin;
pub use scenarios::ScenariosPlugin;

/* =================================================================== */
/* Plug-in                                                             */
/* =================================================================== */

/// Top-level scene router – install once and forget forever.
///
/// The plug-in is intentionally *thin*: every concrete scene lives in its own
/// crate-private module so that dependencies stay minimal and compile times
/// remain snappy.
pub struct SceneManagerPlugin;

impl Plugin for SceneManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
                MainMenuPlugin,
                OptionsScenePlugin,
                ScenariosPlugin,
            ))
            /* ------------------------ camera flips ------------------------ */
            .add_systems(OnEnter(AppState::MainMenu),    hide_world_camera)
            .add_systems(OnEnter(AppState::NewScenario), hide_world_camera)
            .add_systems(OnEnter(AppState::LoadScenario),hide_world_camera)
            .add_systems(OnEnter(AppState::Options),     hide_world_camera)
            .add_systems(OnEnter(AppState::InGame),      show_world_camera)
            .add_systems(OnEnter(AppState::Paused),      show_world_camera)
            .add_systems(OnExit(AppState::InGame),       hide_world_camera)
            .add_systems(OnExit(AppState::Paused),       hide_world_camera)
            /* ------------------------- routers --------------------------- */
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
    /// Inject `Self` as a **one-frame** resource so that
    /// `pause_menu_router` can react to it on the next frame.
    pub fn send(cmd: &mut Commands, act: Self) {
        cmd.insert_resource(act);
    }
}
