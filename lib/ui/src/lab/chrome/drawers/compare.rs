//! Compare drawer: session comparison and exports (moved in from the legacy workbench in Task 16).
use crate::{
    lab::{Effects, LabData, state::LabState},
    theme::widgets::{margin_section, note},
};
use bevy_egui::egui::Ui;

/// Drawer body.
pub fn body(ui: &mut Ui, _: &mut LabState, _: &LabData, _: &mut Effects) {
    margin_section(ui, "compare", true, |ui| {
        note(ui, "Comparison moves here.", false)
    });
}
