use bevy::prelude::*;
use engine_core::systems::simulation::{FixedStepConfig, SimAccumulator};

/// `alpha` ∈ [0, 1) — how far the *render* frame is into the next tick.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct RenderInterpolator { pub alpha: f32 }

/// Calculate `alpha` once per frame **after** FixedUpdate systems ran.
pub fn update_alpha(
    acc: Res<SimAccumulator>,
    cfg: Res<FixedStepConfig>,
    mut ri: ResMut<RenderInterpolator>,
) {
    let dt = cfg.dt.as_secs_f64();
    ri.alpha = (acc.accum / dt) as f32;          // safe: accum < dt by design
}
