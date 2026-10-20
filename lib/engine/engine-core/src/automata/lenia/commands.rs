use super::LeniaSimulation;
use bevy::prelude::*;
use simulation_kernel::core::lenia::{LeniaParams, LeniaSeed};

/// A workbench action directed at one live Lenia entity.
#[derive(Event, Clone, Copy, Debug)]
pub struct LeniaCommand {
    /// Owner entity; stale generations are ignored by the ECS query.
    pub entity: Entity,
    /// Requested transition.
    pub action: LeniaAction,
}

/// Simulation operations used by UI controls and native verification.
#[derive(Clone, Copy, Debug)]
pub enum LeniaAction {
    /// Suspend or resume automatic ticks.
    Pause(bool),
    /// Request exactly one tick when paused.
    Step,
    /// Restore the selected seed.
    Reset,
    /// Apply validated growth parameters without resetting density.
    Parameters(LeniaParams),
    /// Select and restore a new seed.
    Seed(LeniaSeed),
}

/// Registers the same command handler for workbench UI and headless tests.
pub struct LeniaControlPlugin;

impl Plugin for LeniaControlPlugin {
    fn build(&self, app: &mut App) {
        crate::automata::channel::ensure_plugin(app);
        app.add_event::<LeniaCommand>().add_systems(
            Update,
            apply_commands.in_set(crate::automata::channel::ChannelSet::Tick),
        );
    }
}

fn apply_commands(
    mut commands: EventReader<LeniaCommand>,
    mut simulations: Query<&mut LeniaSimulation>,
) {
    for command in commands.read() {
        let Ok(mut sim) = simulations.get_mut(command.entity) else {
            continue;
        };
        match command.action {
            LeniaAction::Pause(paused) => sim.paused = paused,
            LeniaAction::Step if sim.paused => sim.target_tick += 1,
            LeniaAction::Step => {}
            LeniaAction::Reset => sim.reset(),
            LeniaAction::Parameters(params) => {
                if let Ok(params) = params.validate() {
                    sim.params = params;
                }
            }
            LeniaAction::Seed(seed) => {
                sim.seed = seed;
                sim.reset();
            }
        }
    }
}
