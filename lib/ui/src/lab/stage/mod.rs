//! Stage area between the chrome panels (filled in by Task 15).
use super::{Effects, LabData, state::LabState};
use bevy_egui::egui;
use engine_common::controls::camera::manager::stage::StageRect;
pub mod overlay;
pub mod painters;
pub mod presenter;
pub mod view;

/// Draw the stage and publish its rectangle.
pub fn show(
    _: &egui::Context,
    _: &mut LabState,
    _: &LabData,
    _: &mut Effects,
    _: bool,
) -> StageRect {
    StageRect::default()
}
