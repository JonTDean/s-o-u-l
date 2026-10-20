//! The lab's stage rectangle drives every stage camera's viewport, activity and input.
use bevy::{prelude::*, render::camera::Viewport, window::PrimaryWindow};
use engine_core::automata::{
    GpuGridSlice,
    channel::{ChannelId, Channels},
};

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

/// Fit stage cameras to the stage; deactivate them while a CPU channel is shown.
pub fn apply_stage(
    rect: Res<StageRect>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut cameras: Query<&mut Camera, With<StageCamera>>,
) {
    let Some(window) = window else { return };
    if !rect.managed {
        return;
    }
    let viewport = physical_viewport(&rect, window.physical_size());
    for mut camera in &mut cameras {
        camera.is_active = rect.world_visible && viewport.is_some();
        camera.viewport = viewport.clone();
    }
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
mod tests {
    use super::*;

    fn rect(min: Vec2, size: Vec2, scale: f32) -> StageRect {
        StageRect {
            managed: true,
            min,
            size,
            scale,
            world_visible: true,
            ..default()
        }
    }

    #[test]
    fn logical_rect_maps_to_physical_pixels() {
        let v = physical_viewport(
            &rect(Vec2::new(32.0, 28.0), Vec2::new(988.0, 652.0), 2.0),
            UVec2::new(2560, 1440),
        )
        .unwrap();
        assert_eq!(v.physical_position, UVec2::new(64, 56));
        assert_eq!(v.physical_size, UVec2::new(1976, 1304));
    }

    #[test]
    fn viewport_is_clamped_inside_the_window() {
        let v = physical_viewport(
            &rect(Vec2::new(1200.0, 0.0), Vec2::new(500.0, 900.0), 1.0),
            UVec2::new(1280, 720),
        )
        .unwrap();
        assert_eq!(v.physical_size, UVec2::new(80, 720));
    }

    #[test]
    fn degenerate_stage_has_no_viewport() {
        let window = UVec2::new(1280, 720);
        assert!(physical_viewport(&rect(Vec2::ZERO, Vec2::new(0.4, 300.0), 1.0), window).is_none());
        assert!(
            physical_viewport(
                &rect(Vec2::new(1400.0, 0.0), Vec2::splat(100.0), 1.0),
                window
            )
            .is_none()
        );
        assert!(
            physical_viewport(&rect(Vec2::ZERO, Vec2::splat(100.0), 1.0), UVec2::ZERO).is_none()
        );
    }

    #[test]
    fn input_gates_default_open_without_a_lab() {
        assert!(stage_pointer_allowed(None));
        assert!(stage_keys_allowed(None));
    }

    #[test]
    fn frame_selected_centres_on_the_slice_and_ignores_missing_slice() {
        use engine_core::automata::{
            GpuGridSlice,
            channel::{Backend, ChannelId, ChannelRow, ChannelSettings, Channels, Status},
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Channels>()
            .add_systems(Update, frame_selected);
        let camera = app
            .world_mut()
            .spawn((
                StageCamera,
                Transform::from_xyz(0.0, 0.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y),
            ))
            .id();
        let id = ChannelId(3);
        app.world_mut().spawn((
            id,
            GpuGridSlice {
                layer: 0,
                offset: UVec2::new(64, 0),
                size: UVec2::splat(64),
                depth: 8,
                rule: "lenia".into(),
                rule_bits: 0,
            },
        ));
        let row = ChannelRow {
            id,
            kind: "lenia".into(),
            backend: Backend::Gpu,
            status: Status::Paused,
            tick: 0,
            epoch: 0,
            settings: ChannelSettings::new(60.0),
        };
        let mut channels = app.world_mut().resource_mut::<Channels>();
        channels.rows.push(row);
        channels.selected = Some(id);
        app.update();
        let t = app.world().get::<Transform>(camera).unwrap().translation;
        assert!(
            (t.x - 384.0).abs() < 1e-3 && (t.y - 128.0).abs() < 1e-3,
            "{t:?}"
        );
        let mut channels = app.world_mut().resource_mut::<Channels>();
        channels.selected = Some(ChannelId(99));
        app.update();
        assert_eq!(
            app.world().get::<Transform>(camera).unwrap().translation,
            t,
            "missing slice leaves camera alone"
        );
    }
}
