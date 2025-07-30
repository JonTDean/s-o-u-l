//! Small minimap rendered in the lower‑left corner of the screen.
//!
//! The panel shows the active automaton’s texture and, when available,
//! overlays both
//!
//! * a **light‑grey world‑bounds rectangle**, and
//! * a **bright‑green camera‑frustum rectangle**
//!
//! so the player can immediately see what portion of the automaton is
//! on screen.
//!
//! ### Thread‑safety
//!
//! This module contains no interior mutability outside Bevy’s ECS
//! scheduling and therefore runs safely in parallel with any other
//! system that obeys normal borrowing rules.
//!
//! ### Public API
//!
//! * [`MinimapSelection`] – resource storing the automaton currently
//!   picked by the user.
//! * [`minimap_overlay`]   – main egui draw routine (add to
//!   `EguiPrimaryContextPass`).

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, StrokeKind, TextureId},
    EguiContexts,
};

use engine_core::{events::AutomatonId, prelude::AutomataRegistry};
use engine_render::{prelude::AutomataRenderMap, WorldCamera};
use simulation_kernel::grid::GridBackend;

/* --------------------------------------------------------------------- */
/*                                Resources                              */
/* --------------------------------------------------------------------- */

/// Runtime cache mapping the Bevy [`Image`] handle currently shown in the
/// minimap to the **egui** [`TextureId`] under which it was registered.
#[derive(Default)]
pub struct Cache(Option<(Handle<Image>, TextureId)>);

/// User‑controlled minimap target (`None` → first automaton available).
#[derive(Resource, Default)]
pub struct MinimapSelection(pub Option<AutomatonId>);

/* --------------------------------------------------------------------- */
/*                            Main draw system                            */
/* --------------------------------------------------------------------- */

/// Renders the minimap panel every frame.
///
/// * **texture selection** – chooses which automaton slice to display
///   based on [`MinimapSelection`] (falls back gracefully when no
///   automata are alive).
/// * **cache sync**        – keeps egui’s texture cache in lock‑step
///   with the chosen Bevy `Handle<Image>`.
/// * **overlay rendering** – draws the world‑bounds rectangle **and**
///   the camera‑frustum rectangle on top of the minimap.
///
/// The system is entirely **side‑effect‑free** outside the egui
/// context, so Bevy may schedule it on any available thread.
#[allow(clippy::too_many_arguments)]
pub fn minimap_overlay(
    automata:     Res<AutomataRegistry>,
    render_map:   Res<AutomataRenderMap>,
    mut egui_ctx: EguiContexts<'_, '_>,
    mut cache:    Local<Cache>,
    sel:          Res<MinimapSelection>,
    cam_q:        Query<(&GlobalTransform, &Projection), With<WorldCamera>>,
    win_q:        Query<&Window>,
) {
    /* ───────────────────── 1 – choose texture ─────────────────────── */
    let target_id  = sel.0.or_else(|| render_map.map.keys().next().copied());
    let tex_handle = target_id.and_then(|id| render_map.map.get(&id)).map(|(_, h, _)| h);

    /* ───────────────────── 2 – sync egui cache ────────────────────── */
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

    /* ───────────────────── 3 – display size ───────────────────────── */
    const MAX: f32 = 150.0;
    let (disp_w, disp_h) = target_id
        .and_then(|id| automata.get(id))
        .map(|info| {
            let (w, h) = match &info.grid {
                GridBackend::Dense(g)  => (g.size.x as f32, g.size.y as f32),
                GridBackend::Sparse(_) => (512.0, 512.0),
            };
            if w >= h { (MAX, MAX * h / w) } else { (MAX * w / h, MAX) }
        })
        .unwrap_or((MAX, MAX));

    /* ───────────────────── 4 – draw panel ─────────────────────────── */
    let Ok(ctx) = egui_ctx.ctx_mut() else { return };

    egui::Area::new("minimap_area".into())
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .show(ctx, |ui| {
            ui.collapsing("Minimap", |ui| {
                // NOTE: we deliberately ignore the response so the closure returns `()`.
                let _resp = match tex_id {
                    /* ------------------------------------------------- */
                    /* 1 ░ actual texture present                       */
                    /* ------------------------------------------------- */
                    Some(id) => {
                        // Draw the texture first so any overlay appears on top.
                        let resp = ui.image((id, egui::vec2(disp_w, disp_h)));

                        // 1‑a ░ always draw world bounds (full texture).
                        ui.painter().rect_stroke(
                            resp.rect,
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::GRAY),
                            StrokeKind::Middle,
                        );

                        // 1‑b ░ draw camera frustum if all inputs available.
                        if let (Ok((xf, Projection::Orthographic(o))), Ok(win)) =
                            (cam_q.single(), win_q.single())
                        {
                            /* world‑space frustum AABB */
                            let half_w  = win.width()  * 0.5 * o.scale;
                            let half_h  = win.height() * 0.5 * o.scale;
                            let centre  = xf.translation().truncate();
                            let cam_min = centre - Vec2::new(half_w, half_h);
                            let cam_max = centre + Vec2::new(half_w, half_h);

                            /* automaton slice AABB */
                            let Some(info) = target_id.and_then(|id| automata.get(id)) else { return };
                            let (w, h) = match &info.grid {
                                GridBackend::Dense(g)  => (g.size.x as f32, g.size.y as f32),
                                GridBackend::Sparse(_) => (512.0, 512.0),
                            };
                            let off  = info.world_offset;
                            let size = Vec2::new(w, h) * info.cell_size;

                            /* normalise to [0, 1] and clamp */
                            let rel_min = ((cam_min - off) / size).clamp(Vec2::ZERO, Vec2::ONE);
                            let rel_max = ((cam_max - off) / size).clamp(Vec2::ZERO, Vec2::ONE);

                            /* egui’s y‑axis points down → flip Y */
                            let top_left = egui::pos2(
                                resp.rect.left() + resp.rect.width()  * rel_min.x,
                                resp.rect.top()  + resp.rect.height() * (1.0 - rel_max.y),
                            );
                            let bottom_right = egui::pos2(
                                resp.rect.left() + resp.rect.width()  * rel_max.x,
                                resp.rect.top()  + resp.rect.height() * (1.0 - rel_min.y),
                            );

                            // Ensure positive rect regardless of coordinate ordering.
                            let frustum_rect = egui::Rect::from_two_pos(top_left, bottom_right);

                            ui.painter().rect_stroke(
                                frustum_rect,
                                0.0,
                                egui::Stroke::new(2.0, egui::Color32::LIGHT_GREEN),
                                StrokeKind::Inside,
                            );
                        }

                        resp // still assign to `_resp` so we can use it later if desired
                    }

                    /* ------------------------------------------------- */
                    /* 2 ░ fall‑back messages                            */
                    /* ------------------------------------------------- */
                    None if automata.list().is_empty() => ui.label("(no automata)"),
                    None                                => ui.label("(select an automaton in the list)"),
                };

            });
        });
}
