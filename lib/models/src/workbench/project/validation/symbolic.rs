use crate::workbench::{catalog::Kind, symbolic::machine::Symbolic};
pub(super) fn valid(model: &Symbolic, kind: Kind) -> bool {
    match (model, kind) {
        (Symbolic::Finite(m), Kind::Nfa) => m.input.len() <= 256 && m.cursor <= m.input.len(),
        (Symbolic::Pushdown(m), Kind::Pushdown | Kind::Parentheses) => {
            m.input.len() <= 256 && m.cursor <= m.input.len() && m.stack.len() <= 256
        }
        (Symbolic::Bounded(m), Kind::Lba) => m.tape.len() <= 256 && m.head <= m.tape.len(),
        (Symbolic::Turing(m), Kind::Turing) => {
            m.tape.len() <= 10257
                && m.head <= m.tape.len()
                && m.transitions.len() <= 4096
                && m.accept_state != m.reject_state
        }
        (Symbolic::Koch { word, .. }, Kind::Koch) => word.chars().count() <= 4096,
        _ => false,
    }
}
