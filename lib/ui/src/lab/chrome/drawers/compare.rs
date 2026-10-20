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
use engine_core::automata::channel::{ChannelAction, ChannelCommand, Status};
use models::workbench::{
    experiment::{export::write_bundle, metrics::describe},
    plugin::Workbench,
    project::{self, Project},
    session::Session,
};

/// Selected pair (session indices), export folder, status line and running export.
#[derive(Resource)]
pub struct Comparison {
    /// Left session index.
    pub left: usize,
    /// Right session index.
    pub right: usize,
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
            left: 0,
            right: 1,
            folder: "soul-export".into(),
            status: String::new(),
            job: None,
        }
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
        let last = sessions.len() - 1;
        (state.left, state.right) = (state.left.min(last), state.right.min(last));
        for (salt, index) in [("left", &mut state.left), ("right", &mut state.right)] {
            egui::ComboBox::from_id_salt(("lab.compare", salt))
                .selected_text(format!("{} · {}", *index + 1, sessions[*index].title()))
                .show_ui(ui, |ui| {
                    for (i, s) in sessions.iter().enumerate() {
                        ui.selectable_value(index, i, format!("{} · {}", i + 1, s.title()));
                    }
                });
        }
        if pill(ui, "Pause and step both", true, true).clicked() {
            state.status = match pair_error(sessions, state.left, state.right) {
                Some(error) => error.into(),
                None => {
                    for index in [state.left, state.right] {
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
            for (column, index) in columns.iter_mut().zip([state.left, state.right]) {
                thumbnail(column, &sessions[index]);
            }
        });
    });
    exports(ui, state, &data.bench, saved_path);
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

fn exports(ui: &mut Ui, state: &mut Comparison, bench: &Workbench, saved_path: &str) {
    if state.job.as_ref().is_some_and(|job| job.is_finished()) {
        let result = state.job.take().unwrap().join();
        state.status = result
            .unwrap_or_else(|_| Err("Export worker failed".into()))
            .map_or_else(
                |e| e,
                |()| "Export complete: PNG, GIF, project and manifest".into(),
            );
    }
    margin_section(ui, "export", false, |ui| {
        ui.add(
            egui::TextEdit::singleline(&mut state.folder).hint_text(serif_italic("output folder")),
        );
        note(
            ui,
            "30 frames, one transition per frame, 100 ms each. Choose a new folder.",
            false,
        );
        let idle = state.job.is_none();
        if pill(ui, "Export comparison", false, idle).clicked() {
            let project = pair_project(bench, state.left, state.right);
            start(state, project);
        }
        if pill(ui, "Export saved project (includes Lenia)", false, idle).clicked() {
            start(state, project::read(std::path::Path::new(saved_path)));
        }
        if !state.status.is_empty() {
            note(ui, &state.status, false);
        }
    });
}

/// A project holding only the chosen pair, left first.
fn pair_project(bench: &Workbench, left: usize, right: usize) -> Result<Project, String> {
    let mut project = Project::capture(bench, Vec::new())?;
    if left == right || left >= project.sessions.len() || right >= project.sessions.len() {
        return Err("Choose two different sessions".into());
    }
    let mut chosen = Vec::new();
    for index in [left, right] {
        let value = serde_json::to_value(&project.sessions[index]).map_err(|e| e.to_string())?;
        chosen.push(serde_json::from_value(value).map_err(|e| e.to_string())?);
    }
    project.sessions = chosen;
    project.selected = 0;
    Ok(project)
}

fn start(state: &mut Comparison, project: Result<Project, String>) {
    match project {
        Err(e) => state.status = e,
        Ok(project) => {
            let folder = std::path::PathBuf::from(&state.folder);
            state.job = Some(std::thread::spawn(move || {
                write_bundle(&project, &folder, 30, 100)
            }));
            state.status = "Exporting isolated simulation copies…".into();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::workbench::catalog::Kind;

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
