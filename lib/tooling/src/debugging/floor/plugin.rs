use std::sync::Arc;

use bevy::app::{App, Plugin, Startup, Update};
use engine_core::prelude::{in_state, AppState, IntoScheduleConfigs, RuleRegistry};

use crate::debugging::floor::{
    register::ensure_debug_floor, seeder::seed_checkerboard, system::toggle_floor_keyboard, StaticRule, DEBUG_FLOOR_ID
};




pub struct DebugFloorPlugin;
impl Plugin for DebugFloorPlugin {
    fn build(&self, app: &mut App) {
        /* 1 ░ register rule + seed once */
        {
            let mut rules = app.world_mut().resource_mut::<RuleRegistry>();
            if rules.get(DEBUG_FLOOR_ID).is_none() {
                rules.register_with_seed(
                    DEBUG_FLOOR_ID,
                    Arc::new(StaticRule),
                    seed_checkerboard,
                );
            }
        }

        /* 2 ░ ⇧ + F toggler */
        app.add_systems(
            Update,
            toggle_floor_keyboard
                .run_if(in_state(AppState::InGame))
                .in_set(engine_core::systems::schedule::MainSet::Input),
        );
    }
}


pub struct DebugFloorRulePlugin;
impl Plugin for DebugFloorRulePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ensure_debug_floor);
    }
}