//! Composition controls edit the same portable program as the text editor.
use super::*;
use models::workbench::experiment::config::authored::composition::{Blend, Layer};
pub(super) fn draw(ui: &mut Ui, editor: &mut Editor) {
    let Ok(mut program) = storage::parse(&editor.text) else {
        return;
    };
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        if ui.button("Add copy-west layer").clicked() {
            program.secondary = Some(Layer {
                rule: Rule::copy_west(),
                blend: Blend::Union,
            });
            changed = true;
        }
        if ui.button("Remove second layer").clicked() {
            program.secondary = None;
            changed = true;
        }
    });
    if let Some(layer) = &mut program.secondary {
        ui.label("Both layers read the prior grid; their combined output feeds the next tick.");
        ui.horizontal_wrapped(|ui| {
            for (value, label) in [
                (Blend::Union, "Union"),
                (Blend::Intersection, "Intersection"),
                (Blend::ExclusiveOr, "XOR"),
            ] {
                changed |= ui
                    .selectable_value(&mut layer.blend, value, label)
                    .changed();
            }
        });
    }
    if changed {
        editor.set(&program);
    }
}
