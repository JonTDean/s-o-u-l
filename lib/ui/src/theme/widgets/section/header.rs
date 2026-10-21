//! Section header row: title, chevron, folded summary, action link, hover edge.
use super::Section;
use crate::theme::{
    tokens::*,
    widgets::readout::{caps, mono},
};
use bevy_egui::egui::{
    FontSelection, Id, Rect, RichText, Sense, Shape, Stroke, TextWrapMode, Ui, WidgetText, pos2,
    vec2,
};

pub struct Head {
    pub action_clicked: bool,
    pub toggled: bool,
}

fn galley(ui: &Ui, text: RichText) -> std::sync::Arc<bevy_egui::egui::Galley> {
    WidgetText::from(text).into_galley(
        ui,
        Some(TextWrapMode::Extend),
        f32::INFINITY,
        FontSelection::default(),
    )
}

/// Lay out and paint the header row. `openness` is `Some` for foldable sections.
pub fn draw(ui: &mut Ui, s: &Section, id: Id, open: bool, openness: Option<f32>) -> Head {
    let colour = if s.active { SIGNAL } else { INK_DIM };
    let title = galley(ui, caps(s.label).color(colour));
    let height = title.size().y.max(16.0);
    let title_right = title.size().x;
    let sense = if openness.is_some() {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (row, resp) = ui.allocate_exact_size(vec2(ui.available_width(), height), sense);
    let painter = ui.painter().clone();
    edge(ui, id, row);
    painter.galley(
        pos2(row.left(), row.center().y - title.size().y / 2.0),
        title,
        colour,
    );
    let mut right = row.right();
    if let Some(openness) = openness {
        right -= 10.0;
        chevron(&painter, pos2(right + 4.0, row.center().y), openness);
        right -= 8.0;
    }
    if !open && let Some(text) = &s.summary {
        let room = right - row.left() - title_right - 12.0;
        let mut shown = text.clone();
        let mut g = galley(ui, mono(shown.clone()).color(INK_DIM).size(size::LABEL));
        while g.size().x > room && shown.pop().is_some() {
            let elided = format!("{}…", shown.trim_end());
            g = galley(ui, mono(elided).color(INK_DIM).size(size::LABEL));
            if shown.is_empty() {
                break;
            }
        }
        if g.size().x > room {
            g = galley(ui, mono(String::new()).size(size::LABEL));
        }
        right -= g.size().x;
        painter.galley(pos2(right, row.center().y - g.size().y / 2.0), g, INK_DIM);
    }
    let mut action_clicked = false;
    if let Some(label) = s.action {
        let g = galley(ui, RichText::new(label).size(size::LABEL).color(INK_DIM));
        let rect = Rect::from_min_size(
            pos2(right - g.size().x, row.center().y - g.size().y / 2.0),
            g.size(),
        );
        let hit = ui.interact(rect, id.with("action"), Sense::click());
        let line = [rect.left_bottom(), rect.right_bottom()];
        painter.galley(rect.min, g, INK_DIM);
        painter.add(Shape::dotted_line(&line, INK_DIM, 3.0, 0.5));
        action_clicked = hit.clicked();
    }
    Head {
        action_clicked,
        toggled: resp.clicked() && !action_clicked,
    }
}

/// Amber 2 px bar in the margin, faded by pointer presence in last frame's rect.
fn edge(ui: &Ui, id: Id, row: Rect) {
    let hovered = ui
        .data(|d| d.get_temp::<Rect>(id))
        .zip(ui.ctx().pointer_hover_pos())
        .is_some_and(|(rect, pos)| rect.contains(pos));
    let alpha = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), hovered, 0.15);
    if alpha > 0.0 {
        let bar = Rect::from_min_size(pos2(row.left() - 8.0, row.top()), vec2(2.0, row.height()));
        let painter = ui
            .painter()
            .with_clip_rect(ui.clip_rect().expand2(vec2(10.0, 0.0)));
        painter.rect_filled(bar, 0.0, SIGNAL.gamma_multiply(alpha));
    }
}

/// Down-pointing triangle, rotated -90 degrees as the section folds.
fn chevron(painter: &bevy_egui::egui::Painter, centre: bevy_egui::egui::Pos2, openness: f32) {
    let angle = -std::f32::consts::FRAC_PI_2 * (1.0 - openness);
    let (sin, cos) = angle.sin_cos();
    let points = [vec2(-3.5, -2.0), vec2(3.5, -2.0), vec2(0.0, 2.5)]
        .map(|p| centre + vec2(p.x * cos - p.y * sin, p.x * sin + p.y * cos));
    painter.add(Shape::convex_polygon(
        points.to_vec(),
        INK_DIM,
        Stroke::NONE,
    ));
}
