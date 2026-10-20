//! Stage rectangle → viewport mapping, input gates, framing and camera activity.
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
    assert!(physical_viewport(&rect(Vec2::ZERO, Vec2::splat(100.0), 1.0), UVec2::ZERO).is_none());
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

fn stage_app(rect: StageRect) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(rect)
        .add_systems(Update, apply_stage);
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let camera = app.world_mut().spawn((StageCamera, Camera::default())).id();
    (app, camera)
}

#[test]
fn hidden_world_keeps_stage_cameras_running_in_one_pixel() {
    let world_hidden = StageRect {
        world_visible: false,
        ..rect(Vec2::new(32.0, 28.0), Vec2::new(900.0, 600.0), 1.0)
    };
    let (mut app, camera) = stage_app(world_hidden);
    app.update();
    let camera = app.world().get::<Camera>(camera).unwrap();
    assert!(
        camera.is_active,
        "GPU channels step in the Core3d graph, so it must keep running"
    );
    let viewport = camera.viewport.clone().unwrap();
    assert_eq!(viewport.physical_size, UVec2::ONE);
    assert_eq!(viewport.physical_position, UVec2::new(32, 28));
}

#[test]
fn stage_applies_only_while_a_world_is_shown() {
    use engine_core::prelude::AppState;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<AppState>();
    let shown = |app: &mut App| app.world_mut().run_system_cached(stage_active).unwrap();
    assert!(!shown(&mut app), "main menu");
    for state in [AppState::InGame, AppState::Paused] {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(state);
        app.update();
        assert!(shown(&mut app), "{state:?}");
    }
}
