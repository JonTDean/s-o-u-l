use super::*;
use crate::automata::classical::{
    regular::regex_nfa::NFA,
    turing::universal_tm::{Direction, TMTransition, example_tm_replace_a},
};
use std::collections::{HashMap, HashSet};

#[test]
fn nfa_epsilon_cycles_and_branching_remain_closed() {
    let nfa = NFA {
        start: 0,
        accept_states: HashSet::from([3]),
        transitions: HashMap::from([
            ((0, None), vec![1]),
            ((1, None), vec![0, 2]),
            ((2, Some('x')), vec![3]),
        ]),
    };
    assert_eq!(nfa.initial_states(), HashSet::from([0, 1, 2]));
    assert_eq!(nfa.advance(&nfa.initial_states(), 'x'), HashSet::from([3]));
    assert!(nfa.is_match("x"));
    assert!(!nfa.is_match(""));
    assert!(!nfa.is_match("xx"));
}

#[test]
fn turing_rewrites_halts_rejects_and_extends_both_tape_ends() {
    let machine = run(Kind::Turing, "aaaa");
    assert_eq!(machine.result(), Some(true));
    let Symbolic::Turing(m) = machine else {
        unreachable!()
    };
    assert_eq!(m.tape.iter().collect::<String>(), "XXXX ");
    assert_eq!(run(Kind::Turing, "ab").result(), Some(false));
    let mut machine = example_tm_replace_a();
    machine.head = 4;
    machine.step();
    assert_eq!(machine.tape.len(), 5);
    machine = example_tm_replace_a();
    machine.transitions.insert(
        (0, ' '),
        TMTransition {
            write: 'x',
            new_state: 1,
            direction: Direction::Left,
        },
    );
    machine.step();
    assert_eq!(machine.tape, vec![' ', 'x']);
    assert_eq!(machine.head, 0);
}

#[test]
fn lsystem_rewrites_in_parallel_and_stops_before_exceeding_its_bound() {
    let mut machine = Symbolic::new(Kind::Koch, "F");
    for generation in 1..=5 {
        machine.step();
        let Symbolic::Koch { word, limited } = &machine else {
            unreachable!()
        };
        assert!(!limited);
        assert_eq!(
            word.chars().filter(|&c| c == 'F').count(),
            4_usize.pow(generation)
        );
        if generation == 1 {
            assert_eq!(word, "F+F--F+F");
        }
        assert!(word.len() <= 4096);
    }
    machine.step();
    assert!(machine.halted());
    assert_eq!(machine.result(), None);
}
