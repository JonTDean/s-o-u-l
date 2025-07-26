//! Small minimap – lower‑left corner – that shows the grid texture of the
//! *first* (and only) running automaton.  If no automata are active, or if
//! several are running at once, we fall back to a short status label.

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, TextureId},
    EguiContexts,
};
use computational_intelligence::registry::AutomataRegistry;

use crate::rendering::active::plugin::AutomataRenderMap;

/* --------------------------------------------------------------------- */

/// Runtime cache mapping the currently‑displayed Bevy `Image` handle to the
/// `egui` texture‑ID under which it was registered.
#[derive(Default)]
pub(crate) struct Cache(Option<(Handle<Image>, TextureId)>);

/* --------------------------------------------------------------------- */

pub fn minimap_overlay(
    automata:   Res<AutomataRegistry>,
    render_map: Res<AutomataRenderMap>,
    mut egui_ctx: EguiContexts<'_, '_>,
    mut cache:    Local<Cache>,
) {
    /* ------------------------------------------------------------------ */
    /* 1 – Pick a texture to display (only if *exactly* one automaton).   */
    /* ------------------------------------------------------------------ */
    let tex_id = if render_map.map.len() == 1 {
        if let Some((_, (_, handle, _))) = render_map.map.iter().next() {
            match cache.0 {
                Some((ref cached, id)) if *cached == *handle => Some(id),
                _ => {
                    if let Some((old, _)) = cache.0.take() {
                        egui_ctx.remove_image(&old);
                    }
                    let new_id = egui_ctx.add_image(handle.clone());
                    cache.0 = Some((handle.clone(), new_id));
                    Some(new_id)
                }
            }
        } else {
            None
        }
    } else {
        if let Some((old, _)) = cache.0.take() {
            egui_ctx.remove_image(&old);
        }
        None
    };

    /* ------------------------------------------------------------------ */
    /* 2 – Compute display size (≤150 px while preserving aspect ratio).  */
    /* ------------------------------------------------------------------ */
    let (disp_w, disp_h) = if let Some(info) = automata.list().first() {
        let (w, h) = match &info.grid {
            engine_core::engine::grid::GridBackend::Dense(g)  => (g.size.x as f32, g.size.y as f32),
            engine_core::engine::grid::GridBackend::Sparse(_) => (512.0, 512.0),
        };
        const MAX: f32 = 150.0;
        if w >= h { (MAX, MAX * h / w) } else { (MAX * w / h, MAX) }
    } else {
        (150.0, 150.0)
    };

    /* ------------------------------------------------------------------ */
    /* 3 – Draw the overlay.                                              */
    /* ------------------------------------------------------------------ */
    let Ok(ctx) = egui_ctx.ctx_mut() else { return };

    egui::Area::new("minimap_area".into())
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .show(ctx, |ui| {
            ui.collapsing("Minimap", |ui| {
                match tex_id {
                    Some(id) => { ui.image((id, egui::vec2(disp_w, disp_h))); }
                    None if automata.list().is_empty() =>
                        { ui.label("(minimap unavailable)"); }
                    None =>
                        { ui.label("(multiple automata – minimap not supported)"); }
                }
            });
        });
}
