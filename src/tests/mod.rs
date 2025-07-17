#[cfg(test)]
mod tests {
    use crate::state_engine::state_machines::bounded::type_3::class::acceptors::rules::dfa::dfa;

    #[test]
    fn accepts_positive_number_of_a() {
        let m = dfa();
        assert!(m.accepts("a".chars()));
        assert!(m.accepts("aaaaa".chars()));
        assert!(!m.accepts("".chars()));
        assert!(!m.accepts("ab".chars()));
    }
}
