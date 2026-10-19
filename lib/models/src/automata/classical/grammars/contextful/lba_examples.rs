//! Examples of linear-bounded automata (LBA) for context-sensitive languages.

/// Checks whether a string is in the language L = { a^n b^n c^n | n > 0 } (equal numbers of a, b, c in order).
/// This compact predicate cross-checks the workbench's incremental bounded-tape machine.
pub fn is_a_n_b_n_c_n(s: &str) -> bool {
    // A bounded tape recognizes ordered nonempty blocks, not interleaved triples.
    let bytes = s.as_bytes();
    let count = bytes.iter().take_while(|&&c| c == b'a').count();
    count > 0
        && bytes.len() == count * 3
        && bytes[count..count * 2].iter().all(|&c| c == b'b')
        && bytes[count * 2..].iter().all(|&c| c == b'c')
}
