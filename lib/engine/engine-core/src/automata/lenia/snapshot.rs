//! Portable checkpoint of completed GPU work, independent of atlas allocation.
use simulation_kernel::core::lenia::{LeniaParams, LeniaSeed};
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedLenia {
    pub size: [u32; 3],
    pub tick: u64,
    pub params: LeniaParams,
    pub seed: LeniaSeed,
    pub paused: bool,
    pub densities: Vec<u8>,
}
impl SavedLenia {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.params.validate()?;
        if self.size != [64, 64, 8] || self.densities.len() != 32768 || self.tick > u64::MAX - 10000
        {
            return Err("Invalid Lenia checkpoint dimensions, density length or tick");
        }
        Ok(())
    }
}
