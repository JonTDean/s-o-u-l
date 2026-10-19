use super::*;
use bevy_egui::{EguiContexts, egui};
use engine_core::events::AutomataCommand;
use models::workbench::{plugin::Workbench, project};
pub(super) fn show(
    mut contexts: EguiContexts,
    mut files: ResMut<Files>,
    mut bench: ResMut<Workbench>,
    mut sims: Query<(Entity, &mut LeniaSimulation)>,
    mut commands: EventWriter<AutomataCommand>,
    #[cfg(feature = "gpu-compute")] probe: Res<engine_gpu::LeniaProbe>,
) {
    if let Some(project) = files.load.take() {
        project.apply_cpu(&mut bench);
    }
    #[cfg(feature = "gpu-compute")]
    capture::poll(&mut files, &probe, &sims);
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    egui::Window::new("Project")
        .default_pos([12.0, 650.0])
        .default_width(290.0)
        .show(ctx, |ui| {
            ui.text_edit_singleline(&mut files.path);
            let paused =
                bench.sessions.iter().all(|s| s.paused) && sims.iter().all(|(_, s)| s.paused);
            if ui.button("Pause all").clicked() {
                for session in &mut bench.sessions {
                    session.paused = true;
                }
                for (_, mut sim) in &mut sims {
                    sim.paused = true;
                }
            }
            ui.small("Pause all to save a completed checkpoint.");
            let idle = files.save.is_none();
            if ui
                .add_enabled(paused && idle, egui::Button::new("Save project"))
                .clicked()
            {
                match Project::capture(&bench, Vec::new()) {
                    Err(e) => files.status = e,
                    Ok(project) => {
                        let mut owners: Vec<_> = sims.iter().map(|(e, s)| (e, s.clone())).collect();
                        owners.sort_by_key(|(e, _)| e.to_bits());
                        if owners.is_empty() {
                            files.status =
                                project::write(std::path::Path::new(&files.path), &project)
                                    .map_or_else(|e| e, |_| "Project saved".into());
                        } else {
                            #[cfg(feature = "gpu-compute")]
                            {
                                for (entity, _) in &owners {
                                    probe.take(*entity);
                                    probe.request(*entity);
                                }
                                files.save = Some(Pending {
                                    project,
                                    owners,
                                    path: files.path.clone().into(),
                                    started: std::time::Instant::now(),
                                });
                                files.status = "Waiting for completed GPU state…".into();
                            }
                            #[cfg(not(feature = "gpu-compute"))]
                            {
                                files.status =
                                    "GPU checkpoints require the gpu-compute feature".into();
                            }
                        }
                    }
                }
            }
            if ui
                .add_enabled(idle, egui::Button::new("Load project (replace sessions)"))
                .clicked()
            {
                match project::read(std::path::Path::new(&files.path)) {
                    Err(e) => files.status = e,
                    Ok(mut project) => {
                        commands.write(AutomataCommand::Clear);
                        for saved in project.lenia.drain(..) {
                            commands.write(AutomataCommand::RestoreLenia(Box::new(saved)));
                        }
                        files.load = Some(project);
                        files.status = "Project loaded".into();
                    }
                }
            }
            ui.label(&files.status);
        });
}
