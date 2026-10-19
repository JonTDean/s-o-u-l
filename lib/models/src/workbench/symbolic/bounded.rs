//! Linear-bounded marking machine for nonempty ordered a^n b^n c^n.

/// Each transition moves or marks within the original input tape.
pub struct Bounded {
    pub tape: Vec<char>,
    pub head: usize,
    pub result: Option<bool>,
    phase: u8,
}
impl Bounded {
    /// Allocate exactly the input tape; reject invalid alphabets or block ordering.
    pub fn new(input: &str) -> Self {
        let tape: Vec<char> = input.chars().collect();
        let ordered = !tape.is_empty()
            && tape.iter().all(|c| matches!(c, 'a' | 'b' | 'c'))
            && tape.windows(2).all(|pair| pair[0] <= pair[1]);
        Self {
            tape,
            head: 0,
            phase: 0,
            result: if ordered { None } else { Some(false) },
        }
    }

    /// Perform one bounded scan or mark without changing the tape length.
    pub fn step(&mut self) {
        if self.result.is_some() {
            return;
        }
        if self.phase == 3 {
            if self.head == 0 {
                self.phase = 0;
            } else {
                self.head -= 1;
            }
            return;
        }
        let Some(&symbol) = self.tape.get(self.head) else {
            self.result = Some(false);
            return;
        };
        match (self.phase, symbol) {
            (0, 'A') | (1, 'a' | 'B') | (2, 'b' | 'C') => self.head += 1,
            (0, 'a') => {
                self.tape[self.head] = 'A';
                self.phase = 1;
                self.head += 1;
            }
            (1, 'b') => {
                self.tape[self.head] = 'B';
                self.phase = 2;
                self.head += 1;
            }
            (2, 'c') => {
                self.tape[self.head] = 'C';
                self.phase = 3;
            }
            (0, 'B') => self.result = Some(self.tape.iter().all(|c| matches!(c, 'A' | 'B' | 'C'))),
            _ => self.result = Some(false),
        }
    }
}
