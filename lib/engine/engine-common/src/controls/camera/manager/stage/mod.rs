//! The lab's stage rectangle drives every stage camera's viewport, activity and input.
use bevy::{prelude::*, render::camera::Viewport, window::PrimaryWindow};
use engine_core::automata::{
    GpuGridSlice,
    channel::{ChannelId, Channels},
};
use engine_core::prelude::AppState;

/// Stage area in logical egui points, published by the lab shell every frame.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct StageRect {
    /// False until a lab shell publishes; cameras are left untouched while false.
    pub managed: bool,
    /// Top-left corner in logical points.
    pub min: Vec2,
    /// Size in logical points.
    pub size: Vec2,
    /// Physical pixels per logical point (`egui::Context::pixels_per_point`).
    pub scale: f32,
    /// Pointer is over the stage and not over a panel, drawer or popup.
    pub hovered: bool,
    /// A text field has keyboard focus.
    pub typing: bool,
    /// The selected channel renders in the 3-D world (GPU); false deactivates stage cameras.
    pub world_visible: bool,
}

/// Marks 3-D cameras that render into the stage.
#[derive(Component, Default, Debug)]
pub struct StageCamera;

/// Distance kept from a framed instance by perspective stage cameras.
const FRAME_DISTANCE: f32 = 260.0;

/// Physical viewport for the stage, clamped into the window; `None` when degenerate.
pub fn physical_viewport(rect: &StageRect, window: UVec2) -> Option<Viewport> {
    let min = (rect.min * rect.scale).round().max(Vec2::ZERO);
    let max = ((rect.min + rect.size) * rect.scale)
        .round()
        .min(window.as_vec2());
    let size = (max - min).max(Vec2::ZERO);
    if size.x < 1.0 || size.y < 1.0 {
        return None;
    }
    Some(Viewport {
        physical_position: min.as_uvec2(),
        physical_size: size.as_uvec2(),
        ..default()
    })
}

/// Fit stage cameras to the stage.
///
/// GPU channels advance inside the `Core3d` render graph, which only runs for active 3-D
/// cameras. While a CPU channel is shown the cameras therefore stay active in a 1 px
/// viewport at the stage corner, beneath the opaque CPU stage, so every GPU channel keeps
/// stepping whatever tab is selected.
pub fn apply_stage(
    rect: Res<StageRect>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut cameras: Query<&mut Camera, With<StageCamera>>,
) {
    let Some(window) = window else { return };
    if !rect.managed {
        return;
    }
    let viewport = physical_viewport(&rect, window.physical_size()).map(|mut v| {
        if !rect.world_visible {
            v.physical_size = UVec2::ONE;
        }
        v
    });
    for mut camera in &mut cameras {
        camera.is_active = viewport.is_some();
        camera.viewport = viewport.clone();
    }
}

/// Stage cameras follow the lab only while a world is on screen (in game or paused).
pub fn stage_active(state: Option<Res<State<AppState>>>) -> bool {
    state.is_some_and(|s| matches!(s.get(), AppState::InGame | AppState::Paused))
}

/// When the selection changes to a GPU channel, look at its volume's centre.
pub fn frame_selected(
    channels: Res<Channels>,
    slices: Query<(&ChannelId, &GpuGridSlice)>,
    mut cameras: Query<&mut Transform, With<StageCamera>>,
    mut framed: Local<Option<ChannelId>>,
) {
    let selected = channels.selected;
    if selected == *framed {
        return;
    }
    let Some((_, slice)) = slices.iter().find(|(id, _)| Some(**id) == selected) else {
        return;
    };
    *framed = selected;
    // Atlas texels map to 4 world units, matching the voxel renderer.
    let size = Vec3::new(
        slice.size.x as f32,
        slice.size.y as f32,
        slice.depth.max(1) as f32,
    ) * 4.0;
    let centre = Vec3::new(
        slice.offset.x as f32 * 4.0,
        slice.offset.y as f32 * 4.0,
        0.0,
    ) + size * 0.5;
    for mut transform in &mut cameras {
        let forward = transform.forward();
        transform.translation = centre - forward * FRAME_DISTANCE;
    }
}

/// Pointer-driven camera input only while the pointer is over the stage.
pub fn stage_pointer_allowed(rect: Option<Res<StageRect>>) -> bool {
    rect.is_none_or(|r| !r.managed || r.hovered)
}

/// Keyboard camera input only while no text field is focused.
pub fn stage_keys_allowed(rect: Option<Res<StageRect>>) -> bool {
    rect.is_none_or(|r| !r.managed || !r.typing)
}

#[cfg(test)]
mod tests;
