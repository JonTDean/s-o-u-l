//! One-stop shop for registering every Bevy `Plugin`.
//!
//! Centralising the list means:
//! * A clean top-level `runner.rs` (no mile-long `.add_plugins()` chains).
//! * Conditional compilation or run-time feature flags happen in *one*
//!   place (e.g. network server vs client vs disabled).
//!
//! Down-stream code calls [`add_all_plugins()`] once; the order here
//! **must** match the architecture docs & Kanban cards.

use bevy::prelude::*;

use crate::{
    core::engine::plugin::EnginePlugin,
    input::plugin::InputPlugin,
    intelligence_engine::renderer::Renderer2DPlugin,
    state::StatePlugin,
    ui::components::{
        file_io::FileIoPlugin,
        menus::main_menu::MainMenuPlugin,
    },
};

/// Runtime flags that influence which plugins are added.
///
/// Right now only `networking` is used, but more (e.g. headless-only
/// export plugin) can be added without changing the public API.
pub struct PluginFlags<'a> {
    pub networking: &'a str, // "server" | "client" | "disabled"
}

/// Add **every** core & feature plugin in the correct order.
pub fn add_all_plugins(app: &mut App, flags: PluginFlags) {
    // ── 1. Dev / global utilities ───────────────────────────────────────
    app.add_plugins(StatePlugin);

    // ── 2. Input (player / network / scripted) ──────────────────────────
    app.add_plugins(InputPlugin);

    // ── 3. Core simulation engine (adds rule-set sub-plugins later) ─────
    app.add_plugins(EnginePlugin);

    // ── 4. (Optional) Networking layer  ─────────────────────────────────
    match flags.networking {
        "server" => { app.add_plugins(crate::network::server::ServerPlugin); },
        "client" => { app.add_plugins(crate::network::client::ClientPlugin); },
        _ => {}
    }

    // ── 5. UI & Rendering (runs in `MainSet::Render`) ───────────────────
    app.add_plugins((
        MainMenuPlugin,
        Renderer2DPlugin,
        FileIoPlugin,
    ));

    // Systems inside UI/renderer should live in `MainSet::Render`;
    // Renderer2DPlugin already tags its draw system appropriately, but
    // custom plugins can enforce it like:
    //     .add_systems(Update, my_system.in_set(MainSet::Render));
    
    // NOTE: Systems inside UI / renderer plugins should tag themselves with
    // `MainSet::Render` so they run after simulation logic.
}
