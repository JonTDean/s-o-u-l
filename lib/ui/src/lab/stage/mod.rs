//! The stage: CPU painters or the 3-D world through `StageRect`, plus overlay chrome.
use super::{Effects, LabData, state::LabState};
use crate::theme::tokens::BG;
use bevy_egui::egui::{self, Frame, Sense};
use engine_common::controls::camera::manager::stage::StageRect;
use engine_core::automata::channel::Backend;
pub mod editing;
pub mod overlay;
pub mod painters;
pub mod presenter;
pub mod view;

/// Draw the stage as the central panel and return the rectangle for stage cameras.
pub fn show(
    ctx: &egui::Context,
    lab: &mut LabState,
    data: &LabData,
    fx: &mut Effects,
    typing: bool,
) -> StageRect {
    egui::CentralPanel::default()
        .frame(Frame::NONE)
        .show(ctx, |ui| paint(ui, lab, data, typing, true, fx))
        .inner
}

/// Shared by workbench and presenter; `chrome` toggles overlay and caption.
///
/// GPU channels leave the stage unpainted so the stage camera's 3-D view shows through.
pub(crate) fn paint(
    ui: &mut egui::Ui,
    lab: &mut LabState,
    data: &LabData,
    typing: bool,
    chrome: bool,
    fx: &mut Effects,
) -> StageRect {
    let rect = ui.max_rect();
    let response = ui.interact(rect, ui.id().with("stage"), Sense::click_and_drag());
    let row = data.channels.selected_row();
    let gpu = row.is_some_and(|r| r.backend == Backend::Gpu);
    let mut cursor = None;
    let mut zoom = 1.0;
    if !gpu {
        ui.painter().rect_filled(rect, 0.0, BG);
    }
    let session = row.and_then(|r| data.bench.sessions.iter().find(|s| s.channel == r.id));
    match session {
        Some(session) => {
            let view = lab.views.entry(session.channel).or_default();
            if response.hovered() {
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll != 0.0 {
                    view.zoom_by((scroll / 200.0).exp());
                }
            }
            if response.dragged() && (!chrome || lab.editor.tool == editing::Tool::Navigate) {
                view.pan += response.drag_delta();
            }
            if response.double_clicked() {
                *view = view::StageView::default();
            }
            zoom = view.zoom;
            let content = painters::content_size(&session.model);
            let fit = view::fit(content, rect, *view);
            let painter = ui.painter_at(rect);
            let colour = session.settings.colour;
            painters::paint(&painter, &fit, &session.model, session.kind, colour);
            cursor = response.hover_pos().and_then(|p| fit.to_cell(p));
            lab.editor.gesture(
                ui,
                &response,
                &fit,
                session,
                fx,
                chrome && !typing && lab.drawer.is_none(),
            );
        }
        None if row.is_none() => overlay::empty(ui.painter(), rect),
        None => {}
    }
    if chrome {
        overlay::chrome(ui.painter(), rect, data, cursor, zoom);
    }
    StageRect {
        managed: true,
        min: bevy::math::Vec2::new(rect.min.x, rect.min.y),
        size: bevy::math::Vec2::new(rect.width(), rect.height()),
        scale: ui.ctx().pixels_per_point(),
        hovered: response.hovered(),
        typing,
        world_visible: gpu,
    }
}
