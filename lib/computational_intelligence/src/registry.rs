use bevy::prelude::*;
use engine_core::core::World2D;
use engine_core::core::dim::Dim2;
use engine_core::core::AutomatonRule;
use std::collections::HashMap;
use std::sync::Arc;
 
 #[derive(Resource, Default)]
pub struct RuleRegistry {
    entries: HashMap<String, RuleEntry>,
}

struct RuleEntry {
    rule: Arc<dyn AutomatonRule<D = Dim2>>,
    seed: Option<fn(&mut World2D)>,
}

impl RuleRegistry {
    /// Register a rule without a seed function
    pub fn register(&mut self, id: &str, rule: Arc<dyn AutomatonRule<D = Dim2>>) {
        self.entries.insert(id.to_string(), RuleEntry { rule, seed: None });
    }

    /// Register a rule with an associated seeding function
    pub fn register_with_seed(&mut self, id: &str, rule: Arc<dyn AutomatonRule<D = Dim2>>, seed_fn: fn(&mut World2D)) {
        self.entries.insert(id.to_string(), RuleEntry { rule, seed: Some(seed_fn) });
    }

    /// Retrieve a rule by its ID
    pub fn get(&self, id: &str) -> Option<&Arc<dyn AutomatonRule<D = Dim2>>> {
        self.entries.get(id).map(|entry| &entry.rule)
    }

    /// Spawn the default pattern for the given rule ID (if available)
    pub fn spawn_default(&self, id: &str, world: &mut World2D) {
        if let Some(entry) = self.entries.get(id) {
            if let Some(seed_fn) = entry.seed {
                seed_fn(world);
            }
        }
    }

    /// Iterate over all registered rule IDs
    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }
}
