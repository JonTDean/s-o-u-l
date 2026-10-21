//! Divided sections: hairline, small-caps title, optional fold, amber hover edge.
mod header;
use crate::theme::{tokens::*, widgets::readout::dim};
use bevy_egui::egui::{Rect, Stroke, Ui, collapsing_header::CollapsingState, pos2, vec2};

/// Outcome of showing a [`Section`].
pub struct SectionResponse {
    pub action_clicked: bool,
    pub open: bool,
}

/// Builder for one divided inspector or drawer section.
pub struct Section<'a> {
    label: &'a str,
    active: bool,
    fold: Option<bool>,
    summary: Option<String>,
    action: Option<&'a str>,
}

/// Start a section titled `label`; an empty label draws no header row.
pub fn section(label: &str) -> Section<'_> {
    Section {
        label,
        active: false,
        fold: None,
        summary: None,
        action: None,
    }
}

impl<'a> Section<'a> {
    /// Title in the signal colour.
    pub fn active(mut self, on: bool) -> Self {
        self.active = on;
        self
    }

    /// Make the section foldable, open on first sight when `open_by_default`.
    pub fn foldable(mut self, open_by_default: bool) -> Self {
        self.fold = Some(open_by_default);
        self
    }

    /// Text shown on the header's right while folded.
    pub fn summary(mut self, text: impl Into<String>) -> Self {
        self.summary = Some(text.into());
        self
    }

    /// Dim dotted-underline link on the header's right.
    pub fn action(mut self, label: &'a str) -> Self {
        self.action = Some(label);
        self
    }

    /// Draw the divider, header and (if open) `body`.
    pub fn show(self, ui: &mut Ui, body: impl FnOnce(&mut Ui)) -> SectionResponse {
        let id = ui.make_persistent_id(("section", self.label));
        // Nothing placed yet. Not `min_rect`: a CentralPanel pre-fills it.
        let first = ui.cursor().top() <= ui.max_rect().top();
        let top = ui.cursor().top();
        if !first {
            let rect = ui.allocate_space(vec2(ui.available_width(), 1.0)).1;
            let stroke = Stroke::new(1.0, LINE);
            ui.painter().hline(rect.x_range(), rect.center().y, stroke);
            ui.add_space(PAD);
        }
        let mut state = self
            .fold
            .map(|open| CollapsingState::load_with_default_open(ui.ctx(), id, open));
        let mut action_clicked = false;
        if !self.label.is_empty() {
            let open = state.as_ref().is_none_or(|s| s.is_open());
            let openness = state.as_ref().map(|s| s.openness(ui.ctx()));
            let head = header::draw(ui, &self, id, open, openness);
            action_clicked = head.action_clicked;
            if head.toggled
                && let Some(state) = state.as_mut()
            {
                state.toggle(ui);
            }
            ui.add_space(6.0);
        }
        match state.as_mut() {
            Some(state) => {
                state.show_body_unindented(ui, |ui| ui.vertical(body).response);
            }
            None => {
                ui.vertical(body);
            }
        }
        ui.add_space(PAD);
        let rect = Rect::from_min_max(
            pos2(ui.max_rect().left(), top),
            pos2(ui.max_rect().right(), ui.cursor().top()),
        );
        ui.data_mut(|d| d.insert_temp(id, rect));
        SectionResponse {
            action_clicked,
            open: state.as_ref().is_none_or(|s| s.is_open()),
        }
    }
}

/// Vertical padding above and below a section's content.
const PAD: f32 = 11.0;

/// Small dim note; `warn` uses the signal colour.
pub fn note(ui: &mut Ui, text: &str, warn: bool) {
    ui.label(
        dim(text)
            .size(size::BODY - 1.0)
            .color(if warn { SIGNAL } else { INK_DIM }),
    );
}

#[cfg(test)]
mod tests;
