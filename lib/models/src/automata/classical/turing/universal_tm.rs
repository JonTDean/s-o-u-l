//! A simple example Turing machine (not truly universal, but illustrative).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction { Left, Right }

/// A single transition rule for a Turing machine.
pub struct TMTransition {
    pub write: char,
    pub new_state: usize,
    pub direction: Direction,
}

/// A deterministic Turing machine with a single tape (semi-infinite to the right).
pub struct TuringMachine {
    pub tape: Vec<char>,
    pub head: usize,
    pub state: usize,
    pub accept_state: usize,
    pub reject_state: usize,
    pub transitions: std::collections::HashMap<(usize, char), TMTransition>,
}

impl TuringMachine {
    /// Step the Turing machine by one transition. Returns false if a halting state is reached.
    pub fn step(&mut self) -> bool {
        if self.state == self.accept_state || self.state == self.reject_state {
            return false;
        }
        // Read current tape symbol (blank if head is beyond tape end)
        let current_symbol = if self.head < self.tape.len() {
            self.tape[self.head]
        } else {
            ' ' // blank symbol
        };
        if let Some(trans) = self.transitions.get(&(self.state, current_symbol)) {
            // Write symbol
            if self.head < self.tape.len() {
                self.tape[self.head] = trans.write;
            } else {
                // extend tape if head is at new cell
                self.tape.push(trans.write);
            }
            // Move head
            match trans.direction {
                Direction::Left => {
                    if self.head > 0 {
                        self.head -= 1;
                    } else {
                        // extend tape on left (unbounded to left) by adding blank at front
                        self.tape.insert(0, ' ');
                        // head stays at 0
                    }
                }
                Direction::Right => {
                    self.head += 1;
                }
            }
            // Update state
            self.state = trans.new_state;
            true
        } else {
            // No transition defined for this state/symbol (implicit reject)
            self.state = self.reject_state;
            false
        }
    }

    /// Runs the Turing machine until halt (accept or reject) or until max_steps exceeded.
    pub fn run(&mut self, max_steps: usize) -> bool {
        for _ in 0..max_steps {
            if !self.step() {
                break;
            }
        }
        // Return true if accepted
        self.state == self.accept_state
    }
}

/// Constructs a simple Turing machine that replaces all 'a' characters with 'X' then halts.
pub fn example_tm_replace_a() -> TuringMachine {
    // States: 0 = initial, 1 = accept
    let accept = 1;
    let reject = 2;
    let mut transitions = std::collections::HashMap::new();
    // In state 0:
    //  - on 'a': write 'X', stay in 0, move Right
    //  - on blank: write blank, go to accept state (1), move Left (head goes back one)
    transitions.insert((0, 'a'), TMTransition { write: 'X', new_state: 0, direction: Direction::Right });
    transitions.insert((0, ' ',), TMTransition { write: ' ', new_state: accept, direction: Direction::Left });
    TuringMachine {
        tape: Vec::new(),
        head: 0,
        state: 0,
        accept_state: accept,
        reject_state: reject,
        transitions,
    }
}
