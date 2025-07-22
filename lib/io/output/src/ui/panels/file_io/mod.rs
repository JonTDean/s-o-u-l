//! src/ui/components/file_io/mod.rs
//! Manual Ctrl/⌘+S  +  first‑frame & periodic autosave.

use bevy::prelude::*;
use bevy::color::ColorToComponents;
use serde::{Deserialize, Serialize};
use std::{fs, time::Duration};
use engine_core::{
    core::world::World2D,
    engine::grid::GridBackend,
    state::{
        AppState, 
        resources::{
            Settings, 
            doc_dir
        }
    }
};

use crate::ui::panels::main_menu::controller::scenario::new::{ScenarioMeta, init_new_world};

/* ---------- on‑disk format ------------------------------------------------ */

#[derive(Serialize, Deserialize)]
struct SavedScenario {
    backend:   GridBackend,
    cell_size: f32,
    bg_color:  [f32; 4],   // linear RGBA floats
    params:    ScenarioMeta,
}

/* ---------- helpers ------------------------------------------------------- */

fn sanitize(name: &str) -> String {
    let mut s = name.trim()
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_', "_");
    if s.is_empty() { s = "Unnamed".into(); }
    s
}

fn write_snapshot(world: &World2D, meta: &ScenarioMeta, autosave: bool) {
    let dir   = doc_dir().join("saves");
    let base  = sanitize(&meta.0.name);
    let file  = dir.join(if autosave { format!("{base}-auto.json") } else { format!("{base}.json") });

    let payload = SavedScenario {
        backend:   world.backend.clone(),
        cell_size: world.cell_size,
        bg_color:  world.bg_color.to_linear().to_f32_array(),
        params:    meta.clone(),
    };

    match fs::create_dir_all(&dir)
        .and_then(|_| fs::write(&file, serde_json::to_vec_pretty(&payload)?))
    {
        Ok(_)  => println!("Saved → {}", file.display()),
        Err(e) => eprintln!("Save failed: {e}"),
    }
}

/* ---------- ECS resources & systems -------------------------------------- */

#[derive(Resource)] struct AutosaveTimer(Timer);

fn first_save(world: Res<World2D>, meta: Res<ScenarioMeta>) {
    // called once, right after we enter the game state
    write_snapshot(&world, &meta, false);
}

fn setup_autosave_timer(mut commands: Commands, set: Res<Settings>) {
    if set.autosave {
        commands.insert_resource(AutosaveTimer(Timer::new(
            Duration::from_secs(set.autosave_interval),
            TimerMode::Repeating,
        )));
    }
}

fn autosave_tick(
    time:   Res<Time>,
    timer:  Option<ResMut<AutosaveTimer>>,
    world:  Res<World2D>,
    meta:   Res<ScenarioMeta>,
) {
    let Some(mut t) = timer else { return };
    if t.0.tick(time.delta()).just_finished() {
        write_snapshot(&world, &meta, true);
    }
}

fn save_on_ctrl_s(
    keys: Res<ButtonInput<KeyCode>>,
    world: Res<World2D>,
    meta: Res<ScenarioMeta>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let cmd  = keys.pressed(KeyCode::SuperLeft)   || keys.pressed(KeyCode::SuperRight);
    if (ctrl || cmd) && keys.just_pressed(KeyCode::KeyS) {
        write_snapshot(&world, &meta, false);
    }
}

/* ---------- Bevy plugin --------------------------------------------------- */

pub struct FileIoPlugin;
impl Plugin for FileIoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                OnEnter(AppState::InGame),
                (
                    // run AFTER the world is created
                    first_save.after(init_new_world),
                    setup_autosave_timer,
                ),
            )
           .add_systems(
                Update,
                (save_on_ctrl_s, autosave_tick).run_if(in_state(AppState::InGame)),
            );
    }
}
