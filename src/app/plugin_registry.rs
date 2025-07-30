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
use bevy_egui::EguiPlugin;
use models::plugin::ComputationalIntelligencePlugin;
use engine_core::{plugin::EngineCorePlugin, systems::state::StatePlugin};
use engine_render::plugin::EngineRendererPlugin;
use ui::plugin::OutputPlugin;

// /// Runtime flags that influence which plugins are added.
// ///
// /// Right now only `networking` is used, but more (e.g. headless-only
// /// export plugin) can be added without changing the public API.
// pub struct PluginFlags<'a> {
//     pub networking: &'a str, // "server" | "client" | "disabled"
// }

/// Add **every** core & feature plugin in the correct order.
pub fn add_all_plugins(app: &mut App) {
    /* 0 egui framework */ 
    app.add_plugins(EguiPlugin::default());     //  enables egui + registers the
                                                //      EguiPrimaryContextPass schedule
                                     
    /* 1 dev utilities    */
    app.add_plugins(StatePlugin);

    /* 2 core engine      */
    app.add_plugins(EngineCorePlugin);

    /* 3 *render* layer   */
    app.add_plugins(EngineRendererPlugin);

    /* 4 C.I. layer       */
    app.add_plugins(ComputationalIntelligencePlugin);

    // // ── 4. Networking layer  ─────────────────────────────────
    // match flags.networking {
    //     "server" => { app.add_plugins(input::network::server::ServerPlugin); }
    //     "client" => { app.add_plugins(input::network::client::ClientPlugin); }
    //     _ => { /* networking disabled */ }
    // }

    /* 5 UI / HUD         */
    app.add_plugins((
        OutputPlugin,       // menus, HUD, active-grid renderer
    ));

    // Systems inside UI/renderer should live in `MainSet::Render`;
    // Renderer2DPlugin already tags its draw system appropriately, but
    // custom plugins can enforce it like:
    //     .add_systems(Update, my_system.in_set(MainSet::Render));
    // NOTE: Systems inside UI / renderer plugins should tag themselves with
    // `MainSet::Render` so they run after simulation logic.
}
