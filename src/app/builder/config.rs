//! Runtime configuration loader.
//!
//! This file was extracted from the original `builder.rs` so that **all**
//! configuration concerns live in a single, test‑friendly unit (`config.rs`),
//! while public access goes through `builder::RuntimeConfig`.
//!
//! See `builder/mod.rs` for a high‑level overview and directory layout.

use std::{env, fs, path::PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

/* ====================================================================== */
/* Public data‑structures                                                 */
/* ====================================================================== */

/// Complete set of runtime parameters.
///
/// *Defaults* are hard‑coded to ensure deterministic behaviour when no
/// external configuration is present.
#[derive(Debug)]
pub struct RuntimeConfig {
    pub headless: bool,
    pub grid_size: u32,
    pub automaton: String,
    pub networking: String,
    /// Target fixed‑step frequency for the simulation loop (Hz).
    pub simulation_rate_hz: u32,
    /// Maximum number of fixed‑step iterations allowed in a **single** frame.
    pub max_sim_steps_per_frame: u8,
}

/// **Optional** deserialisable layer used only for TOML parsing.
///
/// Any field omitted in the config‑file simply falls back to the default.
#[derive(Debug, Deserialize)]
struct PartialConfig {
    headless: Option<bool>,
    grid_size: Option<u32>,
    automaton: Option<String>,
    networking: Option<String>,
    simulation_rate: Option<u32>,
    max_steps_per_frame: Option<u8>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            headless: false,
            grid_size: 256,
            automaton: "surface".into(),
            networking: "disabled".into(),
            simulation_rate_hz: 60,
            max_sim_steps_per_frame: 3,
        }
    }
}

/* ====================================================================== */
/* Public API                                                             */
/* ====================================================================== */

impl RuntimeConfig {
    /// Build a configuration from *defaults*, *config‑file* (if any), and
    /// finally *environment variables* in ascending precedence order.
    pub fn load() -> Self {
        Self::default()
            .merge(Self::from_file())
            .merge(Self::from_env())
    }

    /* ------------------------------------------------------------------ */
    /* Private helpers                                                    */
    /* ------------------------------------------------------------------ */

    /// Attempt to read and parse the first config file found in the search
    /// order. Returns **partial** configuration – any missing field is `None`.
    fn from_file() -> Self {
        // 1. Explicit override via env‑var
        let candidates: [Option<PathBuf>; 3] = [
            env::var_os("SOUL_CONFIG").map(PathBuf::from),
            env::var_os("XDG_CONFIG_HOME")
                .map(|d| PathBuf::from(d).join("soul").join("config.toml")),
            Some(PathBuf::from("soul.toml")),
        ];

        for path in candidates.into_iter().flatten() {
            match fs::read_to_string(&path) {
                Ok(toml) => match toml::from_str::<PartialConfig>(&toml) {
                    Ok(p) => return Self::default().merge_partial(p),
                    Err(e) => {
                        warn!(
                            "Malformed config ({path:?}): {e}; falling back to defaults."
                        );
                        break;
                    }
                },
                Err(_) => continue, // File not found – try next candidate.
            }
        }

        Self::default() // No file → defaults only.
    }

    /// Read overrides from environment variables.
    fn from_env() -> Self {
        let mut cfg = Self::default();

        if let Ok(v) = env::var("SOUL_HEADLESS") {
            cfg.headless = matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes");
        }
        if let Ok(v) = env::var("SOUL_GRID_SIZE") {
            match v.parse() {
                Ok(n) => cfg.grid_size = n,
                Err(_) => info!(
                    "SOUL_GRID_SIZE=\"{v}\" is not u32; keeping {}",
                    cfg.grid_size
                ),
            }
        }
        if let Ok(v) = env::var("SOUL_AUTOMATON") {
            cfg.automaton = v;
        }
        if let Ok(v) = env::var("SOUL_NETWORKING") {
            cfg.networking = v;
        }
        if let Ok(v) = env::var("SOUL_SIM_RATE") {
            match v.parse() {
                Ok(n) if n > 0 => cfg.simulation_rate_hz = n,
                _ => info!(
                    "SOUL_SIM_RATE=\"{v}\" is not a positive integer; keeping {}",
                    cfg.simulation_rate_hz
                ),
            }
        }
        if let Ok(v) = env::var("SOUL_MAX_STEPS") {
            match v.parse() {
                Ok(n) if n > 0 => cfg.max_sim_steps_per_frame = n,
                _ => info!(
                    "SOUL_MAX_STEPS=\"{v}\" is not a positive integer; keeping {}",
                    cfg.max_sim_steps_per_frame
                ),
            }
        }

        cfg
    }

    /// Merge two *complete* configurations – all fields in `other` win.
    #[inline]
    fn merge(mut self, other: Self) -> Self {
        self.headless = other.headless;
        self.grid_size = other.grid_size;
        self.automaton = other.automaton;
        self.networking = other.networking;
        self.simulation_rate_hz = other.simulation_rate_hz;
        self.max_sim_steps_per_frame = other.max_sim_steps_per_frame;
        self
    }

    /// Overlay a `PartialConfig` onto an existing full config.
    #[inline]
    fn merge_partial(mut self, other: PartialConfig) -> Self {
        if let Some(v) = other.headless {
            self.headless = v;
        }
        if let Some(v) = other.grid_size {
            self.grid_size = v;
        }
        if let Some(v) = other.automaton {
            self.automaton = v;
        }
        if let Some(v) = other.networking {
            self.networking = v;
        }
        if let Some(v) = other.simulation_rate {
            if v > 0 {
                self.simulation_rate_hz = v;
            }
        }
        if let Some(v) = other.max_steps_per_frame {
            if v > 0 {
                self.max_sim_steps_per_frame = v;
            }
        }
        self
    }
}
