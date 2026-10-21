//! Rule exchange never overwrites an existing destination.
use super::*;
use crate::theme::widgets::{dim, pill};
pub(super) fn draw(ui: &mut Ui, editor: &mut Editor) {
    let (imported, exported) = ui
        .horizontal_wrapped(|ui| {
            (
                pill(ui, "Import rule", false, true).clicked(),
                pill(ui, "Export new file", false, true).clicked(),
            )
        })
        .inner;
    ui.add(
        egui::TextEdit::singleline(&mut editor.path)
            .desired_width(ui.available_width())
            .hint_text(dim("Rule JSON import/export path")),
    )
    .on_hover_text("Rule JSON import/export path");
    if imported {
        match storage::read(std::path::Path::new(&editor.path)) {
            Ok(program) => {
                editor.set(&program);
                editor.status = "Imported text; validate to prepare".into();
            }
            Err(e) => editor.status = e,
        }
    }
    if exported {
        editor.status = storage::parse(&editor.text)
            .and_then(|p| storage::write(std::path::Path::new(&editor.path), &p))
            .map_or_else(|e| e, |_| "Exported".into());
    }
}
