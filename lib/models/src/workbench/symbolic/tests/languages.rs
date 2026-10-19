use super::*;
use crate::automata::classical::{
    contextful::lba_examples::is_a_n_b_n_c_n, contextless::ll1_pushdown::is_a_n_b_n,
};

#[test]
fn bounded_languages_match_exhaustive_short_words_and_reject_unicode() {
    for length in 0..=6 {
        for mut code in 0..3_usize.pow(length) {
            let input: String = (0..length)
                .map(|_| {
                    let c = ['a', 'b', 'c'][code % 3];
                    code /= 3;
                    c
                })
                .collect();
            let n = input.len() / 2;
            let expected_ab = input == format!("{}{}", "a".repeat(n), "b".repeat(n));
            let n = input.len() / 3;
            let expected_abc =
                n > 0 && input == format!("{}{}{}", "a".repeat(n), "b".repeat(n), "c".repeat(n));
            assert_eq!(
                run(Kind::Pushdown, &input).result(),
                Some(expected_ab),
                "{input}"
            );
            assert_eq!(
                run(Kind::Lba, &input).result(),
                Some(expected_abc),
                "{input}"
            );
            assert_eq!(is_a_n_b_n(&input), expected_ab);
            assert_eq!(is_a_n_b_n_c_n(&input), expected_abc);
            let expected_nfa = input.ends_with('b')
                && input[..input.len().saturating_sub(1)]
                    .chars()
                    .all(|c| c == 'a');
            assert_eq!(
                run(Kind::Nfa, &input).result(),
                Some(expected_nfa),
                "{input}"
            );
        }
    }
    for input in ["abcabc", "é", "☃", "AAA", "aébc"] {
        assert_eq!(run(Kind::Lba, input).result(), Some(false));
        assert!(!is_a_n_b_n_c_n(input));
    }
}

#[test]
fn pushdown_stack_tracks_nesting_and_rejects_nonparenthesis_symbols() {
    for (input, expected) in [
        ("", true),
        ("(())()", true),
        (")(", false),
        ("(()", false),
        ("(x)", false),
    ] {
        assert_eq!(run(Kind::Parentheses, input).result(), Some(expected));
    }
}

#[test]
fn bounded_tape_never_allocates_outside_its_input() {
    let mut machine = Symbolic::new(Kind::Lba, "aaaabbbbcccc");
    for _ in 0..500 {
        machine.step();
        if let Symbolic::Bounded(m) = &machine {
            assert_eq!(m.tape.len(), 12);
        }
        if machine.halted() {
            break;
        }
    }
    assert_eq!(machine.result(), Some(true));
}
