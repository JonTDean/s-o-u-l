use super::hpp::{HPPRule, seed_hpp};
use bevy::prelude::*;
use engine_core::prelude::RuleRegistry;

/// Registers the HPP lattice-gas rule in the rule registry.
///
/// This makes the "particle:hpp" automaton available for spawning.
pub struct ParticleAutomataPlugin;
impl Plugin for ParticleAutomataPlugin {
    fn build(&self, app: &mut App) {
        let mut reg = app
            .world_mut()
            .remove_resource::<RuleRegistry>()
            .unwrap_or_default();
        reg.register_with_seed("particle:hpp", HPPRule::boxed(), seed_hpp);
        reg.register_with_seed(
            "life:conway",
            std::sync::Arc::new(super::life::Life),
            super::life::seed_life,
        );
        app.insert_resource(reg);
    }
}
