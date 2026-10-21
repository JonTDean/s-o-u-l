//! S4 Marginalia vocabulary: margin sections, readouts, footnotes, pills, lamps, sliders, segmented controls.
pub mod controls;
pub mod margin;
pub mod readout;
pub mod section;
pub use controls::{
    lamp, lamp_colour, pill, pill_fill, pill_with_key, segment_layout, segmented, slider, value_box,
};
pub use margin::{MARGIN_WIDTH, margin_section, note};
pub use readout::{caps, dim, footnote, mono, plain, readout, tick_digits};
pub use section::{Section, SectionResponse, section};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::tokens::*;
    use bevy_egui::egui::{
        CentralPanel, Color32, Context, Event, PointerButton, Pos2, RawInput, Shape,
    };
    use engine_core::automata::channel::Status;

    #[test]
    fn colour_rules() {
        assert_eq!(lamp_colour(Status::Running), OK);
        assert_eq!(lamp_colour(Status::Paused), SIGNAL);
        assert_eq!(lamp_colour(Status::Halted), FAULT);
        assert_eq!(pill_fill(true, true), SIGNAL);
        assert_eq!(pill_fill(false, true), RAISED);
        assert_eq!(pill_fill(true, false), RAISED);
    }

    #[test]
    fn ticks_are_zero_padded_to_six_digits() {
        assert_eq!(tick_digits(184), "000184");
        assert_eq!(tick_digits(12_345_678), "12345678");
    }

    #[test]
    fn widgets_render_headless_without_panicking() {
        let ctx = Context::default();
        crate::theme::visuals::install(&ctx);
        let output = ctx.run(RawInput::default(), |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                margin_section(ui, "readout", true, |ui| {
                    readout(ui, "000184", "tick", true)
                });
                let mut v = 0.27;
                slider(ui, "μ", &mut v, 0.0..=1.0, false);
                let mut seed = 0;
                segmented(ui, &mut seed, &[(0, "blob"), (1, "ring")]);
                lamp(ui, Status::Running);
                pill(ui, "Pause", true, true);
                pill_with_key(ui, "Step", "S", false, true);
                pill(ui, "Off", false, false);
                assert_eq!(
                    footnote(ui, 1, "Born with three.", "Gardner (1970).", "10.1038/x"),
                    None
                );
            });
        });
        assert!(!output.shapes.is_empty());
    }

    fn pill_frame(
        ctx: &Context,
        enabled: bool,
        events: Vec<Event>,
    ) -> (bool, Option<Color32>, Pos2) {
        let mut out = (false, None, Pos2::ZERO);
        let input = RawInput {
            events,
            ..Default::default()
        };
        let shapes = ctx.run(input, |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                let r = pill(ui, "Go", false, enabled);
                out.0 = r.clicked();
                out.2 = r.rect.center();
            });
        });
        out.1 = shapes.shapes.iter().find_map(|s| match &s.shape {
            Shape::Text(t) if t.galley.text() == "Go" => Some(rendered(t)),
            _ => None,
        });
        out
    }

    #[test]
    fn disabled_pill_text_is_faded_and_ignores_clicks() {
        let ctx = Context::default();
        crate::theme::visuals::install(&ctx);
        let (_, on, _) = pill_frame(&ctx, true, vec![]);
        assert_eq!(on, Some(INK));
        let (_, off, at) = pill_frame(&ctx, false, vec![]);
        assert_eq!(off, Some(INK.gamma_multiply(0.4)));
        let press = |pressed| Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        pill_frame(&ctx, false, vec![Event::PointerMoved(at)]);
        pill_frame(&ctx, false, vec![press(true)]);
        assert!(!pill_frame(&ctx, false, vec![press(false)]).0);
        // Control: the same gesture clicks an enabled pill.
        pill_frame(&ctx, true, vec![Event::PointerMoved(at)]);
        pill_frame(&ctx, true, vec![press(true)]);
        assert!(pill_frame(&ctx, true, vec![press(false)]).0);
    }

    /// Colour a text shape really renders in: its glyph mesh must be uncoloured
    /// (PLACEHOLDER) so the fallback colour applies when tessellated.
    fn rendered(t: &bevy_egui::egui::epaint::TextShape) -> Color32 {
        let baked = t
            .galley
            .rows
            .iter()
            .flat_map(|r| r.visuals.mesh.vertices.iter());
        assert!(baked.map(|v| v.color).all(|c| c == Color32::PLACEHOLDER));
        t.fallback_color
    }
}
