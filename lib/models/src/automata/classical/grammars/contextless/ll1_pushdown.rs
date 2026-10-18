//! Simple pushdown automata examples (LL(1) parsing or balanced bracket recognition).

/// Checks if a string is in the language L = { a^n b^n | n >= 0 } using a pushdown stack.
/// All `a`'s must come before `b`'s and the counts must match.
pub fn is_a_n_b_n(input: &str) -> bool {
    let mut stack: Vec<char> = Vec::new();
    for ch in input.chars() {
        if ch == 'a' {
            stack.push('A');
        } else if ch == 'b' {
            // Pop an A for each b
            if stack.pop() != Some('A') {
                return false;
            }
        } else {
            // invalid character
            return false;
        }
    }
    // Accept if stack is exactly empty (all a's matched by b's)
    stack.is_empty()
}

/// Checks if a string of parentheses is balanced (every '(' has a matching ')').
pub fn is_balanced_parens(input: &str) -> bool {
    let mut stack: Vec<char> = Vec::new();
    for ch in input.chars() {
        if ch == '(' {
            stack.push(ch);
        } else if ch == ')' {
            if stack.pop() != Some('(') {
                return false;
            }
        } else {
            // ignore any non-paren characters (or treat as error)
            continue;
        }
    }
    stack.is_empty()
}
