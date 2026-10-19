use super::Comparison;
use bevy_egui::egui;
use models::workbench::{experiment::export::write_bundle, plugin::Workbench, project::Project};
pub(super) fn show(ui: &mut egui::Ui, state: &mut Comparison, bench: &Workbench, saved_path: &str) {
    if state.job.as_ref().is_some_and(|job| job.is_finished()) {
        state.status = state
            .job
            .take()
            .unwrap()
            .join()
            .unwrap_or_else(|_| Err("Export worker failed".into()))
            .map_or_else(
                |e| e,
                |()| "Export complete: PNG, GIF, project and manifest".into(),
            );
    }
    ui.text_edit_singleline(&mut state.folder);
    ui.small("Exports: 30 frames, one transition/frame, 100 ms/frame. Choose a new output folder.");
    ui.add_enabled_ui(state.job.is_none(), |ui| {
        if ui.button("Export comparison").clicked() {
            let project = Project::capture(bench, Vec::new()).and_then(|mut p| {
                if state.left == state.right
                    || state.left >= p.sessions.len()
                    || state.right >= p.sessions.len()
                {
                    return Err("Choose two different sessions".into());
                }
                let mut chosen = Vec::new();
                for index in [state.left, state.right] {
                    chosen.push(
                        serde_json::from_value(
                            serde_json::to_value(&p.sessions[index]).map_err(|e| e.to_string())?,
                        )
                        .map_err(|e| e.to_string())?,
                    );
                }
                p.sessions = chosen;
                p.selected = 0;
                Ok(p)
            });
            start(state, project);
        }
        if ui.button("Export saved project (includes Lenia)").clicked() {
            start(
                state,
                models::workbench::project::read(std::path::Path::new(saved_path)),
            );
        }
    });
    ui.label(&state.status);
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
