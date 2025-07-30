//! Root plugin for the **Computational‑Intelligence (CI)** layer.
//!
//! # What this plugin does
//!
//! 1. **Initialises global CI resources & events**
//!    * [`RuleRegistry`] —— look‑up table for every compiled‐in automaton rule.
//!    * [`AutomataRegistry`] —— runtime list of **live** automata
//!      (grid, rule handle, colours, etc.).  
//!      Both registries are created with [`App::init_resource`] so that any
//!      `Res<…>` or `ResMut<…>` access in downstream systems is **always**
//!      valid — even before any rule or automaton is registered.
//!    * [`AutomatonAdded`] / [`AutomatonRemoved`] —— emitted whenever the
//!      simulation spawns or despawns an automaton.  Declared here so that
//!      all `EventReader`s (render / UI side) can be safely constructed at
//!      startup — without the notorious *“Event not initialized”* panic.
//!
//! 2. **Adds every CI sub‑plugin** in a single call so the main Bevy `App`
//!    only needs one line of code:
//!
//! ```no_run
//! app.add_plugins(computational_intelligence::ComputationalIntelligencePlugin);
//! ```
//!
//! 3. **Thread‑safety guarantee** – every resource registered here is
//!    `Send + Sync`, so Bevy’s scheduler may freely parallelise CI, rendering
//!    and input systems across threads.
//!
//! ## Extending the CI layer
//!
//! New CI modules (optimisers, analytics, ML, …) should expose their own
//! `FooPlugin`.  Simply append that plugin to the `.add_plugins(( … ))` call
//! below — **no other changes** are required by the host application.

use bevy::prelude::*;
use engine::{
    events::{AutomatonAdded, AutomatonRemoved},
    systems::registry::{RuleRegistry, AutomataRegistry},
};

use crate::automata::plugin::AutomataPlugin;

/// **Single entry‑point** for everything CI‑related.
///
/// Adding this plugin gives the game:
///
/// * **Rule & automata registries** ready for use in any system.
/// * Global `AutomatonAdded` / `AutomatonRemoved` events (avoids Bevy panics).
/// * All default rule‑set families + naïve world stepper (via
///   [`AutomataPlugin`]), fully wired‑up and ticking.
///
/// ```no_run
/// use bevy::prelude::*;
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(computational_intelligence::ComputationalIntelligencePlugin)
///         .run();
/// }
/// ```
pub struct ComputationalIntelligencePlugin;

impl Plugin for ComputationalIntelligencePlugin {
    fn build(&self, app: &mut App) {
        // ------------------------------------------------------------------
        // 1.  Core resources — created exactly *once*.
        //     If they already exist (e.g. in a hot‑reload scenario), the
        //     second call is a no‑op.  This guarantees that *every* system
        //     accessing these resources finds a valid value.
        // ------------------------------------------------------------------
        app.init_resource::<RuleRegistry>()
           .init_resource::<AutomataRegistry>();

        // ------------------------------------------------------------------
        // 2.  Global events — MUST be registered **before** any `EventReader`
        //     is instantiated (renderers, HUD overlays, …); otherwise Bevy
        //     aborts with *“Event not initialized”*.
        // ------------------------------------------------------------------
        app.add_event::<AutomatonAdded>()
           .add_event::<AutomatonRemoved>();

        // ------------------------------------------------------------------
        // 3.  Sub‑plugins — classical & dynamical rule families, world
        //     stepper, and anything we add in future roadmaps.
        // ------------------------------------------------------------------
        app.add_plugins(AutomataPlugin);
    }
}
