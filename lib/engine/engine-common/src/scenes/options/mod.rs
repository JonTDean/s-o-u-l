//! Options scene – font size, autosave & GPU settings.
//!
//! * Reads/writes persistent [`Settings`] and fast-changing [`RuntimeFlags`].
//! * UI lives in [`MainSet::Render`] to avoid race conditions with logic.

use bevy::prelude::*;
use engine_core::{
    prelude::AppState,
    systems::{
        schedule::MainSet,
        state::resources::{RuntimeFlags, Settings},
    },
};

/* ------------------------------------------------------------------ */
/* Local draft buffer                                                 */
/* ------------------------------------------------------------------ */

#[derive(Clone, Default)]
struct Draft {
    font_size:         f32,
    autosave:          bool,
    autosave_interval: u64,
    gpu_compute:       bool,
}

#[derive(Resource)]
struct OptionsScene { draft: Draft }

/* ------------------------------------------------------------------ */
/* Plug-in                                                            */
/* ------------------------------------------------------------------ */

/// Plugin for Options Menu
pub struct OptionsScenePlugin;

impl Plugin for OptionsScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                OnEnter(AppState::Options),
                |mut cmd: Commands, settings: Res<Settings>| {
                    cmd.insert_resource(OptionsScene {
                        draft: Draft {
                            font_size:         settings.ui_font_size,
                            autosave:          settings.autosave,
                            autosave_interval: settings.autosave_interval,
                            gpu_compute:       settings.gpu_compute,
                        },
                    });
                },
            );
    }
}