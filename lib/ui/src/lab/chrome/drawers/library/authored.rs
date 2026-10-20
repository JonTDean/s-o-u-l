//! User-authored specimens share project capture and restoration.
use super::*;
use crate::lab::chrome::drawers::project::{Files, Request};
use models::workbench::project::library;
use std::path::Path;

#[derive(Default, Debug)]
pub struct Authoring {
    pub name: String,
    pub transfer: String,
    pub status: String,
}

pub fn body(ui: &mut Ui, lab: &mut LabState, data: &LabData, files: &mut Files) {
    let root = models::workbench::project::library::default_root();
    let state = &mut lab.authoring;
    margin_section(ui, "my specimens", false, |ui| {
        ui.text_edit_singleline(&mut state.name)
            .on_hover_text("New specimen name");
        let idle = files.save.is_none() && files.request.is_none() && files.load.is_none();
        let selected = data
            .channels
            .selected_row()
            .filter(|r| r.status != engine_core::automata::channel::Status::Running);
        if pill(ui, "Save selected as new", true, idle && selected.is_some()).clicked() {
            match library::path(&root, &state.name) {
                Ok(path) if !path.exists() => {
                    files.path = path.to_string_lossy().into();
                    files.request = selected.map(|r| Request::Specimen(r.id));
                }
                Ok(_) => state.status = "Name already exists; choose a new name".into(),
                Err(e) => state.status = e,
            }
        }
        ui.label(serif_italic(
            "Open replaces channels with the saved starting state.",
        ));
        match library::entries(&root) {
            Err(e) => state.status = e,
            Ok(names) => {
                for name in names {
                    if !name.to_lowercase().contains(&lab.filter.to_lowercase()) {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        if pill(ui, &name, false, idle).clicked() {
                            files.path = library::path(&root, &name)
                                .unwrap()
                                .to_string_lossy()
                                .into();
                            files.request = Some(Request::Load);
                        }
                        if pill(ui, "duplicate", false, idle).clicked() {
                            state.status = library::path(&root, &state.name)
                                .and_then(|dest| {
                                    library::copy(&library::path(&root, &name)?, &dest)
                                })
                                .map_or_else(|e| e, |_| "Duplicated".into());
                        }
                        if pill(ui, "export", false, idle).clicked() {
                            state.status = library::copy(
                                &library::path(&root, &name).unwrap(),
                                Path::new(&state.transfer),
                            )
                            .map_or_else(|e| e, |_| "Exported".into());
                        }
                    });
                }
            }
        }
        ui.text_edit_singleline(&mut state.transfer)
            .on_hover_text("Import/export JSON path");
        if pill(ui, "Import as new name", false, idle).clicked() {
            state.status = library::path(&root, &state.name)
                .and_then(|dest| library::copy(Path::new(&state.transfer), &dest))
                .map_or_else(|e| e, |_| "Imported".into());
        }
        note(ui, &state.status, false);
        note(ui, &files.status, false);
    });
}
