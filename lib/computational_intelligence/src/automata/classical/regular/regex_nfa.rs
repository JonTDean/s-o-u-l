//! Nondeterministic Finite Automaton (NFA) implementation and regex example.

use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct NFA {
    pub transitions: HashMap<(usize, Option<char>), Vec<usize>>,
    pub start: usize,
    pub accept_states: HashSet<usize>,
}

impl NFA {
    /// Compute the epsilon-closure of a set of states (all states reachable via ε-moves).
    fn epsilon_closure(&self, states: &HashSet<usize>) -> HashSet<usize> {
        let mut closure = states.clone();
        let mut stack: Vec<usize> = states.iter().cloned().collect();
        while let Some(s) = stack.pop() {
            if let Some(eps_targets) = self.transitions.get(&(s, None)) {
                for &t in eps_targets {
                    if !closure.contains(&t) {
                        closure.insert(t);
                        stack.push(t);
                    }
                }
            }
        }
        closure
    }

    /// Simulates the NFA on the input string. Returns true if it accepts.
    pub fn is_match(&self, input: &str) -> bool {
        // current set of possible states (start with epsilon-closure of start)
        let mut current_states = HashSet::new();
        current_states.insert(self.start);
        current_states = self.epsilon_closure(&current_states);
        // consume input symbols
        for ch in input.chars() {
            let mut next_states = HashSet::new();
            for &state in &current_states {
                if let Some(targets) = self.transitions.get(&(state, Some(ch))) {
                    for &t in targets {
                        next_states.insert(t);
                    }
                }
            }
            // get epsilon-closure of next_states
            current_states = self.epsilon_closure(&next_states);
            if current_states.is_empty() {
                return false; // no possible states, fail early
            }
        }
        // after consuming input, check if any state is accepting
        current_states.intersection(&self.accept_states).next().is_some()
    }
}

/// Constructs a simple NFA that matches the regex `a* b` (any number of 'a's followed by a 'b').
pub fn example_nfa() -> NFA {
    let mut transitions: HashMap<(usize, Option<char>), Vec<usize>> = HashMap::new();
    // NFA for a* b:
    // states: 0 (start), 1 (accept)
    // transitions: 0 -'a'-> 0 (loop on 'a'), 0 -'b'-> 1
    transitions.insert((0, Some('a')), vec![0]);
    transitions.insert((0, Some('b')), vec![1]);
    let start = 0;
    let accept_states = vec![1].into_iter().collect();
    NFA { transitions, start, accept_states }
}
