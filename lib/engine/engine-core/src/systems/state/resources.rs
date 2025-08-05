//! Long‑lived resources shared across the whole program.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};
use std::{fs, path::PathBuf};

/// Cross‑platform user‑specific config dir:  “~/Documents/SOUL/”
pub fn doc_dir() -> PathBuf {
    let mut p = dirs_next::document_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("SOUL");
    p
}

/// --------------------------------------------------------------------
/// NEW ‑ RuntimeFlags
/// --------------------------------------------------------------------
#[derive(Resource, Debug, Clone, Copy)]
pub struct RuntimeFlags {
    /// Is the GPU compute back‑end *allowed* this session?
    pub gpu_enabled: bool,
}

impl FromWorld for RuntimeFlags {
    fn from_world(world: &mut World) -> Self {
        let settings = world.get_resource::<Settings>();
        let force_cpu_env = std::env::var_os("SOUL_CPU").is_some();
        Self {
            gpu_enabled: settings.map_or(true, |s| s.gpu_compute) && !force_cpu_env,
        }
    }
}

/// Runtime counters.
#[derive(Resource, Debug, Default)]
pub struct Session {
    /// Number of frames rendered since start.
    pub frame: u64,
    /// Whether the simulation is currently paused.
    pub sim_paused: bool,
}

/// User / application preferences (saved as TOML).
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Master volume (0–1).  *Not used yet.*
    pub master_volume: f32,
    /// Base egui font size.
    pub ui_font_size: f32,

    /// Autosave enabled?
    pub autosave: bool,
    /// Autosave interval in **seconds**.
    pub autosave_interval: u64,

    // Enable GPU settings
    pub gpu_compute: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            ui_font_size: 16.0,
            autosave: true,
            autosave_interval: 30,
            gpu_compute: true,
        }
    }
}

impl Settings {
    const FILE: &'static str = "settings.toml";

    /// Try to read the previous settings; fall back to defaults.
    pub fn load() -> Self {
        let path = doc_dir().join(Self::FILE);
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Persist the settings to disk.
    pub fn save(&self) {
        let _ = fs::create_dir_all(Self::config_dir());
        let path = Settings::config_path();
        if let Err(e) = fs::write(&path, to_string_pretty(self).unwrap()) {
            eprintln!("Could not save settings: {e}");
        }
    }

    fn config_dir() -> PathBuf {
        dirs_next::document_dir()
            .unwrap_or(PathBuf::from("."))
            .join("SOUL")
    }

    fn config_path() -> PathBuf {
        Self::config_dir().join(Self::FILE)
    }
}
