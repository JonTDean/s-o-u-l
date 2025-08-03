//! engine-core/src/plugin.rs
//! ---------------------------------------------------------------------------
//! Core ECS resources and baseline behaviour for the engine.
//!
//!  * Registers every long-lived resource that **any** downstream plugin
//!    might depend on **before** those plugins are added.
//!  * Keeps startup deterministic and thread-safe: all resources here are
//!    `Send + Sync`, so Bevy’s parallel scheduler can access them freely.
//!
//! © 2025 Obaven Inc.  Released under the MIT Licence.

use bevy::app::{App, Plugin};

use crate::prelude::{
    AutomataRegistry,
    RuleRegistry,
};

/// Adds global registries and other cross-cutting resources.
///
/// Insert this plugin **before** any plugin that needs access to the
/// registries (e.g. render, debugging, AI).
pub struct EngineCorePlugin;

impl Plugin for EngineCorePlugin {
    fn build(&self, app: &mut App) {
        app
            // ───────────────────────────────────────────────────────────────
            // Global registries
            // ───────────────────────────────────────────────────────────────
            //
            // `init_resource` creates the resource only if the user (or a
            // test-harness) hasn’t inserted a customised instance already.
            //
            // Both registries derive `Default`, so construction is free of
            // panics and allocation-free until something is registered.
            .init_resource::<AutomataRegistry>()
            .init_resource::<RuleRegistry>();
    }
}
