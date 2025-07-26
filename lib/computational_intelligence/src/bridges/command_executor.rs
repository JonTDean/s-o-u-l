//! Turns high‑level `AutomataCommand` events coming from the UI / scenario
//! controller into concrete `AutomatonInfo`s stored in `AutomataRegistry`.

use std::sync::Arc;

use bevy::prelude::*;
use engine_core::{
    engine::grid::{DenseGrid, GridBackend},
    events::AutomataCommand,
};
use serde_json::Value;

use crate::registry::{
    AutomataRegistry, AutomatonAdded, AutomatonId, AutomatonInfo, RuleRegistry,
};

/* --------------------------------------------------------------------- */

/// Size of a freshly‑spawned dense grid (256 × 256).
const DEFAULT_GRID: u32 = 256;
/// World‑space size of one cell (in pixels / units).
const DEFAULT_CELL: f32 = 4.0;
/// Background colour used by the renderer.
const BG: Color = Color::srgb(0.07, 0.07, 0.07);

/* --------------------------------------------------------------------- */

pub struct CommandExecutorPlugin;
impl Plugin for CommandExecutorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_commands
                .run_if(in_state(engine_core::state::AppState::InGame)),
        );
    }
}

/* --------------------------------------------------------------------- */

#[allow(clippy::too_many_arguments)]
fn handle_commands(
    mut cmd_reader:   EventReader<AutomataCommand>,
    mut registry:     ResMut<AutomataRegistry>,
    rules:            Res<RuleRegistry>,
    mut added_writer: EventWriter<AutomatonAdded>,
) {
    for event in cmd_reader.read() {
        match event {
            AutomataCommand::SeedPattern { id } => {
                let Some((rule, seed_fn)) = rules.get(id.as_str()) else {
                    warn!("Unknown automaton pattern “{id}” – ignored");
                    continue;
                };

                /* 1 ─ build an empty dense grid */
                let size  = UVec2::splat(DEFAULT_GRID);
                let grid  = DenseGrid::blank(size);
                let mut backend = GridBackend::Dense(grid);

                /* 2 ─ call the rule’s default seeding function (if any) */
                if let Some(seed) = seed_fn {
                    seed(&mut backend);
                }

                /* 3 ─ assemble AutomatonInfo */
                let info = AutomatonInfo {
                    id:             AutomatonId(0), // overwritten in register()
                    name:           id.clone(),
                    rule:           Arc::clone(rule),
                    params:         Value::Null,
                    seed_fn:        *seed_fn,
                    grid:           backend,
                    dimension:      2,
                    cell_size:      DEFAULT_CELL,
                    background_color: BG,
                    palette:        None,
                };

                /* 4 ─ register + notify renderer */
                let new_id = registry.register(info);
                added_writer.write(AutomatonAdded { id: new_id });
            }

            AutomataCommand::Clear => {
                // Despawn *all* automata
                let automata = registry.list().iter().map(|a| a.id).collect::<Vec<_>>();
                for id in automata {
                    registry.remove(id);
                    // The renderer has an EventReader<AutomatonRemoved>,
                    // but for now we simply rely on the map shrinking.
                }
            }
        }
    }
}
