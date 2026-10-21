//! Rule drafts are explicit: text validation prepares settings, Apply changes the session.
use crate::theme::widgets::{code_frame, dim, mono, note, pill};
use bevy_egui::egui::{self, Ui};
use models::workbench::experiment::config::{
    Settings,
    authored::{Rule, composition::Program, storage},
};
const SUCCESS: [&str; 4] = [
    "Built-in rule prepared; press Apply below",
    "Valid rule prepared; press Apply below",
    "Imported text; validate to prepare",
    "Exported",
];
mod layers;
mod transfer;
#[derive(Clone, Debug, Default)]
pub struct Editor {
    text: String,
    path: String,
    status: String,
}
impl Editor {
    pub fn for_settings(settings: &Settings) -> Self {
        let mut editor = Self::default();
        if let Some(program) = &settings.program {
            editor.set(program);
        }
        editor
    }
    fn set(&mut self, program: &Program) {
        self.text = serde_json::to_string_pretty(program).unwrap();
    }
}
pub fn draw(ui: &mut Ui, settings: &mut Settings, editor: &mut Editor) {
    egui::CollapsingHeader::new("Authored rules and layers")
        .id_salt(("authored-rules", settings.program.is_some()))
        .default_open(settings.program.is_some())
        .show(ui, |ui| {
            ui.label(dim("Binary 3×3 rules. First matching transition wins. \
                 Bits 0–8: rows y=-1..1, columns x=-1..1; centre is bit 4. \
                 A transition matches when neighborhood & mask = pattern."));
            ui.horizontal_wrapped(|ui| {
                if pill(ui, "Load current draft", false, true).clicked() {
                    editor.set(&settings.program.clone().unwrap_or(Program {
                        primary: Rule::life(settings.birth, settings.survival),
                        secondary: None,
                    }));
                }
                if pill(ui, "Copy-west example", false, true).clicked() {
                    editor.set(&Program {
                        primary: Rule::copy_west(),
                        secondary: None,
                    });
                }
                if pill(ui, "Use birth/survival masks", false, true).clicked() {
                    settings.program = None;
                    editor.status = "Built-in rule prepared; press Apply below".into();
                }
            });
            egui::ScrollArea::both().max_height(220.0).show(ui, |ui| {
                code_frame().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut editor.text)
                            .code_editor()
                            .frame(false)
                            .desired_rows(10)
                            .desired_width(ui.available_width())
                            .char_limit(storage::MAX_BYTES),
                    );
                });
            });
            layers::draw(ui, editor);
            transfer::draw(ui, editor);
            if pill(ui, "Validate and prepare rule", true, true).clicked() {
                match storage::parse(&editor.text) {
                    Ok(program) => {
                        settings.program = Some(program);
                        editor.status = "Valid rule prepared; press Apply below".into();
                    }
                    Err(e) => editor.status = e,
                }
            }
            if !editor.status.is_empty() {
                note(
                    ui,
                    &editor.status,
                    !SUCCESS.contains(&editor.status.as_str()),
                );
            }
            if let Some(program) = &settings.program {
                ui.label(mono(format!("Prepared: {}", program.title())));
            }
            ui.label(dim(
                "Rule files share definitions; My specimens shares the world and exact state.",
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::workbench::catalog::Kind;

    #[test]
    fn authored_form_renders_headless() {
        let ctx = egui::Context::default();
        let mut settings = Settings::for_kind(Kind::Life);
        let mut editor = Editor::default();
        let out = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| draw(ui, &mut settings, &mut editor));
        });
        assert!(!out.shapes.is_empty());
    }
}
