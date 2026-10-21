//! Segmented control: one bordered track of equal-width, wrapping segments.
use crate::theme::{fonts::SANS, fonts::family, tokens::*};
use bevy_egui::egui::{
    Color32, CursorIcon, FontId, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2,
};

/// Border of the selected segment: a darkened amber.
const SELECTED_BORDER: Color32 = Color32::from_rgb(0x6a, 0x4f, 0x22);
const SEGMENT_HEIGHT: f32 = 24.0;
const PAD: f32 = 2.0;
const GAP: f32 = 2.0;

/// Columns per row and segment width for `count` segments in `available` pixels.
pub fn segment_layout(available: f32, widest: f32, count: usize) -> (usize, f32) {
    let fit = ((available - 2.0 * PAD + GAP) / (widest + 12.0 + GAP)).floor();
    let per_row = (fit.max(1.0) as usize).clamp(1, count.max(1));
    let width = (available - 2.0 * PAD - GAP * (per_row - 1) as f32) / per_row as f32;
    (per_row, width)
}

/// Segmented choice; returns true only when a different option was clicked.
pub fn segmented<T: PartialEq + Copy>(ui: &mut Ui, current: &mut T, options: &[(T, &str)]) -> bool {
    let font = FontId::new(size::BODY, family(SANS));
    let galleys: Vec<_> = options
        .iter()
        .map(|(_, label)| {
            ui.painter()
                .layout_no_wrap((*label).into(), font.clone(), Color32::PLACEHOLDER)
        })
        .collect();
    let widest = galleys.iter().map(|g| g.size().x).fold(0.0, f32::max);
    let space = ui.available_width();
    // An unbounded parent gives no width to wrap against: lay out on a single row.
    let available = if space.is_finite() && space < 10_000.0 {
        space
    } else {
        options.len() as f32 * (widest + 12.0 + GAP) - GAP + 2.0 * PAD
    };
    let (per_row, width) = segment_layout(available, widest, options.len());
    let rows = options.len().div_ceil(per_row).max(1);
    let height = rows as f32 * SEGMENT_HEIGHT + (rows - 1) as f32 * GAP + 2.0 * PAD;
    let base = ui.next_auto_id();
    let (track, _) = ui.allocate_exact_size(vec2(available, height), Sense::hover());
    ui.painter()
        .rect(track, 6.0, BG, Stroke::new(1.0, LINE), StrokeKind::Inside);
    let mut clicked = None;
    for (index, ((value, _), galley)) in options.iter().zip(galleys).enumerate() {
        let (row, col) = (index / per_row, index % per_row);
        let min = track.min
            + vec2(
                PAD + col as f32 * (width + GAP),
                PAD + row as f32 * (SEGMENT_HEIGHT + GAP),
            );
        let rect = Rect::from_min_size(min, vec2(width, SEGMENT_HEIGHT));
        let response = ui.interact(rect, base.with(index), Sense::click());
        let on = *current == *value;
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        let hover = ui
            .ctx()
            .animate_bool_with_time(response.id, response.hovered(), 0.15);
        let text = if on {
            ui.painter().rect(
                rect,
                4.0,
                SIGNAL_BG,
                Stroke::new(1.0, SELECTED_BORDER),
                StrokeKind::Inside,
            );
            SIGNAL
        } else {
            ui.painter()
                .rect_filled(rect, 4.0, BG.lerp_to_gamma(RAISED, hover));
            INK_DIM.lerp_to_gamma(INK, hover)
        };
        let at = pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        );
        ui.painter().galley(at, galley, text);
        if response.clicked() && !on {
            clicked = Some(*value);
        }
    }
    if let Some(value) = clicked {
        *current = value;
    }
    clicked.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_egui::egui::{CentralPanel, Context, Event, PointerButton, RawInput, pos2};

    #[test]
    fn segment_layout_fits_three_at_default_width() {
        assert_eq!(
            segment_layout(286.0, 80.0, 3),
            (3, (286.0 - 4.0 - 4.0) / 3.0)
        );
    }

    #[test]
    fn segment_layout_wraps_at_minimum_width() {
        let (per_row, width) = segment_layout(206.0, 60.0, 5);
        assert_eq!(per_row, 2);
        assert!(width >= 60.0);
        assert!(per_row as f32 * width + 2.0 * (per_row - 1) as f32 <= 206.0 - 4.0);
    }

    #[test]
    fn segment_layout_never_returns_zero_columns() {
        assert_eq!(segment_layout(10.0, 200.0, 4).0, 1);
    }

    /// Run one pass, pressing/releasing at `events`, returning (changed, origin).
    fn pass(ctx: &Context, current: &mut i32, events: Vec<Event>) -> (bool, f32, f32) {
        let mut out = (false, 0.0, 0.0);
        let input = RawInput {
            events,
            ..Default::default()
        };
        let _ = ctx.run(input, |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                let origin = ui.cursor().min;
                let w = ui.available_width();
                out = (
                    segmented(ui, current, &[(0, "a"), (1, "b"), (2, "c")]),
                    origin.x,
                    origin.y,
                );
                out.1 = origin.x + 2.0 + 2.5 * segment_layout(w, 0.0, 3).1 + 4.0;
                out.2 = origin.y + 2.0 + SEGMENT_HEIGHT / 2.0;
            });
        });
        out
    }

    fn click(ctx: &Context, current: &mut i32, target: Option<(f32, f32)>) -> bool {
        let (_, x, y) = pass(ctx, current, vec![]);
        let (x, y) = target.unwrap_or((x, y));
        let at = pos2(x, y);
        let press = |pressed| Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        pass(ctx, current, vec![Event::PointerMoved(at)]);
        pass(ctx, current, vec![press(true)]);
        pass(ctx, current, vec![press(false)]).0
    }

    #[test]
    fn segmented_click_changes_only_on_new_value() {
        let ctx = Context::default();
        crate::theme::visuals::install(&ctx);
        let mut current = 0;
        assert!(click(&ctx, &mut current, None));
        assert_eq!(current, 2);
        assert!(!click(&ctx, &mut current, None));
        assert_eq!(current, 2);
    }

    #[test]
    fn segment_text_uses_selected_and_dim_colours() {
        let ctx = Context::default();
        crate::theme::visuals::install(&ctx);
        let out = ctx.run(RawInput::default(), |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                segmented(ui, &mut 0, &[(0, "a"), (1, "b")]);
            });
        });
        let colour = |text: &str| {
            out.shapes.iter().find_map(|s| match &s.shape {
                bevy_egui::egui::Shape::Text(t) if t.galley.text() == text => {
                    Some(t.fallback_color)
                }
                _ => None,
            })
        };
        assert_eq!(colour("a"), Some(SIGNAL));
        assert_eq!(colour("b"), Some(INK_DIM));
    }
}
