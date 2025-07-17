//! Generic definitions that every *bounded* automaton (Type‑0 … Type‑3) can reuse.
//!
//! The naming mirrors Chomsky’s hierarchy but stays Rust‑centric:
//! * `TransitionFn` –– δ
//! * `FiniteStateMachine` –– ⟨Q, Σ, δ, q₀, F⟩
//!
//! Higher‑power machines (push‑down, LB‑NDTM, Turing) extend the
//! basic tuple with extra memory structures; that will live in each
//! type‑specific sub‑module.

use std::collections::HashSet;

pub(crate) mod type_0;
pub(crate) mod type_1;
pub(crate) mod type_2;
pub(crate) mod type_3;
pub(crate) mod type_4;
pub(crate) mod type_n;


/// Universal transition relation.
///
/// *Deterministic* DFAs can return ≤ 1 state;  
/// *Non‑deterministic* NFAs may return *n* states.  
/// (We keep the more‑general signature so the same trait serves both.)
pub trait TransitionFn<Q, S> {
    fn delta(&self, state: &Q, symbol: &S) -> Vec<Q>;
}

/// Canonical FSM tuple ⟨Q, Σ, δ, q₀, F⟩ (Type‑3).
///
/// The generic is 100 % library‑friendly:
/// * `Q` – state type (must be `Eq + Hash` for the `HashSet`s)
/// * `S` – alphabet symbol type
/// * `D` – transition‑function object
#[derive(Clone)]
pub struct FiniteStateMachine<Q, S, D>
where
    Q: Eq + std::hash::Hash + Clone,
    S: Eq + std::hash::Hash + Clone,
    D: TransitionFn<Q, S> + Clone,
{
    pub states:        HashSet<Q>,
    pub alphabet:      HashSet<S>,
    pub delta:         D,
    pub start_state:   Q,
    pub accept_states: HashSet<Q>,
}

impl<Q, S, D> FiniteStateMachine<Q, S, D>
where
    Q: Eq + std::hash::Hash + Clone,
    S: Eq + std::hash::Hash + Clone,
    D: TransitionFn<Q, S> + Clone
{
    /// Runs the machine on an input iterator and returns `true` iff it halts in an accept state.
    pub fn accepts<I>(&self, input: I) -> bool
    where
        I: IntoIterator<Item = S>,
    {
        // For *deterministic* machines we always keep exactly one current state.
        // For an NFA you would keep a HashSet and union the next‑state sets.
        let mut current = self.start_state.clone();
        for sym in input {
            if !self.alphabet.contains(&sym) {
                // Unknown symbol ⇒ reject immediately.
                return false;
            }
            let mut next = self.delta.delta(&current, &sym);
            if next.len() != 1 {
                // DFA invariant violated; NFA runner would differ.
                return false;
            }
            current = next.pop().unwrap();
        }
        self.accept_states.contains(&current)
    }
}

// ---------------------------------------------------------------------------
// PUBLIC REGISTRY – UI can query this without knowing concrete types
// ---------------------------------------------------------------------------

/// Opaque handle returned by every factory.
/// Split this into Chomskis Hierarchy
/// Followed by the determinism type
/// Followed by the class of the machine
pub enum MachineInstance {
    Type3Dfa(Box<dyn Fn() -> bool + Send + Sync>),   // you can grow this enum later
    /* Type-2, Type-1 … */
}

/// Function pointer that builds a machine *and* an evaluator closure.
pub type MachineFactory = fn() -> MachineInstance;

/// Registry is populated in each sub-module’s `pub fn register()` impl.
pub fn all_factories() -> Vec<(&'static str, MachineFactory)> {
    let mut v = Vec::new();
    type_3::register(&mut v);
    // type_2::register(&mut v); …
    v
}