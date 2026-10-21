//! Bordered pills and status lamps; segmented controls and sliders live beside.
use crate::theme::{
    fonts::{SANS, SANS_SEMIBOLD, family},
    tokens::*,
};
use bevy_egui::egui::{
    self, Color32, FontFamily, FontId, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2,
};
use engine_core::automata::channel::Status;

pub mod segmented;
pub use segmented::{segment_layout, segmented};
pub mod slider;
pub use slider::{slider, value_box};

/// Pill fill: amber only for an enabled primary action.
pub fn pill_fill(primary: bool, enabled: bool) -> Color32 {
    if primary && enabled { SIGNAL } else { RAISED }
}

/// Border of the key chip on a primary pill: a darker amber.
const CHIP_PRIMARY: Color32 = Color32::from_rgb(0xc9, 0x8a, 0x2c);
/// Primary fill on hover.
const PRIMARY_HOVER: Color32 = Color32::from_rgb(0xff, 0xc6, 0x6e);
const PILL_HEIGHT: f32 = 24.0;

/// Bordered action button.
pub fn pill(ui: &mut Ui, label: &str, primary: bool, enabled: bool) -> Response {
    draw_pill(ui, label, None, primary, enabled)
}

/// Pill with a trailing keyboard-shortcut chip.
pub fn pill_with_key(
    ui: &mut Ui,
    label: &str,
    key: &str,
    primary: bool,
    enabled: bool,
) -> Response {
    draw_pill(ui, label, Some(key), primary, enabled)
}

fn draw_pill(
    ui: &mut Ui,
    label: &str,
    key: Option<&str>,
    primary: bool,
    enabled: bool,
) -> Response {
    let live = primary && enabled;
    let text_colour = if live { BG } else { INK };
    let font = FontId::new(size::BODY, family(if live { SANS_SEMIBOLD } else { SANS }));
    let galley = ui
        .painter()
        .layout_no_wrap(label.into(), font, Color32::PLACEHOLDER);
    let chip = key.map(|key| {
        let font = FontId::new(size::LABEL, FontFamily::Monospace);
        ui.painter()
            .layout_no_wrap(key.into(), font, Color32::PLACEHOLDER)
    });
    let chip_width = chip.as_ref().map_or(0.0, |g| g.size().x + 10.0 + 6.0);
    let desired = vec2(galley.size().x + chip_width + 20.0, PILL_HEIGHT);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(desired, sense);
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let hover = ui
        .ctx()
        .animate_bool_with_time(response.id, enabled && response.hovered(), 0.15);
    let base = pill_fill(primary, enabled);
    let lifted = if live { PRIMARY_HOVER } else { TRACK };
    let fade = |colour: Color32| {
        if enabled {
            colour
        } else {
            colour.gamma_multiply(0.4)
        }
    };
    let fill = fade(base.lerp_to_gamma(lifted, hover));
    let border = fade(if live {
        SIGNAL.lerp_to_gamma(PRIMARY_HOVER, hover)
    } else {
        LINE.lerp_to_gamma(LINE_HI, hover)
    });
    if enabled && response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    ui.painter().rect(
        rect,
        6.0,
        fill,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );
    let nudge = if enabled && response.is_pointer_button_down_on() {
        1.0
    } else {
        0.0
    };
    let mut x = rect.min.x + 10.0;
    let centre_y = rect.center().y + nudge;
    ui.painter().galley(
        pos2(x, centre_y - galley.size().y / 2.0),
        galley.clone(),
        fade(text_colour),
    );
    x += galley.size().x;
    if let Some(chip) = chip {
        let chip_rect = Rect::from_min_size(
            pos2(x + 6.0, centre_y - (chip.size().y + 2.0) / 2.0),
            vec2(chip.size().x + 10.0, chip.size().y + 2.0),
        );
        let outline = if live { CHIP_PRIMARY } else { LINE };
        ui.painter().rect_stroke(
            chip_rect,
            3.0,
            Stroke::new(1.0, fade(outline)),
            StrokeKind::Inside,
        );
        ui.painter().galley(
            pos2(
                chip_rect.min.x + 5.0,
                chip_rect.center().y - chip.size().y / 2.0,
            ),
            chip,
            fade(text_colour),
        );
    }
    response
}

/// Lamp colour per lifecycle state.
pub fn lamp_colour(status: Status) -> Color32 {
    match status {
        Status::Running => OK,
        Status::Paused => SIGNAL,
        Status::Halted => FAULT,
    }
}

/// 7 px status lamp; running lamps carry a soft glow.
pub fn lamp(ui: &mut Ui, status: Status) {
    let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
    let colour = lamp_colour(status);
    if status == Status::Running {
        ui.painter()
            .circle_filled(rect.center(), 5.0, colour.gamma_multiply(0.25));
    }
    ui.painter().circle_filled(rect.center(), 3.5, colour);
}
