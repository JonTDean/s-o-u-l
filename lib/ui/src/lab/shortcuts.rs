//! Lab keyboard map; camera bindings (arrows, WASD, Z/X, freecam Space) take precedence.
use super::{
    Effects,
    state::{Drawer, LabMode, LabState},
};
use bevy_egui::egui::Key;
use engine_core::automata::channel::{ChannelAction, ChannelCommand, Channels, Status};

/// Lab actions reachable from the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shortcut {
    /// Space: run or pause the selected channel.
    RunPause,
    /// Period: one step while paused.
    Step,
    /// R: reset the selected channel.
    Reset,
    /// 1–8: select the nth tab.
    Select(usize),
    /// P: toggle presenter mode.
    Presenter,
    /// N: toggle the inspector.
    Inspector,
    /// L: toggle the library drawer.
    Library,
    /// Escape: close the drawer or leave presenter.
    Back,
}

/// Key → shortcut. Arrow keys, WASD and Z/X stay with the camera.
pub fn map(key: Key) -> Option<Shortcut> {
    const DIGITS: [Key; 8] = [
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
    ];
    Some(match key {
        Key::Space => Shortcut::RunPause,
        Key::Period => Shortcut::Step,
        Key::R => Shortcut::Reset,
        Key::P => Shortcut::Presenter,
        Key::N => Shortcut::Inspector,
        Key::L => Shortcut::Library,
        Key::Escape => Shortcut::Back,
        other => Shortcut::Select(DIGITS.iter().position(|d| *d == other)?),
    })
}

/// Shortcuts for this frame's pressed keys; none while a text field has focus.
pub fn collect(keys: &[Key], typing: bool) -> Vec<Shortcut> {
    if typing {
        Vec::new()
    } else {
        keys.iter().filter_map(|k| map(*k)).collect()
    }
}

/// Apply one shortcut; returns whether the lab consumed it.
pub fn apply(
    shortcut: Shortcut,
    lab: &mut LabState,
    channels: &Channels,
    freecam: bool,
    fx: &mut Effects,
) -> bool {
    let selected = channels.selected_row();
    let mut send = |action| match selected {
        Some(row) => {
            fx.commands.push(ChannelCommand { id: row.id, action });
            true
        }
        None => false,
    };
    match shortcut {
        Shortcut::RunPause if freecam => false,
        Shortcut::RunPause => match selected.map(|r| r.status) {
            Some(Status::Running) => send(ChannelAction::Pause(true)),
            Some(Status::Paused) => send(ChannelAction::Pause(false)),
            _ => false,
        },
        Shortcut::Step if selected.is_some_and(|r| r.status == Status::Paused) => {
            send(ChannelAction::Step)
        }
        Shortcut::Step => false,
        Shortcut::Reset => send(ChannelAction::Reset),
        Shortcut::Select(index) => match channels.rows.get(index) {
            Some(row) => {
                fx.commands.push(ChannelCommand {
                    id: row.id,
                    action: ChannelAction::Select,
                });
                true
            }
            None => false,
        },
        Shortcut::Presenter => {
            lab.mode = match lab.mode {
                LabMode::Presenter => LabMode::Workbench,
                LabMode::Workbench => LabMode::Presenter,
            };
            true
        }
        Shortcut::Inspector => {
            lab.inspector_open = !lab.inspector_open;
            true
        }
        Shortcut::Library => {
            lab.toggle_drawer(Drawer::Library);
            true
        }
        Shortcut::Back if lab.settings_open => {
            lab.settings_open = false;
            true
        }
        Shortcut::Back if lab.drawer.is_some() => {
            lab.drawer = None;
            true
        }
        Shortcut::Back if lab.mode == LabMode::Presenter => {
            lab.mode = LabMode::Workbench;
            true
        }
        Shortcut::Back => false,
    }
}
