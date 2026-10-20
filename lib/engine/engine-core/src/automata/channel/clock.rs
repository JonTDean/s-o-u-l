//! Integer pacing clock converting fixed simulation steps into channel ticks.
use bevy::prelude::Component;

/// Converts fixed `SimulationStep`s into channel ticks exactly.
///
/// Integer carry avoids the float drift that would drop ticks when the
/// channel rate equals the base rate (60 of 60 Hz must pass through).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepClock {
    carry: u64,
}

impl StepClock {
    /// Ticks due after `steps` base steps at `base_hz`, for a channel at `rate_hz`.
    pub fn due(&mut self, steps: u64, rate_hz: u32, base_hz: u32) -> u64 {
        let base = u64::from(base_hz.max(1));
        let total = self.carry + steps * u64::from(rate_hz);
        self.carry = total % base;
        total / base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_clock_passes_through_at_base_rate() {
        let mut clock = StepClock::default();
        let total: u64 = (0..1000).map(|_| clock.due(1, 60, 60)).sum();
        assert_eq!(total, 1000);
    }

    #[test]
    fn step_clock_divides_exactly_with_integer_carry() {
        let mut clock = StepClock::default();
        let total: u64 = (0..60).map(|_| clock.due(1, 10, 60)).sum();
        assert_eq!(total, 10);
        assert_eq!(clock.due(5, 10, 60), 0, "5/6 of a tick carried");
        assert_eq!(clock.due(1, 10, 60), 1);
    }
}
