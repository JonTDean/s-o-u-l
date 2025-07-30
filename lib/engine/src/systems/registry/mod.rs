use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;                    // Color, Resource, etc.

use simulation_kernel::core::dim::Dim2;
use simulation_kernel::grid::GridBackend;
use simulation_kernel::AutomatonRule;

use crate::{events::AutomatonId, core::{world, automata::AutomatonInfo}};

/* ──────────────────────────────────────────────────────────────────── */
/* Rule registry                                                       */
/* ──────────────────────────────────────────────────────────────────── */

#[derive(Resource, Default)]
pub struct RuleRegistry {
    rules: HashMap<
        String,
        (Arc<dyn AutomatonRule<D = Dim2> + Send + Sync>, Option<fn(&mut GridBackend)>),
    >,
}

impl RuleRegistry {
    /* Register -------------------------------------------------------- */

    pub fn register_with_seed(
        &mut self,
        id: impl Into<String>,
        rule: Arc<dyn AutomatonRule<D = Dim2> + Send + Sync>,
        seed_fn: fn(&mut GridBackend),
    ) {
        self.rules.insert(id.into(), (rule, Some(seed_fn)));
    }

    pub fn register(
        &mut self,
        id: impl Into<String>,
        rule: Arc<dyn AutomatonRule<D = Dim2> + Send + Sync>,
    ) {
        self.rules.insert(id.into(), (rule, None));
    }

    /* Lookup ---------------------------------------------------------- */

    pub fn get(
        &self,
        id: &str,
    ) -> Option<&(Arc<dyn AutomatonRule<D = Dim2> + Send + Sync>, Option<fn(&mut GridBackend)>)>
    {
        self.rules.get(id)
    }

    /// Iterator over all registered IDs.
    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.rules.keys()
    }

    /* Convenience ----------------------------------------------------- */

    /// Spawn the *default* pattern of a rule into an existing `World2D`.
    pub fn spawn_default(&self, id: &str, world: &mut world::World2D) {
        if let Some(&(_, seed_opt)) = self.get(id) {
            if let Some(seed) = seed_opt {
                seed(&mut world.backend);
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct AutomataRegistry {
    automata: Vec<AutomatonInfo>,
    next_id:  u32,
}

impl AutomataRegistry {
    /* CRUD ------------------------------------------------------------ */

    pub fn register(&mut self, mut info: AutomatonInfo) -> AutomatonId {
        let id = AutomatonId(self.next_id);
        self.next_id += 1;
        info.id = id;
        self.automata.push(info);
        id
    }

    pub fn remove(&mut self, id: AutomatonId) {
        self.automata.retain(|a| a.id != id);
    }

    /* Read‑only helpers ---------------------------------------------- */

    pub fn list(&self) -> &[AutomatonInfo]                { &self.automata }
    pub fn get(&self, id: AutomatonId) -> Option<&AutomatonInfo> {
        self.automata.iter().find(|a| a.id == id)
    }
    pub fn find_by_name(&self, name: &str) -> Option<&AutomatonInfo> {
        self.automata.iter().find(|a| a.name == name)
    }

    /* Mutable iterator (used by the stepper) ------------------------- */

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, AutomatonInfo> {
        self.automata.iter_mut()
    }
}