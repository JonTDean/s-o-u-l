//! Small minimap – lower‑left corner – that shows the grid texture of the
//! automaton chosen in the *Active automata* HUD list.  When nothing is
//! selected, it falls back to “first automaton” or a status label.

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, TextureId},
    EguiContexts,
};
use computational_intelligence::registry::AutomataRegistry;
use engine_core::events::AutomatonId;

use crate::rendering::active::plugin::AutomataRenderMap;

/* --------------------------------------------------------------------- */

/// Runtime cache mapping the currently‑displayed Bevy `Image` handle to the
/// `egui` texture‑ID under which it was registered.
#[derive(Default)]
pub struct Cache(Option<(Handle<Image>, TextureId)>);

#[derive(Resource, Default)]
pub struct MinimapSelection(pub Option<AutomatonId>);

/* --------------------------------------------------------------------- */

pub fn minimap_overlay(
    automata:    Res<AutomataRegistry>,
    render_map:  Res<AutomataRenderMap>,
    mut egui_ctx: EguiContexts<'_, '_>,
    mut cache:    Local<Cache>,
    sel:          Res<MinimapSelection>,
) {
    /* -------------------------------------------------------------- */
    /* 1 – Decide which automaton (texture) the minimap should show.  */
    /* -------------------------------------------------------------- */
    let target_id = sel.0.or_else(|| render_map.map.keys().next().copied());

    let some_entry = target_id.and_then(|id| render_map.map.get(&id));
    let tex_handle = some_entry.map(|(_, handle, _)| handle);

    /* -------------------------------------------------------------- */
    /* 2 – Sync egui image cache whenever the target texture changes. */
    /* -------------------------------------------------------------- */
    let tex_id = if let Some(handle) = tex_handle {
        match cache.0 {
            Some((ref cached, id)) if *cached == *handle => Some(id), // unchanged
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
        if let Some((old, _)) = cache.0.take() {
            egui_ctx.remove_image(&old);
        }
        None
    };

    /* -------------------------------------------------------------- */
    /* 3 – Determine display size (≤150 px, keep aspect ratio).       */
    /* -------------------------------------------------------------- */
    let (disp_w, disp_h) = target_id
        .and_then(|id| automata.get(id))
        .map(|info| {
            let (w, h) = match &info.grid {
                engine_core::engine::grid::GridBackend::Dense(g)  => (g.size.x as f32, g.size.y as f32),
                engine_core::engine::grid::GridBackend::Sparse(_) => (512.0, 512.0),
            };
            const MAX: f32 = 150.0;
            if w >= h { (MAX, MAX * h / w) } else { (MAX * w / h, MAX) }
        })
        .unwrap_or((150.0, 150.0));

    /* -------------------------------------------------------------- */
    /* 4 – Draw the overlay.                                          */
    /* -------------------------------------------------------------- */
    let Ok(ctx) = egui_ctx.ctx_mut() else { return };

    egui::Area::new("minimap_area".into())
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .show(ctx, |ui| {
            ui.collapsing("Minimap", |ui| {
                match tex_id {
                    Some(id) => { ui.image((id, egui::vec2(disp_w, disp_h))); }
                    None if automata.list().is_empty() =>
                        { ui.label("(no automata)"); }
                    None =>
                        { ui.label("(select an automaton in the list)"); }
                }
            });
        });
}
