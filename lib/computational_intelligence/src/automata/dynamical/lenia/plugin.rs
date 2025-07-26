use bevy::prelude::*;
use crate::registry::RuleRegistry;
use super::{LeniaRule, seed_lenia, seed_orbium};

/// Registers the Lenia rule and its seed patterns into the rule registry.
///
/// This plugin does **not** start any simulation by itself. It simply makes Lenia available for use. 
/// Actual Lenia automata will be spawned when a scenario is started or when the user adds one at runtime.
pub struct LeniaPlugin;
impl Plugin for LeniaPlugin {
    fn build(&self, app: &mut App) {
        // Access the global RuleRegistry resource and register Lenia variants
        let mut reg = app.world_mut().remove_resource::<RuleRegistry>().unwrap_or_default();
        reg.register_with_seed("lenia", LeniaRule::boxed(), seed_lenia);
        reg.register_with_seed("lenia:orbium", LeniaRule::boxed(), seed_orbium);
        app.insert_resource(reg);
    }
}
