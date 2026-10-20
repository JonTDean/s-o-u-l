//! Scenario launcher and persistent guide/presenter cues.
use crate::lab::{Effects, LabData, state::{Drawer, LabState}};
use crate::lab::chrome::drawers::{Comparison, Files};
use bevy_egui::egui::{self, Ui};
use engine_core::{automata::channel::{ChannelAction, ChannelCommand, Status}, events::AutomataCommand};
use models::workbench::catalog::scenarios::Scenario;
#[derive(Debug, Default)]
pub struct Guide {
    pub selected: Scenario,
    pub active: Option<Scenario>,
    pub step: usize,
}
pub fn launcher(ui: &mut Ui, lab: &mut LabState, files: &mut Files, fx: &mut Effects) {
    ui.heading("Experiments");
    egui::ComboBox::from_id_salt("scenario").selected_text(lab.guide.selected.title()).show_ui(ui, |ui| {
        for scenario in Scenario::ALL { ui.selectable_value(&mut lab.guide.selected, scenario, scenario.title()); }
    });
    let idle = files.save.is_none() && files.load.is_none() && files.request.is_none();
    if ui.add_enabled(idle, egui::Button::new("Start scenario (replace channels)")).clicked() {
        match lab.guide.selected.project() {
            Ok(project) => {
                fx.automata.push(AutomataCommand::Clear);
                files.load = Some(project);
                lab.guide.active = Some(lab.guide.selected);
                lab.guide.step = 0;
                lab.drawer = None;
            }
            Err(e) => files.status = e,
        }
    }
    ui.separator();
}
pub fn overlay(ctx: &egui::Context, lab: &mut LabState, data: &LabData, comparison: &mut Comparison, fx: &mut Effects) {
    let Some(scenario) = lab.guide.active else { return };
    egui::Window::new("Experiment guide").default_pos(egui::pos2(55.0, 80.0)).default_width(360.0).show(ctx, |ui| {
        ui.label(scenario.title());
        ui.label(format!("{} / 4", lab.guide.step + 1));
        ui.label(scenario.steps()[lab.guide.step]);
        ui.horizontal(|ui| {
            if ui.add_enabled(lab.guide.step > 0, egui::Button::new("Back")).clicked() { lab.guide.step -= 1; }
            if ui.add_enabled(lab.guide.step < 3, egui::Button::new("Next cue")).clicked() { lab.guide.step += 1; }
            if ui.button("Finish guide").clicked() { lab.guide.active = None; }
        });
        let rows = &data.channels.rows;
        let paused = !rows.is_empty() && rows.iter().all(|r| r.status != Status::Running);
        if ui.add_enabled(paused, egui::Button::new("Step all paused channels")).clicked() {
            fx.commands.extend(rows.iter().map(|r| ChannelCommand { id: r.id, action: ChannelAction::Step }));
        }
        if ui.add_enabled(rows.len() >= 2, egui::Button::new("Compare and export")).clicked() {
            comparison.left = Some(rows[0].id);
            comparison.right = Some(rows[1].id);
            lab.drawer = Some(Drawer::Compare);
            lab.mode = crate::lab::state::LabMode::Workbench;
        }
        ui.small("Presenter script: predict → step → explain → retain evidence. Cues stay visible in presenter mode.");
    });
}
