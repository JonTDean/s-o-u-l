//! engine/plugin.rs – root **engine** plug‑in.
use bevy::prelude::*;
use engine_core::events::AutomataCommand;
use crate::render::camera::systems::CameraManagerPlugin;  

#[cfg(feature = "gpu-compute")]
use crate::gpu::GpuAutomataComputePlugin;

pub struct EngineRendererPlugin;

impl Plugin for EngineRendererPlugin {
    fn build(&self, app: &mut App) {
        // global events ----------------------------------------------------
        app.add_event::<AutomataCommand>();

        // world‑camera / zoom‑pan stack ------------------------------------
        app.add_plugins(CameraManagerPlugin); 

        // Optional GPU compute --------------------------------------------
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
