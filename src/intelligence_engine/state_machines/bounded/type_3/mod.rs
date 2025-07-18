use super::{MachineFactory, MachineInstance};

pub mod class;
pub mod plugin;

// collect names + factory fns
pub fn register(list: &mut Vec<(&'static str, MachineFactory)>) {
    use class::acceptors::rules::dfa;
    fn build_dfa() -> MachineInstance {
        let m = dfa::dfa();                       // your FiniteStateMachine<u8,char,_>
        // evaluator closure so UI can run the machine once per tick if it wants
        MachineInstance::Type3Dfa(Box::new(move || m.accepts("aaa".chars())))
    }
    list.push(("Type-3 : DFA a⁺", build_dfa));
}
