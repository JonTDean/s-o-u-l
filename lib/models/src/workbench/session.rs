//! Independent execution state and shared controls used by UI and tests.
use super::{
    agents::{Flock, Foraging},
    catalog::Kind,
    grid::GridSession,
    symbolic::machine::Symbolic,
};

/// State representation follows the model rather than coercing tapes into cells.
pub enum Model {
    Grid(GridSession),
    Symbolic(Symbolic),
    Boids(Flock),
    Ants(Box<Foraging>),
}

/// One paused-by-default simulation with a reproducible initial condition.
pub struct Session {
    pub kind: Kind,
    pub input: String,
    pub model: Model,
    pub ticks: u64,
    pub paused: bool,
}
impl Session {
    pub fn new(kind: Kind) -> Self {
        let input = kind.input().to_string();
        Self {
            kind,
            model: Self::initial(kind, &input),
            input,
            ticks: 0,
            paused: true,
        }
    }
    fn initial(kind: Kind, input: &str) -> Model {
        match kind {
            Kind::Rule30 | Kind::Rule110 | Kind::Life | Kind::Hpp => {
                Model::Grid(GridSession::new(kind))
            }
            Kind::Boids => Model::Boids(Flock::default()),
            Kind::Ants => Model::Ants(Box::default()),
            _ => Model::Symbolic(Symbolic::new(kind, input)),
        }
    }
    /// Reset keeps the selected input and pauses. Oversized input changes nothing.
    pub fn reset(&mut self) -> Result<(), &'static str> {
        if self.input.chars().count() > 256 {
            return Err("Input is limited to 256 symbols");
        }
        self.model = Self::initial(self.kind, &self.input);
        self.ticks = 0;
        self.paused = true;
        Ok(())
    }
    pub fn halted(&self) -> bool {
        matches!(&self.model, Model::Symbolic(machine) if machine.halted() || self.ticks >= 10000)
    }
    /// One model transition; terminal symbolic states never advance again.
    pub fn step(&mut self) {
        if self.halted() {
            self.paused = true;
            return;
        }
        match &mut self.model {
            Model::Grid(grid) => grid.step(),
            Model::Symbolic(machine) => machine.step(),
            Model::Boids(flock) => flock.step(),
            Model::Ants(arena) => arena.step(),
        }
        self.ticks += 1;
        if self.halted() {
            self.paused = true;
        }
    }
    pub fn single_step(&mut self) {
        if self.paused {
            self.step();
        }
    }
    pub fn advance(&mut self) {
        if !self.paused {
            self.step();
        }
    }
}
