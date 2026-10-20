use super::config::{Pattern, Settings};
use crate::workbench::catalog::Kind;
impl Settings {
    /// Reject nonfinite, incompatible or excessive settings without mutation.
    pub fn validate(&self, kind: Kind) -> Result<(), &'static str> {
        self.world.validate(kind)?;
        if let Some(program) = &self.program {
            if kind != Kind::Life {
                return Err("Authored XY rules require a Life cellular channel");
            }
            program.validate()?;
        }
        let bounded = |value: f32, low, high| value.is_finite() && (low..=high).contains(&value);
        if !bounded(self.rate_hz, 1.0, 60.0) || self.birth > 511 || self.survival > 511 {
            return Err("Rate must be 1..60; Life masks must contain only neighbor counts 0..8");
        }
        if ![
            self.cohesion,
            self.alignment,
            self.separation,
            self.evaporation,
            self.exploration,
        ]
        .into_iter()
        .all(|v| bounded(v, 0.0, 1.0))
            || !bounded(self.separation_distance, 0.1, 32.0)
            || !bounded(self.deposit, 0.0, 10.0)
            || !(1..=256).contains(&self.agent_count)
        {
            return Err("Invalid flocking or ant parameter range");
        }
        let valid_pattern = match kind {
            Kind::Rule30 | Kind::Rule110 => matches!(
                self.pattern,
                Pattern::Default | Pattern::Single | Pattern::Noise
            ),
            Kind::Life => true,
            Kind::Hpp => matches!(self.pattern, Pattern::Default | Pattern::Noise),
            _ => self.pattern == Pattern::Default,
        };
        if !valid_pattern {
            return Err("This seed pattern does not apply to the selected model");
        }
        Ok(())
    }
}
