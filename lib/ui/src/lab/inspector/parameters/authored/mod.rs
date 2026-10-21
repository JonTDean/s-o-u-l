//! Rule drafts are explicit: text validation prepares settings, Apply changes the session.
use crate::theme::widgets::{code_frame, dim, mono, note, pill};
use bevy_egui::egui::{self, Ui};
use models::workbench::experiment::config::{
    Settings,
    authored::{Rule, composition::Program, storage},
};
mod layers;
mod transfer;
#[derive(Clone, Debug, Default)]
pub struct Editor {
    text: String,
    path: String,
    /// Last outcome message and whether it is an error.
    status: (String, bool),
}
impl Editor {
    pub fn for_settings(settings: &Settings) -> Self {
        let mut editor = Self::default();
        if let Some(program) = &settings.program {
            editor.set(program);
        }
        editor
    }
    fn ok(&mut self, message: &str) {
        self.status = (message.into(), false);
    }
    fn fail(&mut self, message: String) {
        self.status = (message, true);
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
                    editor.ok("Built-in rule prepared; press Apply below");
                }
            });
            code_frame().show(ui, |ui| {
                egui::ScrollArea::both().max_height(220.0).show(ui, |ui| {
                    // Long lines scroll sideways instead of soft-wrapping.
                    let mut unwrapped = |ui: &Ui, text: &str, _: f32| {
                        let font = egui::TextStyle::Monospace.resolve(ui.style());
                        let mut job = egui::text::LayoutJob::single_section(
                            text.into(),
                            egui::TextFormat::simple(font, ui.visuals().text_color()),
                        );
                        job.wrap.max_width = f32::INFINITY;
                        ui.fonts(|fonts| fonts.layout_job(job))
                    };
                    ui.add(
                        egui::TextEdit::multiline(&mut editor.text)
                            .code_editor()
                            .frame(false)
                            .layouter(&mut unwrapped)
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
                        editor.ok("Valid rule prepared; press Apply below");
                    }
                    Err(e) => editor.fail(e),
                }
            }
            if !editor.status.0.is_empty() {
                note(ui, &editor.status.0, editor.status.1);
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

    #[test]
    fn long_lines_do_not_soft_wrap() {
        let ctx = egui::Context::default();
        crate::theme::visuals::install(&ctx);
        ctx.set_fonts(crate::theme::fonts::definitions());
        let _ = ctx.run(egui::RawInput::default(), |_| {});
        let mut settings = Settings::for_kind(Kind::Life);
        settings.program = Some(Program {
            primary: Rule::life(settings.birth, settings.survival),
            secondary: None,
        });
        let mut editor = Editor::default();
        editor.text = format!("{{\"long\": \"{}\"}}", "x".repeat(600));
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(300.0, 900.0),
            )),
            ..Default::default()
        };
        let out = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                draw(ui, &mut settings, &mut editor);
            });
        });
        let rows = out.shapes.iter().find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text().starts_with("{\"long\"") => {
                Some(t.galley.rows.len())
            }
            _ => None,
        });
        assert_eq!(rows, Some(1), "the line stays on one row");
    }
}
