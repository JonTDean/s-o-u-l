//! Compare drawer: two CPU sessions side by side, synchronised stepping and isolated exports.
use crate::{
    lab::{Effects, LabData, stage::painters, stage::view},
    theme::{
        tokens::*,
        widgets::{margin_section, mono, note, pill, serif, serif_italic},
    },
};
use bevy::prelude::Resource;
use bevy_egui::egui::{self, Sense, Ui, vec2};
use engine_core::automata::channel::{ChannelAction, ChannelCommand, ChannelId, Status};
use models::workbench::{experiment::metrics::describe, session::Session};

/// Selected pair (by channel), export folder, status line and running export.
#[derive(Resource)]
pub struct Comparison {
    /// Left channel; `None` once closed until the viewer picks again.
    pub left: Option<ChannelId>,
    /// Right channel.
    pub right: Option<ChannelId>,
    /// Export output folder.
    pub folder: String,
    /// Last result or error.
    pub status: String,
    /// Background export worker.
    pub job: Option<std::thread::JoinHandle<Result<(), String>>>,
}

impl Default for Comparison {
    fn default() -> Self {
        Self {
            left: None,
            right: None,
            folder: "soul-export".into(),
            status: String::new(),
            job: None,
        }
    }
}

impl Comparison {
    /// Session indices of the chosen pair. With nothing chosen the first two sessions are
    /// taken; a side whose channel has closed is cleared rather than replaced.
    pub fn pair(&mut self, sessions: &[Session]) -> Option<(usize, usize)> {
        if self.left.is_none() && self.right.is_none() && sessions.len() >= 2 {
            (self.left, self.right) = (Some(sessions[0].channel), Some(sessions[1].channel));
        }
        let find =
            |id: Option<ChannelId>| id.and_then(|id| sessions.iter().position(|s| s.channel == id));
        let (left, right) = (find(self.left), find(self.right));
        if left.is_none() {
            self.left = None;
        }
        if right.is_none() {
            self.right = None;
        }
        left.zip(right)
    }
}

/// Why a pair cannot step together, if it cannot.
pub fn pair_error(sessions: &[Session], left: usize, right: usize) -> Option<&'static str> {
    if left == right || left >= sessions.len() || right >= sessions.len() {
        return Some("Choose two different sessions");
    }
    let halted = [left, right]
        .iter()
        .any(|i| sessions[*i].status() == Status::Halted);
    halted.then_some("Both sessions must be able to advance")
}

/// Drawer body.
pub fn body(
    ui: &mut Ui,
    state: &mut Comparison,
    saved_path: &str,
    data: &LabData,
    fx: &mut Effects,
) {
    let sessions = &data.bench.sessions;
    margin_section(ui, "compare", true, |ui| {
        if sessions.len() < 2 {
            note(ui, "Open two CPU specimens to compare them.", false);
            return;
        }
        let pair = state.pair(sessions);
        for (salt, side) in [("left", &mut state.left), ("right", &mut state.right)] {
            let shown = side
                .and_then(|id| sessions.iter().position(|s| s.channel == id))
                .map_or("choose a session".into(), |i| {
                    format!("{} · {}", i + 1, sessions[i].title())
                });
            egui::ComboBox::from_id_salt(("lab.compare", salt))
                .selected_text(shown)
                .show_ui(ui, |ui| {
                    for (i, s) in sessions.iter().enumerate() {
                        ui.selectable_value(
                            side,
                            Some(s.channel),
                            format!("{} · {}", i + 1, s.title()),
                        );
                    }
                });
        }
        let Some((left, right)) = pair else {
            note(ui, "A compared channel closed; choose a session.", true);
            return;
        };
        if pill(ui, "Pause and step both", true, true).clicked() {
            state.status = match pair_error(sessions, left, right) {
                Some(error) => error.into(),
                None => {
                    for index in [left, right] {
                        let id = sessions[index].channel;
                        fx.commands.push(ChannelCommand {
                            id,
                            action: ChannelAction::Pause(true),
                        });
                        fx.commands.push(ChannelCommand {
                            id,
                            action: ChannelAction::Step,
                        });
                    }
                    "Both sessions advanced one transition".into()
                }
            };
        }
        ui.columns(2, |columns| {
            for (column, index) in columns.iter_mut().zip([left, right]) {
                thumbnail(column, &sessions[index]);
            }
        });
    });
    super::exports::show(ui, state, &data.bench, saved_path);
}

fn thumbnail(ui: &mut Ui, session: &Session) {
    ui.label(serif(session.title()).color(INK));
    ui.label(mono(format!("tick {}", session.ticks)).size(size::LABEL + 1.0));
    let side = ui.available_width().min(130.0);
    let (rect, _) = ui.allocate_exact_size(vec2(side, side), Sense::hover());
    ui.painter().rect_filled(rect, 4.0, BG);
    let content = painters::content_size(&session.model);
    let fit = view::fit(content, rect, view::StageView::default());
    let painter = ui.painter_at(rect);
    painters::paint(
        &painter,
        &fit,
        &session.model,
        session.kind,
        session.settings.colour,
    );
    ui.label(serif_italic(describe(session)).size(size::LABEL + 1.0));
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::workbench::catalog::Kind;

    #[test]
    fn the_pair_follows_its_channels_when_tabs_close() {
        use engine_core::automata::channel::ChannelId;
        let session = |kind, id| {
            let mut s = Session::new(kind);
            s.channel = ChannelId(id);
            s
        };
        let mut sessions = vec![
            session(Kind::Life, 1),
            session(Kind::Rule30, 2),
            session(Kind::Rule110, 3),
        ];
        let mut state = Comparison::default();
        assert_eq!(
            state.pair(&sessions),
            Some((0, 1)),
            "defaults to the first two"
        );
        state.right = Some(ChannelId(3));
        sessions.remove(0); // close the Life tab
        assert_eq!(
            state.pair(&sessions),
            None,
            "a closed channel is not silently replaced"
        );
        assert_eq!(state.left, None);
        state.left = Some(ChannelId(2));
        assert_eq!(
            state.pair(&sessions),
            Some((0, 1)),
            "Rule 30 and Rule 110 by channel"
        );
    }

    #[test]
    fn pairs_must_be_distinct_and_able_to_advance() {
        let sessions = vec![Session::new(Kind::Life), Session::new(Kind::Rule30)];
        assert_eq!(
            pair_error(&sessions, 0, 0),
            Some("Choose two different sessions")
        );
        assert_eq!(
            pair_error(&sessions, 0, 5),
            Some("Choose two different sessions")
        );
        assert_eq!(pair_error(&sessions, 0, 1), None);
    }
}
