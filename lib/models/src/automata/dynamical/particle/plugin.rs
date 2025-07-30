use bevy::prelude::*;
use engine_core::prelude::RuleRegistry;
use super::hpp::{HPPRule, seed_hpp};

/// Registers the HPP lattice-gas rule in the rule registry.
///
/// This makes the "particle:hpp" automaton available for spawning.
pub struct ParticleAutomataPlugin;
impl Plugin for ParticleAutomataPlugin {
    fn build(&self, app: &mut App) {
        let mut reg = app.world_mut().remove_resource::<RuleRegistry>().unwrap_or_default();
        reg.register_with_seed("particle:hpp", HPPRule::boxed(), seed_hpp);
        app.insert_resource(reg);
    }
}
