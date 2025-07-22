//! Root **engine** plug‑in – registers events and sub‑systems.
use bevy::prelude::*;

use crate::events::AutomataCommand;

#[cfg(feature = "gpu-compute")]
use crate::gpu::GpuAutomataComputePlugin;

pub struct EnginePlugin;
impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AutomataCommand>();

        // Conditionally enable GPU compute
        #[cfg(feature = "gpu-compute")]
        {
            let allow_gpu = app
                .world
                .get_resource::<RuntimeFlags>()
                .map_or(true, |f| f.gpu_enabled);

            if allow_gpu {
                app.add_plugins(GpuAutomataComputePlugin);
            } else {
                warn!("GPU compute suppressed by RuntimeFlags ⇒ CPU stepper remains active");
            }
        }
    }
}