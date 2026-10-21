//! Composition controls edit the same portable program as the text editor.
use super::*;
use crate::theme::widgets::{dim, pill, segmented};
use models::workbench::experiment::config::authored::composition::{Blend, Layer};
pub(super) fn draw(ui: &mut Ui, editor: &mut Editor) {
    let Ok(mut program) = storage::parse(&editor.text) else {
        return;
    };
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        if pill(ui, "Add copy-west layer", false, true).clicked() {
            program.secondary = Some(Layer {
                rule: Rule::copy_west(),
                blend: Blend::Union,
            });
            changed = true;
        }
        if pill(ui, "Remove second layer", false, true).clicked() {
            program.secondary = None;
            changed = true;
        }
    });
    if let Some(layer) = &mut program.secondary {
        ui.label(dim(
            "Both layers read the prior grid; their combined output feeds the next tick.",
        ));
        changed |= segmented(
            ui,
            &mut layer.blend,
            &[
                (Blend::Union, "Union"),
                (Blend::Intersection, "Intersection"),
                (Blend::ExclusiveOr, "XOR"),
            ],
        );
    }
    if changed {
        editor.set(&program);
    }
}
