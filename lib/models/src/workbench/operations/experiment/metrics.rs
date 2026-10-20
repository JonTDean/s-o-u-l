//! Model-specific measurements with explicit units.
use crate::workbench::{
    session::{Model, Session},
    symbolic::machine::Symbolic,
};
pub fn describe(session: &Session) -> String {
    match &session.model {
        Model::Grid(g) => {
            let bytes = g.bytes();
            let occupied = bytes.iter().filter(|v| **v != 0).count();
            format!(
                "{} occupied / {} cells ({:.2}%)",
                occupied,
                bytes.len(),
                100.0 * occupied as f64 / bytes.len() as f64
            )
        }
        Model::Boids(f) => format!(
            "{} boids · mean speed {:.3} cells/tick",
            f.boids.len(),
            f.boids.iter().map(|b| b.velocity.length()).sum::<f32>() / f.boids.len().max(1) as f32
        ),
        Model::Ants(a) => format!(
            "{} ants · {} deliveries · {:.2} pheromone",
            a.colony.ants.len(),
            a.deliveries,
            a.colony.pheromone.values().sum::<f32>()
        ),
        Model::Symbolic(m) => match m {
            Symbolic::Finite(m) => {
                format!("Cursor {} · {} active states", m.cursor, m.states.len())
            }
            Symbolic::Pushdown(m) => format!("Cursor {} · stack depth {}", m.cursor, m.stack.len()),
            Symbolic::Bounded(m) => format!("Head {} · {} tape cells", m.head, m.tape.len()),
            Symbolic::Turing(m) => format!(
                "State {} · head {} · {} tape cells",
                m.state,
                m.head,
                m.tape.len()
            ),
            Symbolic::Koch { word, .. } => format!("{} symbols", word.chars().count()),
        },
    }
}
pub fn step_pair(sessions: &mut [Session], left: usize, right: usize) -> Result<(), &'static str> {
    if left == right || left >= sessions.len() || right >= sessions.len() {
        return Err("Choose two different sessions");
    }
    if sessions[left].halted() || sessions[right].halted() {
        return Err("Both sessions must be able to advance");
    }
    for index in [left, right] {
        sessions[index].paused = true;
        sessions[index].step();
    }
    Ok(())
}
