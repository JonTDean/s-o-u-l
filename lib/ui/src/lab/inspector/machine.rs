//! Symbolic machine readouts: result, states, stack, tape position, Turing table.
use models::{
    automata::classical::turing::universal_tm::{Direction, TuringMachine},
    workbench::symbolic::machine::Symbolic,
};

/// Key/value rows for the inspector's machine section.
pub fn summary(machine: &Symbolic) -> Vec<(&'static str, String)> {
    let result = match machine.result() {
        Some(true) => "accepted",
        Some(false) => "rejected",
        None if machine.halted() => "bounded (4096 symbols)",
        None => "running",
    };
    let mut rows = vec![("result", result.to_string())];
    match machine {
        Symbolic::Finite(m) => {
            let mut states: Vec<_> = m.states.iter().copied().collect();
            states.sort_unstable();
            rows.push(("states", format!("{states:?}")));
            rows.push(("read", format!("{}/{}", m.cursor, m.input.len())));
        }
        Symbolic::Pushdown(m) => {
            rows.push(("stack", m.stack.iter().collect()));
            rows.push(("read", format!("{}/{}", m.cursor, m.input.len())));
        }
        Symbolic::Bounded(m) => rows.push(("head", format!("{}/{}", m.head, m.tape.len()))),
        Symbolic::Turing(m) => rows.push(("state", format!("q{}", m.state))),
        Symbolic::Koch { word, .. } => rows.push(("symbols", word.chars().count().to_string())),
    }
    rows
}

/// `q<state> <read> → <write> <L|R> q<next>` rows, blank shown as `␣`, sorted by state then symbol.
pub fn turing_table(machine: &TuringMachine) -> Vec<String> {
    let glyph = |c: char| if c == ' ' { '␣' } else { c };
    let mut entries: Vec<_> = machine.transitions.iter().collect();
    entries.sort_by_key(|(key, _)| **key);
    entries
        .into_iter()
        .map(|((state, read), t)| {
            let dir = match t.direction {
                Direction::Left => 'L',
                _ => 'R',
            };
            format!(
                "q{state} {} → {} {dir} q{}",
                glyph(*read),
                glyph(t.write),
                t.new_state
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::workbench::{catalog::Kind, symbolic::machine::Symbolic};

    #[test]
    fn summaries_name_state_and_result() {
        let nfa = Symbolic::new(Kind::Nfa, "aaab");
        let rows = summary(&nfa);
        assert_eq!(rows[0], ("result", "running".to_string()));
        assert!(rows.iter().any(|(k, _)| *k == "states"));
        let mut pda = Symbolic::new(Kind::Pushdown, "aabb");
        pda.step();
        assert!(summary(&pda).iter().any(|(k, v)| *k == "stack" && v == "A"));
    }

    #[test]
    fn turing_table_lists_every_transition_sorted() {
        let Symbolic::Turing(tm) = Symbolic::new(Kind::Turing, "aa") else {
            unreachable!()
        };
        let table = turing_table(&tm);
        assert_eq!(table, ["q0 ␣ → ␣ L q1", "q0 a → X R q0"]);
    }
}
