//! The tiniest possible DFA:  L = { a⁺ }  (“one or more *a*’s”).
//!
//! States: 0 (start / reject), 1 (accept)
//! Alphabet: { 'a' }
//! δ:
//!   (0,'a') → 1
//!   (1,'a') → 1
//!   otherwise → 0 (dead   state)

use crate::intelligence_engine::state_machines::bounded::{
    FiniteStateMachine, TransitionFn,
};
use std::collections::HashSet;

/// Concrete transition function object.
#[derive(Clone)]
pub struct DfaAPlus;

impl TransitionFn<u8, char> for DfaAPlus {
    fn delta(&self, state: &u8, sym: &char) -> Vec<u8> {
        match (*state, *sym) {
            (0, 'a') => vec![1],
            (1, 'a') => vec![1],
            _        => vec![0], // dead / non‑accepting sink
        }
    }
}

/// Convenience constructor so the caller can just write `dfa()`
/// when they need an instance (e.g. in tests or ECS resources).
pub fn dfa() -> FiniteStateMachine<u8, char, DfaAPlus> {
    use std::iter::FromIterator;

    FiniteStateMachine {
        states:        HashSet::from_iter([0, 1]),
        alphabet:      HashSet::from_iter(['a']),
        delta:         DfaAPlus,
        start_state:   0,
        accept_states: HashSet::from_iter([1]),
    }
}
