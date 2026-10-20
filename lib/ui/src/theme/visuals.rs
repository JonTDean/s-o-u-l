//! Global egui style: borderless surfaces, Instrument colours, Libertinus type scale.
use super::{
    fonts::{SERIF, definitions, family},
    tokens::*,
};
use bevy_egui::egui::{
    self, CornerRadius, FontFamily, FontId, Shadow, Stroke, Style, TextStyle, Visuals, vec2,
};

/// Legacy menu bottom padding.
pub const BOTTOM_PAD: f32 = 20.0;

/// Full-screen menu background.
pub fn fullscreen_bg() -> egui::Frame {
    egui::Frame::new().fill(BG)
}

/// Side panel background for legacy menus.
pub fn panel_bg() -> egui::Frame {
    egui::Frame::new().fill(PANEL)
}

/// Apply the Instrument style to `style`.
pub fn apply(style: &mut Style) {
    style.text_styles = [
        (
            TextStyle::Small,
            FontId::new(size::LABEL, FontFamily::Proportional),
        ),
        (
            TextStyle::Body,
            FontId::new(size::BODY, FontFamily::Proportional),
        ),
        (
            TextStyle::Button,
            FontId::new(size::BODY, FontFamily::Proportional),
        ),
        (TextStyle::Heading, FontId::new(size::TITLE, family(SERIF))),
        (
            TextStyle::Monospace,
            FontId::new(size::BODY, FontFamily::Monospace),
        ),
    ]
    .into();
    let mut v = Visuals::dark();
    v.panel_fill = PANEL;
    v.window_fill = PANEL;
    v.extreme_bg_color = BG;
    v.faint_bg_color = RAISED;
    v.window_stroke = Stroke::NONE;
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.override_text_color = Some(INK);
    v.hyperlink_color = SIGNAL;
    v.selection.bg_fill = SIGNAL_BG;
    v.selection.stroke = Stroke::new(1.0, SIGNAL);
    let w = &mut v.widgets;
    for state in [
        &mut w.noninteractive,
        &mut w.inactive,
        &mut w.hovered,
        &mut w.active,
        &mut w.open,
    ] {
        state.bg_stroke = Stroke::NONE;
        state.corner_radius = CornerRadius::same(4);
    }
    w.noninteractive.fg_stroke = Stroke::new(1.0, INK);
    (w.inactive.weak_bg_fill, w.inactive.bg_fill) = (RAISED, TRACK);
    (w.hovered.weak_bg_fill, w.hovered.bg_fill) = (TRACK, TRACK);
    (w.active.weak_bg_fill, w.active.bg_fill) = (SIGNAL_BG, SIGNAL);
    w.active.fg_stroke = Stroke::new(1.0, SIGNAL);
    w.hovered.fg_stroke = Stroke::new(1.0, INK_HI);
    style.visuals = v;
    style.spacing.item_spacing = vec2(8.0, 6.0);
    style.spacing.button_padding = vec2(10.0, 4.0);
}

/// Install fonts and style on a context once.
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(definitions());
    ctx.style_mut(apply);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_egui::egui::Style;

    #[test]
    fn no_widget_or_window_draws_a_hairline() {
        let mut style = Style::default();
        apply(&mut style);
        let w = &style.visuals.widgets;
        for state in [
            &w.noninteractive,
            &w.inactive,
            &w.hovered,
            &w.active,
            &w.open,
        ] {
            assert_eq!(state.bg_stroke.width, 0.0);
        }
        assert_eq!(style.visuals.window_stroke.width, 0.0);
        assert_eq!(style.visuals.panel_fill, PANEL);
        assert_eq!(style.visuals.override_text_color, Some(INK));
    }

    #[test]
    fn type_scale_uses_spec_sizes() {
        let mut style = Style::default();
        apply(&mut style);
        assert_eq!(style.text_styles[&TextStyle::Body].size, size::BODY);
        assert_eq!(style.text_styles[&TextStyle::Heading].family, family(SERIF));
        assert_eq!(style.text_styles[&TextStyle::Small].size, size::LABEL);
    }
}
