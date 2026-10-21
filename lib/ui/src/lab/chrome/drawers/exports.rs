//! Compare drawer exports: an isolated pair bundle or the saved project, on a worker thread.
use super::compare::Comparison;
use crate::theme::widgets::{dim, note, pill, section};
use bevy_egui::egui::{self, Ui};
use models::workbench::{
    experiment::export::write_bundle,
    plugin::Workbench,
    project::{self, Project},
};

/// Export controls and the worker's result.
pub fn show(ui: &mut Ui, state: &mut Comparison, bench: &Workbench, saved_path: &str) {
    if state.job.as_ref().is_some_and(|job| job.is_finished()) {
        let result = state.job.take().unwrap().join();
        state.status = result
            .unwrap_or_else(|_| Err("Export worker failed".into()))
            .map_or_else(
                |e| e,
                |()| "Export complete: PNG, GIF, project and manifest".into(),
            );
    }
    section("export").show(ui, |ui| {
        ui.add(egui::TextEdit::singleline(&mut state.folder).hint_text(dim("output folder")));
        note(
            ui,
            "30 frames, one transition per frame, 100 ms each. Choose a new folder.",
            false,
        );
        let idle = state.job.is_none();
        let cpu_pair = state.left.zip(state.right).and_then(|(left, right)| {
            if left == right {
                return None;
            }
            let find = |id| bench.sessions.iter().position(|s| s.channel == id);
            find(left).zip(find(right))
        });
        if pill(
            ui,
            "Export CPU pair (PNG/GIF)",
            false,
            idle && cpu_pair.is_some(),
        )
        .clicked()
        {
            let project = cpu_pair.map_or_else(
                || Err("Choose two different sessions".into()),
                |(l, r)| pair_project(bench, l, r),
            );
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
