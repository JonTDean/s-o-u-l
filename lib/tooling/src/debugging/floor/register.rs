// lib/tooling/src/debugging/floor/register.rs
use bevy::prelude::*;
use engine_core::systems::registry::RuleRegistry;
use std::sync::Arc;

use crate::debugging::floor::{seeder::seed_checkerboard, StaticRule, DEBUG_FLOOR_ID};

/// Register the floor rule once – intended for Startup schedule.
pub fn ensure_debug_floor(mut rules: ResMut<RuleRegistry>) {
    if rules.get(DEBUG_FLOOR_ID).is_none() {
        rules.register_with_seed(
            DEBUG_FLOOR_ID,
            Arc::new(StaticRule),
            seed_checkerboard,
        );
    }
}
