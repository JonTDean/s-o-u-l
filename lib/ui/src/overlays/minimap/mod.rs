//! Small minimap rendered in the lower-left corner of the screen.
//
//! … (module-level docs unchanged) …

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, StrokeKind, TextureId},
    EguiContexts,
};
use engine_common::controls::camera::WorldCamera;
use engine_core::{events::AutomatonId, prelude::AutomataRegistry};
use engine_render::render::minimap::MinimapTextures;

/* ─────────────────────────────────────────────────────────────── */
/* Resources                                                      */
/* ─────────────────────────────────────────────────────────────── */

#[derive(Default)]
pub struct Cache(Option<(Handle<Image>, TextureId)>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct MinimapSelection(pub Option<AutomatonId>);

/* ─────────────────────────────────────────────────────────────── */
/* System                                                          */
/* ─────────────────────────────────────────────────────────────── */

#[allow(clippy::too_many_arguments)]
pub fn minimap_overlay(
    automata:  Res<AutomataRegistry>,
    minimap:   Res<MinimapTextures>,
    mut egui_ctx: EguiContexts<'_, '_>,
    mut cache:   Local<Cache>,
    sel:         Res<MinimapSelection>,
    cam_q:       Query<(&GlobalTransform, &Projection), With<WorldCamera>>,
    win_q:       Query<&Window>,
) {
    /* 1 ─ decide which automaton to display ----------------------- */
    let target_id = sel.0.or_else(|| automata.list().first().map(|a| a.id));

    let tex_handle = target_id
        .and_then(|id| minimap.0.get(&id))
        .map(|e| e.texture.clone());

    /* 2 ─ keep egui’s texture cache in sync ----------------------- */
    let tex_id = match &tex_handle {
        Some(h) => match cache.0 {
            Some((ref cached, id)) if cached == h => Some(id),
            _ => {
                if let Some((old, _)) = cache.0.take() {
                    egui_ctx.remove_image(&old);
                }
                let new_id = egui_ctx.add_image(h.clone());
                cache.0 = Some((h.clone(), new_id));
                Some(new_id)
            }
        },
        None => {
            if let Some((old, _)) = cache.0.take() {
                egui_ctx.remove_image(&old);
            }
            None
        }
    };

    /* 3 ─ pick display size -------------------------------------- */
    const MAX: f32 = 150.0;
    let (disp_w, disp_h) = target_id
        .and_then(|id| automata.get(id))
        .map(|info| {
            let (w, h) = (info.slice.size.x as f32, info.slice.size.y as f32);
            if w >= h { (MAX, MAX * h / w) } else { (MAX * w / h, MAX) }
        })
        .unwrap_or((MAX, MAX));

    /* 4 ─ draw panel --------------------------------------------- */
    let Ok(ctx) = egui_ctx.ctx_mut() else { return };

    egui::Area::new("minimap_area".into())
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .show(ctx, |ui| {
            ui.collapsing("Minimap", |ui| {
                match tex_id {
                    Some(id) => {
                        let resp = ui.image((id, egui::vec2(disp_w, disp_h)));

                        // ── 4a · world bounds ──────────────────────
                        ui.painter().rect_stroke(
                            resp.rect,
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::GRAY),
                            StrokeKind::Middle,
                        );

                        // ── 4b · camera frustum ────────────────────
                        if let (Ok((xf, Projection::Orthographic(o))), Ok(win)) =
                            (cam_q.single(), win_q.single())
                        {
                            let half_w = win.width()  * 0.5 * o.scale;
                            let half_h = win.height() * 0.5 * o.scale;
                            let centre = xf.translation().truncate();
                            let half  = Vec2::new(half_w, half_h);
                            let cam_min = centre - half;
                            let cam_max = centre + half;

                            if let Some(info) = target_id.and_then(|id| automata.get(id)) {
                                let w   = info.slice.size.x as f32;
                                let h   = info.slice.size.y as f32;
                                let off = info.world_offset.truncate();
                                let size = Vec2::new(w, h) * info.voxel_size;

                                let rel_min = ((cam_min - off) / size).clamp(Vec2::ZERO, Vec2::ONE);
                                let rel_max = ((cam_max - off) / size).clamp(Vec2::ZERO, Vec2::ONE);

                                let tl = egui::pos2(
                                    resp.rect.left() + resp.rect.width()  * rel_min.x,
                                    resp.rect.top()  + resp.rect.height() * (1.0 - rel_max.y),
                                );
                                let br = egui::pos2(
                                    resp.rect.left() + resp.rect.width()  * rel_max.x,
                                    resp.rect.top()  + resp.rect.height() * (1.0 - rel_min.y),
                                );
                                ui.painter().rect_stroke(
                                    egui::Rect::from_two_pos(tl, br),
                                    0.0,
                                    egui::Stroke::new(2.0, egui::Color32::LIGHT_GREEN),
                                    StrokeKind::Inside,
                                );
                            }
                        }

                        // ── 4c · automaton centre marker ───────────
                        if let Some(info) = target_id.and_then(|id| automata.get(id)) {
                            let w   = info.slice.size.x as f32;
                            let h   = info.slice.size.y as f32;
                            let off = info.world_offset.truncate();
                            let size = Vec2::new(w, h) * info.voxel_size;
                            let centre = off + size * 0.5;

                            let rel = ((centre - off) / size).clamp(Vec2::ZERO, Vec2::ONE);
                            let dot = egui::pos2(
                                resp.rect.left() + resp.rect.width()  * rel.x,
                                resp.rect.top()  + resp.rect.height() * (1.0 - rel.y),
                            );
                            ui.painter().circle_filled(dot, 3.0, egui::Color32::RED);
                            // `circle_filled` is the idiomatic way to draw a marker in egui :contentReference[oaicite:0]{index=0}
                        }
                    }
                    None if automata.list().is_empty() => { ui.label("(no automatons)"); }
                    None => { ui.label("(select an automaton)"); }
                }
            });
        });
}
