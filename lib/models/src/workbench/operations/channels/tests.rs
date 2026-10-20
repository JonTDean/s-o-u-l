//! Headline metrics are computed from model state exactly as documented.
use super::metrics::{metric_for, value};
use crate::workbench::{catalog::Kind, session::Session};

#[test]
fn every_kind_has_a_named_metric() {
    for kind in Kind::ALL {
        assert!(!metric_for(kind).name.is_empty(), "{kind:?}");
    }
    assert_eq!(metric_for(Kind::Life).name, "population");
    assert_eq!(metric_for(Kind::Turing).name, "steps");
}

#[test]
fn life_counts_the_glider_and_rule30_its_newest_row() {
    assert_eq!(value(&Session::new(Kind::Life)), 5.0);
    let mut rule = Session::new(Kind::Rule30);
    assert_eq!(value(&rule), 1.0);
    rule.single_step();
    assert_eq!(value(&rule), 3.0, "rule 30 from one cell: 111");
}

#[test]
fn hpp_particle_count_is_conserved_before_walls_reach_edges() {
    let mut gas = Session::new(Kind::Hpp);
    let initial = value(&gas);
    assert_eq!(initial, 48.0);
    for _ in 0..12 {
        gas.single_step();
        assert_eq!(value(&gas), initial);
    }
}

#[test]
fn koch_length_symbolic_steps_boids_and_ants() {
    let mut koch = Session::new(Kind::Koch);
    koch.single_step();
    assert_eq!(value(&koch), 8.0, "F → F+F--F+F");
    let mut nfa = Session::new(Kind::Nfa);
    nfa.single_step();
    nfa.single_step();
    assert_eq!(value(&nfa), 2.0);
    let order = value(&Session::new(Kind::Boids));
    assert!((0.0..=1.0).contains(&order));
    assert_eq!(value(&Session::new(Kind::Ants)), 0.0);
}
