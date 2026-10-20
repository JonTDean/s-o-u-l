use super::machine::Symbolic;
use crate::workbench::catalog::Kind;
mod languages;
mod machines;

fn run(kind: Kind, input: &str) -> Symbolic {
    let mut machine = Symbolic::new(kind, input);
    for _ in 0..1000 {
        if machine.halted() {
            return machine;
        }
        machine.step();
    }
    panic!("machine did not halt: {kind:?} {input:?}");
}
