//! World configuration follows the lab's existing parameter draft/apply contract.
use crate::theme::widgets::{note, pill, segmented};
use bevy_egui::egui::{self, Ui};
use models::workbench::{catalog::Kind, experiment::config::Settings};
use simulation_kernel::core::boundary::Boundary;
pub(super) fn draw(ui: &mut Ui, s: &mut Settings, kind: Kind) {
    let old = (s.world.width, s.world.height);
    ui.horizontal(|ui| {
        ui.label("width");
        ui.add(egui::DragValue::new(&mut s.world.width).range(8..=256));
    });
    if !matches!(kind, Kind::Rule30 | Kind::Rule110) {
        ui.horizontal(|ui| {
            ui.label("height");
            ui.add(egui::DragValue::new(&mut s.world.height).range(8..=256));
        });
    }
    if old != (s.world.width, s.world.height) {
        s.world.seed = None;
    }
    segmented(
        ui,
        &mut s.world.boundary,
        &[
            (Boundary::Dead, "dead edges"),
            (Boundary::Periodic, "periodic"),
        ],
    );
    if s.world.seed.is_some() {
        note(ui, "Authored reset seed", false);
        if pill(ui, "Replace with preset", false, true).clicked() {
            s.world.seed = None;
        }
    }
    note(
        ui,
        "Size and edges require Apply and reset. Resizing replaces the seed.",
        false,
    );
}
