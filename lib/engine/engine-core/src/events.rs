//! Cross‑crate events shared by the whole application.

use bevy_ecs::{entity::Entity, event::Event};
use serde::{Deserialize, Serialize};

/// Globally‑unique handle assigned by the **CPU registry** to every
/// automaton that is alive during this run.
///
/// Only the *computational_intelligence* crate mutates / increments this
/// value, but the type lives in *engine_core* so low‑level modules (GPU,
/// events, render bridges) can reference it **without** creating a
/// dependency cycle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AutomatonId(pub u32);

/// High‑level requests coming from the UI / scenarios.
#[derive(Event)]
pub enum AutomataCommand {
    /// Spawn the default seed pattern that belongs to the rule `id`.
    SeedPattern { id: String },

    /// Despawn **all** running automata.
    Clear,
}

/// Sent immediately after a new automaton has been spawned.
///
/// * `id`     – logical identifier inside `AutomataRegistry` (CPU side)  
/// * `entity` – concrete `Entity` that represents this automaton in the
///              Bevy ECS (GPU slice allocator needs this)
#[derive(Event)]
pub struct AutomatonAdded {
    pub id:     AutomatonId,    // logical CPU‑side handle
    pub entity: Entity,         // ECS entity for GPU slice allocation
}

/// Emitted when an automaton is removed (optional convenience).
#[derive(Event)]
pub struct AutomatonRemoved {
    pub id: AutomatonId,
}
