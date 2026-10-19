//! Validation of loaded model invariants before replacement.
use super::super::session::Session;
pub(super) fn session(s: &Session) -> Result<(), String> {
    s.settings.validate(s.kind)?;
    if s.input.chars().count() > 256
        || !s.elapsed.is_finite()
        || !(0.0..=0.5).contains(&s.elapsed)
        || s.ticks > u64::MAX - 10000
    {
        return Err("Invalid input, clock or tick counter".into());
    }
    Ok(())
}
