//! Stage chrome: top-right instrument overlay, figure caption, empty state, toast.
use crate::{
    lab::{LabData, inspector::sections::arena_label, state::LabState},
    theme::{
        fonts::{SANS, SANS_SEMIBOLD, family},
        tokens::*,
        widgets::dim,
    },
};
use bevy_egui::egui::{self, Align2, FontId, Rect};

/// Arena line shown for GPU Lenia channels.
pub const GPU_ARENA: &str = "64 × 64 × 8 · zero edges · GPU";

/// Model-specific boundary description for a GPU owner.
pub fn gpu_arena(sim: &engine_core::automata::lenia::LeniaSimulation) -> String {
    sim.kernel.map_or_else(
        || GPU_ARENA.into(),
        |k| {
            if k.shape == simulation_kernel::core::lenia::continuous::Shape::Orbium {
                "64 × 64 · periodic · f32 Orbium XY GPU".into()
            } else {
                format!("64 × 64 × 8 · {:?} · f32 XYZ GPU", k.boundary)
            }
        },
    )
}
/// Draw the overlay and caption for the selected channel.
pub fn chrome(
    painter: &egui::Painter,
    rect: Rect,
    data: &LabData,
    cursor: Option<(i32, i32)>,
    zoom: f32,
) {
    let Some(row) = data.channels.selected_row() else {
        return;
    };
    let arena = data
        .bench
        .sessions
        .iter()
        .find(|s| s.channel == row.id)
        .map_or_else(
            || {
                data.lenia
                    .iter()
                    .find(|(id, _)| **id == row.id)
                    .map_or_else(|| GPU_ARENA.into(), |(_, s)| gpu_arena(s))
            },
            |s| arena_label(&s.model, s.kind),
        );
    let cursor = cursor.map_or("—".into(), |(x, y)| format!("({x}, {y})"));
    let lines = format!("{arena}\nzoom {:.0}% · {cursor}", zoom * 100.0);
    let corner = rect.right_top() + egui::vec2(-12.0, 10.0);
    let mono = FontId::monospace(size::LABEL + 1.0);
    painter.text(corner, Align2::RIGHT_TOP, lines, mono, INK_DIM);
    if let Some(specimen) = data.specimens.get(&row.kind) {
        let n = data.specimens.figure(&row.kind).unwrap_or(0);
        let at = rect.left_bottom() + egui::vec2(14.0, -12.0);
        let bold = FontId::new(size::BODY, family(SANS_SEMIBOLD));
        let fig = painter.text(at, Align2::LEFT_BOTTOM, format!("Fig. {n}. "), bold, INK_HI);
        let caption_font = FontId::new(size::BODY, family(SANS));
        let after = egui::pos2(fig.right(), at.y);
        let modified = data
            .bench
            .sessions
            .iter()
            .any(|s| s.channel == row.id && crate::lab::inspector::sections::customised(s));
        let modified = modified
            || data
                .lenia
                .iter()
                .any(|(id, s)| *id == row.id && s.kernel.is_some());
        let orbium = data.lenia.iter().any(|(id, s)| {
            *id == row.id
                && s.kernel.is_some_and(|k| {
                    k.shape == simulation_kernel::core::lenia::continuous::Shape::Orbium
                })
        });
        let caption = if orbium {
            "Orbium bicaudatus · polynomial periodic XY model (Chan).".into()
        } else if modified {
            format!(
                "{} (catalog figure; this channel is modified)",
                specimen.caption
            )
        } else {
            specimen.caption.clone()
        };
        painter.text(after, Align2::LEFT_BOTTOM, caption, caption_font, INK_DIM);
    }
}

/// Centred prompt when no channels are open.
pub fn empty(painter: &egui::Painter, rect: Rect) {
    let font = FontId::new(size::TITLE, family(SANS));
    let prompt = "Open a specimen from the library (L)";
    painter.text(rect.center(), Align2::CENTER_CENTER, prompt, font, INK_DIM);
}

/// Bottom-centre plain toast that expires after its duration.
pub fn toast(ctx: &egui::Context, lab: &mut LabState, dt: f32) {
    let Some((message, remaining)) = lab.toast.as_mut() else {
        return;
    };
    *remaining -= dt;
    if *remaining <= 0.0 {
        lab.toast = None;
        return;
    }
    let message = message.clone();
    egui::Area::new("lab.toast".into())
        .anchor(Align2::CENTER_BOTTOM, [0.0, -56.0])
        .order(egui::Order::Tooltip)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(RAISED)
                .corner_radius(6)
                .inner_margin(10)
                .show(ui, |ui| ui.label(dim(message).color(INK)));
        });
}
