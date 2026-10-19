//! Experiment controls separated from model-specific state drawing.
mod fields;
use bevy_egui::egui;
use models::workbench::{catalog::Kind, experiment::config::Settings, session::Session};

#[derive(Default)]
pub struct Editor {
    selected: Option<(usize, Kind)>,
    original: Option<Settings>,
    draft: Option<Settings>,
    error: Option<String>,
}

pub(super) fn settings(
    ui: &mut egui::Ui,
    index: usize,
    session: &mut Session,
    editor: &mut Editor,
) {
    if editor.selected != Some((index, session.kind))
        || editor.original.as_ref() != Some(&session.settings)
    {
        editor.selected = Some((index, session.kind));
        editor.original = Some(session.settings.clone());
        editor.draft = Some(session.settings.clone());
        editor.error = None;
    }
    ui.collapsing("Experiment settings", |ui| {
        let draft = editor.draft.as_mut().unwrap();
        fields::draw(ui, session.kind, draft);
        ui.label("Rate and rule parameters apply live. Seed and population changes require reset.");
        ui.horizontal(|ui| {
            for (label, reset) in [("Apply live", false), ("Apply and reset", true)] {
                if ui.button(label).clicked() {
                    editor.error = session
                        .configure(draft.clone(), reset)
                        .err()
                        .map(str::to_owned);
                }
            }
            if ui.button("Defaults").clicked() {
                *draft = Settings::for_kind(session.kind);
            }
        });
        if let Some(error) = &editor.error {
            ui.colored_label(egui::Color32::LIGHT_RED, error);
        }
    });
}
