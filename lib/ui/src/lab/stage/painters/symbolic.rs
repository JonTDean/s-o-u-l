//! Tapes and stacks as large cells with the head marked; Koch as a turtle path.
use super::Fit;
use crate::theme::tokens::*;
use bevy_egui::egui::{Align2, FontId, Painter, Rect, Shape, Stroke, StrokeKind, Vec2, vec2};
use engine_core::automata::channel::Srgb;
use models::workbench::symbolic::machine::Symbolic;
use std::ops::Range;

/// Visible tape cells: the tape plus a trailing blank, windowed to `max` cells around the head.
pub fn tape_window(len: usize, head: usize, max: usize) -> Range<usize> {
    let total = len + 1;
    if total <= max {
        return 0..total;
    }
    let start = head.saturating_sub(max / 2).min(total - max);
    start..start + max
}

fn turtle(word: &str) -> Vec<Vec2> {
    let mut points = vec![Vec2::ZERO];
    let mut angle = 0.0_f32;
    for symbol in word.chars() {
        match symbol {
            'F' => points.push(*points.last().unwrap() + Vec2::angled(angle)),
            '+' => angle += std::f32::consts::FRAC_PI_3,
            '-' => angle -= std::f32::consts::FRAC_PI_3,
            _ => {}
        }
    }
    points
}

/// Bounding size of the Koch turtle path.
pub fn koch_extent(word: &str) -> Vec2 {
    let p = turtle(word);
    let start = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    let (min, max) = p.iter().fold(start, |(a, b), q| (a.min(*q), b.max(*q)));
    (max - min).max(vec2(1.0, 1.0))
}

/// Visible tape cells at most.
const MAX_CELLS: usize = 20;
/// Content width floor so short tapes keep readable cell sizes.
const MIN_CELLS: usize = 12;

/// The tape (or input) and head position; `None` for the Koch turtle.
fn tape(machine: &Symbolic) -> Option<(&[char], usize)> {
    match machine {
        Symbolic::Finite(m) => Some((&m.input, m.cursor)),
        Symbolic::Pushdown(m) => Some((&m.input, m.cursor)),
        Symbolic::Bounded(m) => Some((&m.tape, m.head)),
        Symbolic::Turing(m) => Some((&m.tape, m.head)),
        Symbolic::Koch { .. } => None,
    }
}

/// Content size: the visible tape window (6 rows for tape, stack and margins), or the Koch path.
pub fn extent(machine: &Symbolic) -> Vec2 {
    match (machine, tape(machine)) {
        (Symbolic::Koch { word, .. }, _) => koch_extent(word),
        (_, Some((cells, head))) => vec2(
            tape_window(cells.len(), head, MAX_CELLS)
                .len()
                .max(MIN_CELLS) as f32,
            6.0,
        ),
        (_, None) => vec2(MAX_CELLS as f32, 6.0),
    }
}

/// Paint a symbolic machine.
pub fn paint(painter: &Painter, fit: &Fit, machine: &Symbolic, colour: Srgb) {
    let Some((tape, head)) = tape(machine) else {
        if let Symbolic::Koch { word, .. } = machine {
            koch(painter, fit, word, colour);
        }
        return;
    };
    let window = tape_window(tape.len(), head, MAX_CELLS);
    let start = window.start;
    // Short windows sit centred in the MIN_CELLS-wide content.
    let inset = MIN_CELLS.saturating_sub(window.len()) as f32 / 2.0;
    for (slot, index) in window.enumerate() {
        let min = fit.to_screen(vec2(inset + slot as f32, 2.0));
        let rect = Rect::from_min_size(min, vec2(fit.scale * 0.9, fit.scale * 1.2));
        let at_head = index == head;
        painter.rect_filled(rect, 3.0, if at_head { SIGNAL_BG } else { RAISED });
        if at_head {
            let stroke = Stroke::new(2.0, color(colour));
            painter.rect_stroke(rect, 3.0, stroke, StrokeKind::Outside);
        }
        let glyph = tape
            .get(index)
            .map_or('␣', |c| if *c == ' ' { '␣' } else { *c });
        let ink = if glyph.is_alphanumeric() {
            color(colour)
        } else {
            INK
        };
        let font = FontId::monospace(fit.scale * 0.6);
        painter.text(rect.center(), Align2::CENTER_CENTER, glyph, font, ink);
    }
    if let Symbolic::Pushdown(m) = machine {
        // Top of stack first, hanging under the head.
        for (depth, symbol) in m.stack.iter().rev().enumerate().take(4) {
            let at = vec2(
                inset + (head - start) as f32 + 0.45,
                3.6 + depth as f32 * 0.6,
            );
            let font = FontId::monospace(fit.scale * 0.45);
            painter.text(fit.to_screen(at), Align2::CENTER_TOP, symbol, font, INK_DIM);
        }
    }
}

fn koch(painter: &Painter, fit: &Fit, word: &str, colour: Srgb) {
    let points = turtle(word);
    let min = points.iter().fold(Vec2::splat(f32::MAX), |a, b| a.min(*b));
    let screen: Vec<_> = points.iter().map(|p| fit.to_screen(*p - min)).collect();
    painter.add(Shape::line(screen, Stroke::new(1.5, color(colour))));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tapes_are_sized_to_their_visible_window() {
        use models::workbench::catalog::Kind;
        let short = Symbolic::new(Kind::Turing, "aaaa");
        assert_eq!(
            extent(&short),
            vec2(12.0, 6.0),
            "short tapes keep a 12-cell floor"
        );
        let long = Symbolic::new(Kind::Nfa, &"a".repeat(40));
        assert_eq!(extent(&long).x, 20.0);
    }

    #[test]
    fn tape_window_keeps_the_head_visible() {
        assert_eq!(
            tape_window(5, 2, 20),
            0..6,
            "short tapes show all cells plus a trailing blank"
        );
        assert_eq!(tape_window(100, 90, 20), 80..100);
        assert_eq!(tape_window(100, 3, 20), 0..20);
        assert!(
            tape_window(100, 100, 20).contains(&100),
            "a head on the trailing blank stays visible"
        );
    }
}
