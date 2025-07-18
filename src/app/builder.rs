//! Declarative builder for the main Bevy [`App`].
//!
//! **Configuration precedence**  
//! `defaults  <  config-file  <  environment‒variables`
//!
//! * **Config file**:  
//!   *Location* – first hit wins:  
//!     1. `SOUL_CONFIG` env-var (explicit path)  
//!     2. `$XDG_CONFIG_HOME/soul/config.toml`  
//!     3. `./soul.toml` (current working directory)
//!   *Format* –
//! ```toml
//! headless   = false              # bool
//! grid_size  = 512                # u32
//! automaton  = "volume"           # "elementary" | "surface" | "volume"
//! networking = "server"           # "server" | "client" | "disabled"
//! ```
//!
//! * **Environment variables** (override individual keys)  
//!   `SOUL_HEADLESS`, `SOUL_GRID_SIZE`, `SOUL_AUTOMATON`, `SOUL_NETWORKING`
//!
//! Extend both the table and the [`RuntimeConfig`] struct whenever a Kanban
//! ticket introduces a new user-visible option.

use std::{env, fs, path::PathBuf};

use bevy::{
    prelude::*,
};
use serde::Deserialize;

/// Complete set of runtime parameters.
///
/// *Defaults* are hard-coded to ensure deterministic behaviour when no
/// external configuration is present.
#[derive(Debug)]
pub struct RuntimeConfig {
    pub headless: bool,
    pub grid_size: u32,
    pub automaton: String,
    pub networking: String,
}

/// **Optional** deserialisable layer used only for TOML parsing.
///
/// Any field omitted in the config-file simply falls back to the default.
#[derive(Debug, Deserialize)]
struct PartialConfig {
    headless: Option<bool>,
    grid_size: Option<u32>,
    automaton: Option<String>,
    networking: Option<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            headless: false,
            grid_size: 256,
            automaton: "surface".into(),
            networking: "disabled".into(),
        }
    }
}

impl RuntimeConfig {
    /* --------------------------------------------------------------------- */
    /* Public API                                                            */
    /* --------------------------------------------------------------------- */

    /// Build a configuration from *defaults*, *config-file* (if any),
    /// and finally *environment variables* in ascending precedence order.
    pub fn load() -> Self {
        Self::default()
            .merge(Self::from_file())
            .merge(Self::from_env())
    }

    /* --------------------------------------------------------------------- */
    /* Internal helpers                                                      */
    /* --------------------------------------------------------------------- */

    /// Attempt to read and parse the first config file found in the search
    /// order. Returns **partial** configuration – any missing field is `None`.
    fn from_file() -> Self {
        // 1. Explicit override via env-var
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
                        warn!("Malformed config ({path:?}): {e}; falling back.");
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
            if let Ok(n) = v.parse() {
                cfg.grid_size = n;
            } else {
                info!("SOUL_GRID_SIZE=\"{v}\" is not u32; keeping {}", cfg.grid_size);
            }
        }
        if let Ok(v) = env::var("SOUL_AUTOMATON") {
            cfg.automaton = v;
        }
        if let Ok(v) = env::var("SOUL_NETWORKING") {
            cfg.networking = v;
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
        self
    }
}
