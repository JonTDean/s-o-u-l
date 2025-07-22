//! Lindenmayer System (L-system) string rewriting.

use std::collections::HashMap;

/// Applies one iteration of L-system rewriting on the input string using the given production rules.
pub fn lsystem_step(input: &str, rules: &HashMap<char, String>) -> String {
    let mut output = String::new();
    for ch in input.chars() {
        if let Some(replacement) = rules.get(&ch) {
            output.push_str(replacement);
        } else {
            output.push(ch);
        }
    }
    output
}

/// Generates the L-system string after n iterations.
pub fn lsystem_iterate(mut axiom: String, rules: &HashMap<char, String>, iterations: u32) -> String {
    for _ in 0..iterations {
        axiom = lsystem_step(&axiom, rules);
    }
    axiom
}

/// Example: Koch curve L-system rules (F -> F+F--F+F).
pub fn koch_curve(iterations: u32) -> String {
    let mut rules = HashMap::new();
    rules.insert('F', String::from("F+F--F+F"));
    lsystem_iterate(String::from("F"), &rules, iterations)
}
