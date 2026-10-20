//! Portable checkpoint of completed GPU work, independent of atlas allocation.
use simulation_kernel::core::lenia::{LeniaParams, LeniaSeed};
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
/// Completed portable density state and its evolution settings.
pub struct SavedLenia {
    /// X, Y and Z extents.
    pub size: [u32; 3],
    /// Number of completed transitions.
    pub tick: u64,
    /// Growth parameters for the next transition.
    pub params: LeniaParams,
    /// Fixture used by Reset.
    pub seed: LeniaSeed,
    /// Whether automatic evolution is suspended.
    pub paused: bool,
    /// Full floating-point checkpoint when continuous XYZ mode is selected.
    #[serde(default)]
    pub continuous: Option<simulation_kernel::core::lenia::continuous::State>,
    /// Density bytes in X-fastest order.
    pub densities: Vec<u8>,
}
impl SavedLenia {
    /// Reject unsupported shapes and unsafe counters before allocation.
    pub fn validate(&self) -> Result<(), &'static str> {
        self.params.validate()?;
        if let Some(state) = &self.continuous {
            state.validate(32768)?;
            if state.bytes() != self.densities {
                return Err("Continuous preview differs from density state");
            }
        }
        if self.size != [64, 64, 8] || self.densities.len() != 32768 || self.tick > u64::MAX - 10000
        {
            return Err("Invalid Lenia checkpoint dimensions, density length or tick");
        }
        Ok(())
    }
}
