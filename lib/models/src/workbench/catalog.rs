//! Stable identifiers and human-readable contracts for executable families.

/// Every non-Lenia model supported by the CPU workbench.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Rule30,
    Rule110,
    Life,
    Hpp,
    Nfa,
    Pushdown,
    Parentheses,
    Koch,
    Lba,
    Turing,
    Boids,
    Ants,
}

impl Kind {
    /// Ordered catalog used by the UI and native qualification.
    pub const ALL: [Self; 12] = [
        Self::Rule30,
        Self::Rule110,
        Self::Life,
        Self::Hpp,
        Self::Nfa,
        Self::Pushdown,
        Self::Parentheses,
        Self::Koch,
        Self::Lba,
        Self::Turing,
        Self::Boids,
        Self::Ants,
    ];

    /// Stable spawn identifier.
    pub fn id(self) -> &'static str {
        match self {
            Self::Rule30 => "wolfram:rule30",
            Self::Rule110 => "wolfram:rule110",
            Self::Life => "life:conway",
            Self::Hpp => "particle:hpp",
            Self::Nfa => "regular:nfa",
            Self::Pushdown => "context-free:anbn",
            Self::Parentheses => "context-free:parentheses",
            Self::Koch => "context-free:koch",
            Self::Lba => "context-sensitive:anbncn",
            Self::Turing => "turing:replace-a",
            Self::Boids => "swarm:boids",
            Self::Ants => "swarm:ants",
        }
    }

    /// Descriptive label with no claim that the example TM is universal.
    pub fn title(self) -> &'static str {
        match self {
            Self::Rule30 => "Rule 30 · space-time",
            Self::Rule110 => "Rule 110 · space-time",
            Self::Life => "Conway Life · B3/S23",
            Self::Hpp => "HPP lattice gas",
            Self::Nfa => "Finite automaton · a*b",
            Self::Pushdown => "Pushdown · aⁿbⁿ",
            Self::Parentheses => "Pushdown · balanced parentheses",
            Self::Koch => "Koch L-system",
            Self::Lba => "Linear-bounded · aⁿbⁿcⁿ",
            Self::Turing => "Turing machine · a → X",
            Self::Boids => "Boids flock",
            Self::Ants => "Ant foraging",
        }
    }

    /// Lookup without silently substituting another model.
    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.id() == id)
    }

    /// Editable initial symbolic input; grid/agent seeds are deterministic.
    pub fn input(self) -> &'static str {
        match self {
            Self::Nfa => "aaab",
            Self::Pushdown => "aaabbb",
            Self::Parentheses => "(()())",
            Self::Koch => "F",
            Self::Lba => "aaabbbccc",
            Self::Turing => "aaaa",
            _ => "",
        }
    }
}
