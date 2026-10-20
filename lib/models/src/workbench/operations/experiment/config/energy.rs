//! Editable two-pool starting model; integer arithmetic makes replay exact.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Energy {
    pub capacity: [u16; 2],
    pub initial: [u16; 2],
    /// Parts per thousand exchanged with the four XY cardinal neighbors.
    pub diffusion: [u16; 2],
    pub decay: [u16; 2],
    pub recharge: [u16; 2],
    pub birth_cost: [u16; 2],
    pub survival_cost: [u16; 2],
}
impl Default for Energy {
    fn default() -> Self {
        Self {
            capacity: [64, 32],
            initial: [32, 16],
            diffusion: [250, 100],
            decay: [1, 1],
            recharge: [3, 2],
            birth_cost: [8, 4],
            survival_cost: [2, 1],
        }
    }
}
impl Energy {
    pub fn validate(&self) -> Result<(), &'static str> {
        for i in 0..2 {
            if !(1..=4096).contains(&self.capacity[i])
                || self.diffusion[i] > 1000
                || [
                    self.initial[i],
                    self.decay[i],
                    self.recharge[i],
                    self.birth_cost[i],
                    self.survival_cost[i],
                ]
                .iter()
                .any(|v| *v > self.capacity[i])
            {
                return Err(
                    "Energy capacities must be 1..4096, diffusion 0..1000, amounts within capacity",
                );
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pool_contract_rejects_every_out_of_range_quantity() {
        Energy::default().validate().unwrap();
        for pool in 0..2 {
            let mut e = Energy::default();
            e.capacity[pool] = 0;
            assert!(e.validate().is_err());
            e.capacity[pool] = 4097;
            assert!(e.validate().is_err());
            e = Energy::default();
            e.diffusion[pool] = 1001;
            assert!(e.validate().is_err());
            for field in [
                "initial",
                "decay",
                "recharge",
                "birth_cost",
                "survival_cost",
            ] {
                let mut value = serde_json::to_value(Energy::default()).unwrap();
                value[field][pool] = serde_json::json!(4097);
                let e: Energy = serde_json::from_value(value).unwrap();
                assert!(e.validate().is_err(), "{field} pool {pool}");
            }
        }
    }
}
