use super::*;
use crate::theme::widgets::{choice, margin_section, note, pill};
pub fn show(ui: &mut Ui, editor: &mut Editor, session: &Session, fx: &mut Effects) {
    if !matches!(session.model, Model::Grid(_)) {
        return;
    }
    margin_section(ui, "author", false, |ui| {
        ui.add_enabled_ui(session.paused, |ui| {
            choice(
                ui,
                &mut editor.tool,
                &[
                    (Tool::Navigate, "pan"),
                    (Tool::Paint, "paint"),
                    (Tool::Erase, "erase"),
                ],
            );
            if session.kind == Kind::Hpp {
                ui.label("Direction bits (1..15)");
                ui.add(bevy_egui::egui::DragValue::new(&mut editor.gas).range(1..=15));
            }
            ui.horizontal(|ui| {
                for (label, redo, enabled) in [
                    ("Undo", false, session.edits.can_undo()),
                    ("Redo", true, session.edits.can_redo()),
                ] {
                    if pill(ui, label, false, enabled).clicked() {
                        fx.commands.push(ChannelCommand {
                            id: session.channel,
                            action: ChannelAction::UndoPaint(redo),
                        });
                    }
                }
            });
        });
        note(
            ui,
            "Pause to paint. Each stroke starts a tick-zero experiment and becomes its reset seed. Undo retains 16 strokes until Reset or load.",
            false,
        );
    });
}
