use crate::automata::classical::regular::regex_nfa::{NFA, example_nfa};
use std::collections::HashSet;

/// Incremental NFA execution, retaining all epsilon-closed active states.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Finite {
    pub states: HashSet<usize>,
    pub input: Vec<char>,
    pub cursor: usize,
    pub result: Option<bool>,
    pub(crate) machine: NFA,
}
impl Finite {
    /// Start the a*b machine with its epsilon-closed initial state set.
    pub fn new(input: &str) -> Self {
        let machine = example_nfa();
        Self {
            states: machine.initial_states(),
            input: input.chars().collect(),
            cursor: 0,
            result: None,
            machine,
        }
    }
    /// Consume at most one input symbol and preserve a terminal result.
    pub fn step(&mut self) {
        if self.result.is_some() {
            return;
        }
        if let Some(&symbol) = self.input.get(self.cursor) {
            self.states = self.machine.advance(&self.states, symbol);
            self.cursor += 1;
        }
        if self.cursor == self.input.len() || self.states.is_empty() {
            self.result = Some(
                !self.states.is_disjoint(&self.machine.accept_states)
                    && self.cursor == self.input.len(),
            );
        }
    }
}

/// Stack machine for ordered a^n b^n or strict balanced parentheses.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Pushdown {
    pub input: Vec<char>,
    pub cursor: usize,
    pub stack: Vec<char>,
    pub result: Option<bool>,
    pub(crate) parentheses: bool,
    reading_b: bool,
}
impl Pushdown {
    /// Select strict parentheses or ordered a^n b^n recognition.
    pub fn new(input: &str, parentheses: bool) -> Self {
        Self {
            input: input.chars().collect(),
            cursor: 0,
            stack: Vec::new(),
            result: None,
            parentheses,
            reading_b: false,
        }
    }
    /// Consume at most one input symbol and preserve a terminal result.
    pub fn step(&mut self) {
        if self.result.is_some() {
            return;
        }
        if let Some(&symbol) = self.input.get(self.cursor) {
            let valid = if self.parentheses {
                match symbol {
                    '(' => {
                        self.stack.push('(');
                        true
                    }
                    ')' => self.stack.pop() == Some('('),
                    _ => false,
                }
            } else {
                match symbol {
                    'a' if !self.reading_b => {
                        self.stack.push('A');
                        true
                    }
                    'b' => {
                        self.reading_b = true;
                        self.stack.pop() == Some('A')
                    }
                    _ => false,
                }
            };
            self.cursor += 1;
            if !valid {
                self.result = Some(false);
            }
        }
        if self.cursor == self.input.len() && self.result.is_none() {
            self.result = Some(self.stack.is_empty());
        }
    }
}
