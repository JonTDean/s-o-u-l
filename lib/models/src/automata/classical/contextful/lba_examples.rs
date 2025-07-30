//! Examples of linear-bounded automata (LBA) for context-sensitive languages.

/// Checks whether a string is in the language L = { a^n b^n c^n | n > 0 } (equal numbers of a, b, c in order).
/// This simulates an LBA marking process: each iteration finds an `a`, a subsequent `b`, and a subsequent `c` and marks them.
pub fn is_a_n_b_n_c_n(s: &str) -> bool {
    let n = s.len();
    if n == 0 {
        return false; // require n>0
    }
    if n % 3 != 0 {
        return false; // total length must be 3n for some n
    }
    let mut tape: Vec<char> = s.chars().collect();
    // Mark triples iteratively
    loop {
        // Find the leftmost unmarked 'a'
        let a_index = tape.iter().position(|&ch| ch == 'a');
        if let Some(i) = a_index {
            tape[i] = 'A'; // mark this 'a' as used
            // Find an unmarked 'b' to the right of this 'a'
            if let Some(j) = ((i+1)..n).find(|&j| tape[j] == 'b') {
                tape[j] = 'B'; // mark this 'b'
                // Find an unmarked 'c' to the right of this 'b'
                if let Some(k) = ((j+1)..n).find(|&k| tape[k] == 'c') {
                    tape[k] = 'C'; // mark this 'c'
                } else {
                    return false; // no unused 'c' found for this triple
                }
            } else {
                return false; // no unused 'b' found after an 'a'
            }
        } else {
            // no unmarked 'a' left, break out
            break;
        }
    }
    // If any unmarked b or c remains, then counts were not equal
    if tape.iter().any(|&ch| ch == 'b' || ch == 'c') {
        return false;
    }
    // All triples matched
    true
}
