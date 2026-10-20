//! Rule drafts are explicit: text validation prepares settings, Apply changes the session.
use bevy_egui::egui::{self, Ui};
use models::workbench::experiment::config::{
    Settings,
    authored::{Rule, composition::Program, storage},
};
mod transfer;
#[derive(Clone, Debug, Default)]
pub struct Editor {
    text: String,
    path: String,
    status: String,
}
impl Editor {
    fn set(&mut self, program: &Program) {
        self.text = serde_json::to_string_pretty(program).unwrap();
    }
}
pub fn draw(ui: &mut Ui, settings: &mut Settings, editor: &mut Editor) {
    ui.collapsing("Authored rules and layers", |ui| {
        ui.label("Binary 3×3 rules. First matching transition wins.");
        ui.label("Bits 0–8: rows y=-1..1, columns x=-1..1; centre is bit 4.");
        ui.label("A transition matches when neighborhood & mask = pattern.");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Load current draft").clicked() {
                editor.set(&settings.program.clone().unwrap_or(Program {
                    primary: Rule::life(settings.birth, settings.survival),
                    secondary: None,
                }));
            }
            if ui.button("Copy-west example").clicked() {
                editor.set(&Program {
                    primary: Rule::copy_west(),
                    secondary: None,
                });
            }
            if ui.button("Use birth/survival masks").clicked() {
                settings.program = None;
                editor.status = "Built-in rule prepared; press Apply below".into();
            }
        });
        ui.add(
            egui::TextEdit::multiline(&mut editor.text)
                .code_editor()
                .desired_rows(10)
                .desired_width(f32::INFINITY)
                .char_limit(storage::MAX_BYTES),
        );
        transfer::draw(ui, editor);
        if ui.button("Validate and prepare rule").clicked() {
            match storage::parse(&editor.text) {
                Ok(program) => {
                    settings.program = Some(program);
                    editor.status = "Valid rule prepared; press Apply below".into();
                }
                Err(e) => editor.status = e,
            }
        }
        ui.label(&editor.status);
        if let Some(program) = &settings.program {
            ui.label(format!("Prepared: {}", program.title()));
        }
        ui.label("Rule files share definitions; My specimens shares the world and exact state.");
    });
}
