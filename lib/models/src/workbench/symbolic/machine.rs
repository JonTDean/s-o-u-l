use super::{
    bounded::Bounded,
    recognizers::{Finite, Pushdown},
};
use crate::{
    automata::classical::{
        contextless::l_system::lsystem_step,
        turing::universal_tm::{TuringMachine, example_tm_replace_a},
    },
    workbench::catalog::Kind,
};
use std::collections::HashMap;

/// Executable symbolic state; resource exhaustion is distinct from rejection.
pub enum Symbolic {
    Finite(Finite),
    Pushdown(Pushdown),
    Bounded(Bounded),
    Turing(TuringMachine),
    Koch { word: String, limited: bool },
}
impl Symbolic {
    pub fn new(kind: Kind, input: &str) -> Self {
        match kind {
            Kind::Nfa => Self::Finite(Finite::new(input)),
            Kind::Pushdown | Kind::Parentheses => {
                Self::Pushdown(Pushdown::new(input, kind == Kind::Parentheses))
            }
            Kind::Lba => Self::Bounded(Bounded::new(input)),
            Kind::Turing => {
                let mut machine = example_tm_replace_a();
                machine.tape = input.chars().collect();
                Self::Turing(machine)
            }
            Kind::Koch => Self::Koch {
                word: input.into(),
                limited: false,
            },
            _ => unreachable!("symbolic kind required"),
        }
    }
    pub fn step(&mut self) {
        match self {
            Self::Finite(machine) => machine.step(),
            Self::Pushdown(machine) => machine.step(),
            Self::Bounded(machine) => machine.step(),
            Self::Turing(machine) => {
                machine.step();
            }
            Self::Koch { word, limited } => {
                let length: usize = word.chars().map(|c| if c == 'F' { 8 } else { 1 }).sum();
                if length > 4096 {
                    *limited = true;
                    return;
                }
                *word = lsystem_step(word, &HashMap::from([('F', "F+F--F+F".into())]));
            }
        }
    }
    /// Halting status; Koch uses its explicit expansion bound.
    pub fn halted(&self) -> bool {
        match self {
            Self::Finite(m) => m.result.is_some(),
            Self::Pushdown(m) => m.result.is_some(),
            Self::Bounded(m) => m.result.is_some(),
            Self::Turing(m) => m.state == m.accept_state || m.state == m.reject_state,
            Self::Koch { limited, .. } => *limited,
        }
    }
    /// Language acceptance, independent of whether the machine is still running.
    pub fn result(&self) -> Option<bool> {
        match self {
            Self::Finite(m) => m.result,
            Self::Pushdown(m) => m.result,
            Self::Bounded(m) => m.result,
            Self::Turing(m) if self.halted() => Some(m.state == m.accept_state),
            _ => None,
        }
    }
}
