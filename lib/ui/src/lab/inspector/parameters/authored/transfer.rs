//! Rule exchange never overwrites an existing destination.
use super::*;
pub(super) fn draw(ui: &mut Ui, editor: &mut Editor) {
    ui.text_edit_singleline(&mut editor.path)
        .on_hover_text("Rule JSON import/export path");
    ui.horizontal(|ui| {
        if ui.button("Import rule").clicked() {
            match storage::read(std::path::Path::new(&editor.path)) {
                Ok(program) => {
                    editor.set(&program);
                    editor.status = "Imported text; validate to prepare".into();
                }
                Err(e) => editor.status = e,
            }
        }
        if ui.button("Export new file").clicked() {
            editor.status = storage::parse(&editor.text)
                .and_then(|p| storage::write(std::path::Path::new(&editor.path), &p))
                .map_or_else(|e| e, |_| "Exported".into());
        }
    });
}
