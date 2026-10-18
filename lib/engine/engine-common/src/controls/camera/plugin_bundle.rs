//! Umbrella plugin that installs the whole camera stack in the
//! correct order.  Import this **one** plugin and you’re done.

use bevy::prelude::*;

use crate::controls::camera::{
    controller::plugin::CameraControllerPlugin, input, input::plugin::InputFacadePlugin, manager,
    manager::plugin::CameraManagerPlugin,
};

/// Add *all* camera-related plugins & resources in one shot.
///
/// ```no_run
/// use bevy::prelude::*;
/// use engine_common::controls::camera::CameraPluginBundle;
/// use engine_core::{plugin::EngineCorePlugin, prelude::AppState};
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .init_state::<AppState>()
///     .add_plugins(EngineCorePlugin)
///     .add_plugins(CameraPluginBundle)   // ← one-liner
///     .run();
/// ```
pub struct CameraPluginBundle;

impl Plugin for CameraPluginBundle {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // low-level glue, floating-origin hooks, etc.
            CameraManagerPlugin,
            // primary orthographic controller
            CameraControllerPlugin,
        ))
        // lightweight input helpers (zoom, drag, pan, orbit)
        .add_plugins(InputFacadePlugin)
        // make sure shared resources exist early
        .init_resource::<manager::ZoomInfo>()
        .init_resource::<manager::DragState>()
        .init_resource::<manager::CameraMetrics>()
        .init_resource::<input::rotate::OrbitAngles>();
    }
}
