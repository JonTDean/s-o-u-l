//! Project drawer: save and load, with asynchronous completed-state capture for GPU owners.
//!
//! The drawer only records a [`Request`]; [`persist`] performs it and keeps polling a pending
//! checkpoint whether or not the drawer is open, so scripted qualification can drive `Files`.
use crate::{
    lab::{Effects, LabData},
    theme::widgets::{dim, note, pill, section},
};
use bevy::prelude::*;
use bevy_egui::egui::{self, Ui};
use engine_core::{
    automata::{
        channel::{ChannelAction, ChannelCommand, ChannelId, Status},
        lenia::{LeniaSimulation, ReadbackBusy},
    },
    events::AutomataCommand,
};
use models::workbench::{
    plugin::Workbench,
    project::{self, Project},
};
#[cfg(feature = "gpu-compute")]
mod capture;

/// Project path, status line, a project waiting to replace CPU sessions, and a pending save.
#[derive(Resource)]
pub struct Files {
    /// JSON project path.
    pub path: String,
    /// Last result or error.
    pub status: String,
    /// Loaded project whose CPU sessions apply on the next pass (after `Clear` lands).
    pub load: Option<Project>,
    /// Save waiting for GPU readbacks.
    pub save: Option<Pending>,
    /// Drawer action to perform this pass.
    pub request: Option<Request>,
}

impl Default for Files {
    fn default() -> Self {
        Self {
            path: "soul-project.json".into(),
            status: String::new(),
            load: None,
            save: None,
            request: None,
        }
    }
}

/// A save waiting for completed GPU state from each Lenia owner.
pub struct Pending {
    /// CPU sessions captured when the save started.
    pub project: Project,
    /// Owners and the state each must still match when its readback arrives.
    pub owners: Vec<(Entity, LeniaSimulation)>,
    /// Destination.
    pub path: std::path::PathBuf,
    /// Start time for the 15 s timeout.
    pub started: std::time::Instant,
    /// Create a new single-channel library entry.
    pub specimen: bool,
}

/// Drawer buttons that need world access.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// Save every paused channel.
    Save,
    /// Save only the selected paused channel as a named specimen.
    Specimen(ChannelId),
    /// Replace every channel with the saved project.
    Load,
}

/// Drawer body.
pub fn body(ui: &mut Ui, files: &mut Files, data: &LabData, fx: &mut Effects) {
    section("project").active(true).show(ui, |ui| {
        ui.add(egui::TextEdit::singleline(&mut files.path).hint_text(dim("project.json")));
        let rows = &data.channels.rows;
        if pill(ui, "Pause all", false, !rows.is_empty()).clicked() {
            for row in rows {
                fx.commands.push(ChannelCommand {
                    id: row.id,
                    action: ChannelAction::Pause(true),
                });
            }
        }
        note(ui, "Pause all to save a completed checkpoint.", false);
        let paused = rows.iter().all(|r| r.status != Status::Running);
        let idle = files.save.is_none() && files.request.is_none();
        if pill(ui, "Save project", true, paused && idle).clicked() {
            files.request = Some(Request::Save);
        }
        if pill(ui, "Load project (replace channels)", false, idle).clicked() {
            files.request = Some(Request::Load);
        }
        if !files.status.is_empty() {
            note(ui, &files.status, false);
        }
    });
}

/// Perform drawer requests, poll a pending GPU capture and apply a loaded project.
pub fn persist(
    mut files: ResMut<Files>,
    mut bench: ResMut<Workbench>,
    sims: Query<(Entity, &LeniaSimulation, &ChannelId)>,
    mut commands: EventWriter<AutomataCommand>,
    #[cfg(feature = "gpu-compute")] probe: Res<engine_gpu::LeniaProbe>,
) {
    // Applied one pass after the load request so the `Clear` it sent has emptied the bench.
    if let Some(project) = files.load.take() {
        project.apply_cpu(&mut bench);
    }
    #[cfg(feature = "gpu-compute")]
    capture::poll(&mut files, &probe, &sims);
    match files.request.take() {
        Some(request @ (Request::Save | Request::Specimen(_))) => save(
            &mut files,
            &bench,
            &sims,
            match request {
                Request::Specimen(id) => Some(id),
                _ => None,
            },
            #[cfg(feature = "gpu-compute")]
            &probe,
        ),
        Some(Request::Load) => match project::read(std::path::Path::new(&files.path)) {
            Err(e) => files.status = e,
            Ok(mut project) => {
                commands.write(AutomataCommand::Clear);
                for saved in project.lenia.drain(..) {
                    commands.write(AutomataCommand::RestoreLenia(Box::new(saved)));
                }
                files.load = Some(project);
                files.status = "Project loaded".into();
            }
        },
        None => {}
    }
}

fn save(
    files: &mut Files,
    bench: &Workbench,
    sims: &Query<(Entity, &LeniaSimulation, &ChannelId)>,
    selected: Option<ChannelId>,
    #[cfg(feature = "gpu-compute")] probe: &engine_gpu::LeniaProbe,
) {
    let mut project = match Project::capture(bench, Vec::new()) {
        Err(e) => return files.status = e,
        Ok(project) => project,
    };
    if let Some(id) = selected {
        project.sessions = bench
            .sessions
            .iter()
            .position(|s| s.channel == id)
            .map(|index| project.sessions.remove(index))
            .into_iter()
            .collect();
        project.selected = 0;
    }
    let mut owners: Vec<_> = sims
        .iter()
        .filter(|(_, _, id)| selected.is_none_or(|selected| selected == **id))
        .map(|(e, s, _)| (e, s.clone()))
        .collect();
    if selected.is_some() && project.sessions.is_empty() && owners.is_empty() {
        files.status = "Selected channel was removed".into();
        return;
    }
    owners.sort_by_key(|(e, _)| e.to_bits());
    if owners.is_empty() {
        files.status = if selected.is_some() {
            project::library::create(std::path::Path::new(&files.path), &project)
        } else {
            project::write(std::path::Path::new(&files.path), &project)
        }
        .map_or_else(|e| e, |_| "Project saved".into());
        return;
    }
    #[cfg(feature = "gpu-compute")]
    {
        for (entity, _) in &owners {
            probe.take(*entity);
            probe.request(*entity);
        }
        let path = files.path.clone().into();
        let started = std::time::Instant::now();
        files.save = Some(Pending {
            project,
            owners,
            path,
            started,
            specimen: selected.is_some(),
        });
        files.status = "Waiting for completed GPU state…".into();
    }
    #[cfg(not(feature = "gpu-compute"))]
    {
        files.status = "GPU checkpoints require the gpu-compute feature".into();
    }
}

/// A pending checkpoint owns the shared `LeniaProbe`; metric sampling waits for it.
pub fn claim_mailbox(files: Res<Files>, mut busy: ResMut<ReadbackBusy>) {
    busy.0 = files.save.is_some();
}
