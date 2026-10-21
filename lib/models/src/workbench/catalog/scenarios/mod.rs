//! Reproducible, paused teaching experiments built from production session contracts.
use crate::workbench::{catalog::Kind, plugin::Workbench, project::Project};
use simulation_kernel::core::boundary::Boundary;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scenario {
    #[default]
    Elementary,
    Boundaries,
    Languages,
}
impl Scenario {
    pub const ALL: [Self; 3] = [Self::Elementary, Self::Boundaries, Self::Languages];
    pub fn title(self) -> &'static str {
        match self {
            Self::Elementary => "Same seed, different elementary rules",
            Self::Boundaries => "Life at a periodic boundary",
            Self::Languages => "Accepted and rejected input",
        }
    }
    /// A fresh deterministic experiment; no ambient random source or existing state.
    pub fn project(self) -> Result<Project, String> {
        let mut bench = Workbench::default();
        match self {
            Self::Elementary => {
                bench.spawn(Kind::Rule30);
                bench.spawn(Kind::Rule110);
            }
            Self::Boundaries => {
                for boundary in [Boundary::Dead, Boundary::Periodic] {
                    bench.spawn(Kind::Life);
                    let session = bench.sessions.last_mut().unwrap();
                    let mut settings = session.settings.clone();
                    settings.world.width = 16;
                    settings.world.height = 16;
                    settings.world.boundary = boundary;
                    settings.world.seed = Some(vec![0; 256]);
                    session.configure(settings, true)?;
                    session.paint(&[[0, 0], [1, 0], [15, 0]], 255, true)?;
                }
            }
            Self::Languages => {
                for input in ["aaabbb", "aaabb"] {
                    bench.spawn(Kind::Pushdown);
                    let session = bench.sessions.last_mut().unwrap();
                    session.input = input.into();
                    session.reset()?;
                }
            }
        }
        Project::capture(&bench, Vec::new())
    }
    pub fn steps(self) -> [&'static str; 4] {
        match self {
            Self::Elementary => [
                "Inspect the identical single-cell starting seeds.",
                "Step both channels. Explain how the local rule changes the next row.",
                "Compare completed fields at equal ticks. Read the error and history.",
                "Export the comparison CSV and save a specimen to repeat the experiment.",
            ],
            Self::Boundaries => [
                "Both worlds contain the same edge-crossing seed. Only the boundary differs.",
                "Step both once. Identify the cells that can see across the periodic seam.",
                "Capture the difference field. Explain the boundary effect using the heatmap.",
                "Export the evidence; reopen a saved specimen and reproduce its next tick.",
            ],
            Self::Languages => [
                "Inspect the two inputs: a^3 b^3 and a^3 b^2. Predict acceptance.",
                "Step both and watch the voxel head and stack. Each pop matches one earlier a.",
                "Continue until both halt. Explain the rejected input's unmatched stack item.",
                "Export the metric trace and save the accepted or rejected specimen.",
            ],
        }
    }
}
#[cfg(test)]
mod tests;
