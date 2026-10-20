//! Lab shortcut behaviour: key map, selected-channel commands, Escape and toggles.
use super::{
    Effects,
    shortcuts::*,
    state::{LabMode, LabState},
};
use crate::lab::state::Drawer;
use bevy::prelude::default;
use bevy_egui::egui::Key;
use engine_core::automata::channel::{Backend, ChannelId, ChannelRow, ChannelSettings};
use engine_core::automata::channel::{ChannelAction, Channels, Status};

fn channels(status: Status) -> Channels {
    let row = |id| ChannelRow {
        id: ChannelId(id),
        kind: "life:conway".into(),
        backend: Backend::Cpu,
        status,
        tick: 0,
        epoch: 0,
        settings: ChannelSettings::new(10.0),
    };
    Channels {
        rows: vec![row(0), row(4)],
        selected: Some(ChannelId(0)),
    }
}

#[test]
fn keys_map_to_shortcuts() {
    assert_eq!(map(Key::Space), Some(Shortcut::RunPause));
    assert_eq!(map(Key::Period), Some(Shortcut::Step));
    assert_eq!(map(Key::Num2), Some(Shortcut::Select(1)));
    assert_eq!(map(Key::ArrowRight), None, "arrows belong to the camera");
    assert_eq!(
        collect(&[Key::R, Key::P], true),
        [],
        "typing suppresses shortcuts"
    );
    assert_eq!(collect(&[Key::R], false), [Shortcut::Reset]);
}

#[test]
fn run_pause_step_reset_target_the_selected_channel() {
    let mut lab = LabState::default();
    let mut fx = Effects::default();
    let paused = channels(Status::Paused);
    assert!(apply(Shortcut::RunPause, &mut lab, &paused, false, &mut fx));
    assert!(matches!(fx.commands[0].action, ChannelAction::Pause(false)));
    assert!(apply(Shortcut::Step, &mut lab, &paused, false, &mut fx));
    assert!(
        !apply(
            Shortcut::Step,
            &mut lab,
            &channels(Status::Running),
            false,
            &mut fx
        ),
        "no step while running"
    );
    assert!(!apply(
        Shortcut::RunPause,
        &mut lab,
        &channels(Status::Halted),
        false,
        &mut fx
    ));
    assert!(
        !apply(Shortcut::RunPause, &mut lab, &paused, true, &mut fx),
        "freecam owns Space"
    );
    assert!(apply(
        Shortcut::Select(1),
        &mut lab,
        &paused,
        false,
        &mut fx
    ));
    assert_eq!(fx.commands.last().unwrap().id, ChannelId(4));
    assert!(!apply(
        Shortcut::Select(5),
        &mut lab,
        &paused,
        false,
        &mut fx
    ));
}

#[test]
fn escape_is_swallowed_only_when_lab_consumes_it() {
    let mut lab = LabState {
        drawer: Some(Drawer::Library),
        ..default()
    };
    let mut fx = Effects::default();
    let c = channels(Status::Paused);
    assert!(lab.consumes_escape());
    assert!(apply(Shortcut::Back, &mut lab, &c, false, &mut fx));
    assert_eq!(lab.drawer, None);
    lab.mode = LabMode::Presenter;
    assert!(apply(Shortcut::Back, &mut lab, &c, false, &mut fx));
    assert_eq!(lab.mode, LabMode::Workbench);
    assert!(
        !lab.consumes_escape(),
        "plain Escape falls through to the pause menu"
    );
    assert!(!apply(Shortcut::Back, &mut lab, &c, false, &mut fx));
}

#[test]
fn toggles() {
    let mut lab = LabState {
        drawer: None,
        ..default()
    };
    let mut fx = Effects::default();
    let c = channels(Status::Paused);
    apply(Shortcut::Presenter, &mut lab, &c, false, &mut fx);
    assert_eq!(lab.mode, LabMode::Presenter);
    apply(Shortcut::Inspector, &mut lab, &c, false, &mut fx);
    assert!(!lab.inspector_open);
    apply(Shortcut::Library, &mut lab, &c, false, &mut fx);
    assert_eq!(lab.drawer, Some(Drawer::Library));
}

#[test]
fn frame_input_reads_keys_without_relocking_the_context() {
    use bevy_egui::egui::{Context, Event, Modifiers, RawInput};
    let ctx = Context::default();
    let key = |key| Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    let input = RawInput {
        events: vec![key(Key::Space), key(Key::R)],
        ..Default::default()
    };
    let mut read = None;
    let _ = ctx.run(input, |ctx| read = Some(super::frame_input(ctx)));
    assert_eq!(read, Some((vec![Key::Space, Key::R], false)));
}

#[test]
fn legacy_ui_modules_are_gone() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    for gone in [
        "presentation",
        "styles",
        "components/banners",
        "components/menus/automata",
    ] {
        assert!(
            !std::path::Path::new(root).join(gone).exists(),
            "{gone} should be removed"
        );
    }
}
