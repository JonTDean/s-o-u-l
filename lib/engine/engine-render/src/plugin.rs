//! engine/plugin.rs – root **engine** plug-in.

use bevy::prelude::*;
use engine_core::events::AutomataCommand;
use crate::render::camera::systems::CameraManagerPlugin;

#[cfg(feature = "gpu-compute")]
use engine_gpu::GpuAutomataComputePlugin;

#[cfg(feature = "gpu-compute")]
use engine_core::systems::state::resources::RuntimeFlags;

pub struct EngineRendererPlugin;

impl Plugin for EngineRendererPlugin {
    fn build(&self, app: &mut App) {
        /* 1 ░ global events */
        app.add_event::<AutomataCommand>();

        /* 2 ░ camera stack */
        app.add_plugins(CameraManagerPlugin);

        /* 3 ░ optional GPU compute back-end */
        #[cfg(feature = "gpu-compute")]
        {
            let allow_gpu = app
                .world()
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
