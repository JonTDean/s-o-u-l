//! Independent execution state and shared controls used by UI and tests.
use super::experiment::config::Settings;
use super::{
    agents::{Flock, Foraging},
    catalog::Kind,
    grid::GridSession,
    symbolic::machine::Symbolic,
};

/// State representation follows the model rather than coercing tapes into cells.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Model {
    Grid(GridSession),
    Symbolic(Symbolic),
    Boids(Flock),
    Ants(Box<Foraging>),
}

/// One paused-by-default simulation with a reproducible initial condition.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub kind: Kind,
    pub input: String,
    pub model: Model,
    pub ticks: u64,
    pub paused: bool,
    pub settings: Settings,
    pub elapsed: f32,
}
impl Session {
    /// Describe the active rules, including live edits to catalog defaults.
    pub fn title(&self) -> String {
        match self.kind {
            Kind::Rule30 | Kind::Rule110 => {
                format!("Rule {} · space-time", self.settings.elementary)
            }
            Kind::Life => {
                let digits = |mask: u16| {
                    (0..=8)
                        .filter(|n| mask & (1 << n) != 0)
                        .map(|n| char::from_digit(n, 10).unwrap())
                        .collect::<String>()
                };
                format!(
                    "Life · B{}/S{}",
                    digits(self.settings.birth),
                    digits(self.settings.survival)
                )
            }
            _ => self.kind.title().into(),
        }
    }
    /// Construct the selected model at its deterministic initial state.
    pub fn new(kind: Kind) -> Self {
        let input = kind.input().to_string();
        let settings = Settings::for_kind(kind);
        Self {
            kind,
            model: Self::initial(kind, &input, &settings),
            input,
            ticks: 0,
            paused: true,
            settings,
            elapsed: 0.0,
        }
    }
    fn initial(kind: Kind, input: &str, settings: &Settings) -> Model {
        match kind {
            Kind::Rule30 | Kind::Rule110 | Kind::Life | Kind::Hpp => {
                Model::Grid(GridSession::configured(kind, settings))
            }
            Kind::Boids => Model::Boids(Flock::configured(settings)),
            Kind::Ants => Model::Ants(Box::new(Foraging::configured(settings))),
            _ => Model::Symbolic(Symbolic::new(kind, input)),
        }
    }
    /// Reset keeps the selected input and pauses. Oversized input changes nothing.
    pub fn reset(&mut self) -> Result<(), &'static str> {
        self.settings.validate(self.kind)?;
        if self.input.chars().count() > 256 {
            return Err("Input is limited to 256 symbols");
        }
        self.model = Self::initial(self.kind, &self.input, &self.settings);
        self.ticks = 0;
        self.elapsed = 0.0;
        self.paused = true;
        Ok(())
    }
    /// Validate atomically; seed and population edits require explicit reset.
    pub fn configure(&mut self, settings: Settings, reset: bool) -> Result<(), &'static str> {
        settings.validate(self.kind)?;
        if !reset && !self.settings.same_initial_conditions(&settings) {
            return Err("Seed and population changes require Apply and reset");
        }
        if reset && self.input.chars().count() > 256 {
            return Err("Input is limited to 256 symbols");
        }
        self.settings = settings;
        if reset {
            self.reset()?;
        }
        Ok(())
    }

    /// Preserve sub-tick clock state; cap catch-up work after a stalled frame.
    pub fn elapse(&mut self, seconds: f32) {
        if self.paused || !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        self.elapsed += seconds.min(0.5);
        let interval = 1.0 / self.settings.rate_hz;
        while self.elapsed >= interval && !self.halted() {
            self.step();
            self.elapsed -= interval;
        }
    }

    /// Terminal language states and explicit execution bounds stop automatic ticks.
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
            Model::Grid(grid) => grid.step_with_settings(&self.settings),
            Model::Symbolic(machine) => machine.step(),
            Model::Boids(flock) => flock.step_with_settings(&self.settings),
            Model::Ants(arena) => arena.step_with_settings(&self.settings),
        }
        self.ticks += 1;
        if self.halted() {
            self.paused = true;
        }
    }
    /// Honor a manual step only while paused.
    pub fn single_step(&mut self) {
        if self.paused {
            self.step();
        }
    }
    /// Advance a running session without affecting paused instances.
    pub fn advance(&mut self) {
        if !self.paused {
            self.step();
        }
    }
}
