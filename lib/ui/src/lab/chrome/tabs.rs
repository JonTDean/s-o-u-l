//! Channel tabs: serif specimen names, status lamps, close and add.
use crate::{
    lab::{
        Effects, LabData,
        state::{Drawer, LabState},
    },
    theme::{
        tokens::*,
        widgets::{lamp, serif},
    },
};
use bevy_egui::egui::{self, Frame, Margin, Sense, Stroke};
use engine_core::automata::channel::{ChannelAction, ChannelCommand, ChannelRow};
use models::workbench::catalog::specimens::Specimens;

/// Tab label: the specimen title, falling back to the catalog id.
pub fn title(row: &ChannelRow, specimens: &Specimens) -> String {
    specimens
        .get(&row.kind)
        .map_or_else(|| row.kind.clone(), |s| s.title.clone())
}

/// Draw the top tab bar.
pub fn show(ctx: &egui::Context, lab: &mut LabState, data: &LabData, fx: &mut Effects) {
    egui::TopBottomPanel::top("lab.tabs")
        .exact_height(28.0)
        .frame(
            Frame::new()
                .fill(PANEL)
                .inner_margin(Margin::symmetric(12, 0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 16.0;
                for row in &data.channels.rows {
                    let selected = data.channels.selected == Some(row.id);
                    let response = ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        lamp(ui, row.status);
                        let name = serif(title(row, &data.specimens))
                            .size(size::TITLE - 2.0)
                            .color(if selected { INK_HI } else { INK_DIM });
                        if ui
                            .add(egui::Label::new(name).sense(Sense::click()))
                            .clicked()
                        {
                            fx.commands.push(ChannelCommand {
                                id: row.id,
                                action: ChannelAction::Select,
                            });
                        }
                        let cross = egui::RichText::new("×").color(INK_DIM);
                        let close = ui.add(egui::Label::new(cross).sense(Sense::click()));
                        if close.on_hover_text("Close").clicked() {
                            fx.commands.push(ChannelCommand {
                                id: row.id,
                                action: ChannelAction::Remove,
                            });
                        }
                    });
                    if selected {
                        let r = response.response.rect;
                        ui.painter().line_segment(
                            [r.left_bottom(), r.right_bottom()],
                            Stroke::new(2.0, SIGNAL),
                        );
                    }
                }
                let plus = serif("+").color(SIGNAL).size(size::TITLE);
                let add = ui.add(egui::Label::new(plus).sense(Sense::click()));
                if add.on_hover_text("Open a specimen (L)").clicked() {
                    lab.drawer = Some(Drawer::Library);
                    lab.launch_library = false;
                }
            });
        });
}
