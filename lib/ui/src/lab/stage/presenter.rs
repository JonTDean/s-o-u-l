//! Presenter mode: full-window stage with HUD readouts (filled in by Task 15).
use crate::lab::{Effects, LabData, state::LabState};
use bevy_egui::egui;
use engine_common::controls::camera::manager::stage::StageRect;

/// Draw presenter mode and publish its stage rectangle.
pub fn show(
    _: &egui::Context,
    _: &mut LabState,
    _: &LabData,
    _: &mut Effects,
    _: bool,
) -> StageRect {
    StageRect::default()
}
