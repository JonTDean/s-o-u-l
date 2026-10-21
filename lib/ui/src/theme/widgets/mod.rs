//! S4 Marginalia vocabulary: margin sections, readouts, footnotes, pills, lamps, sliders, choices.
pub mod controls;
pub mod margin;
pub mod readout;
pub use controls::{choice, lamp, lamp_colour, pill, pill_fill, slider};
pub use margin::{MARGIN_WIDTH, margin_section, note};
pub use readout::{caps, dim, footnote, mono, plain, readout, tick_digits};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::tokens::*;
    use bevy_egui::egui::{CentralPanel, Context, RawInput};
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
                choice(ui, &mut seed, &[(0, "blob"), (1, "ring")]);
                lamp(ui, Status::Running);
                pill(ui, "Pause", true, true);
                assert_eq!(
                    footnote(ui, 1, "Born with three.", "Gardner (1970).", "10.1038/x"),
                    None
                );
            });
        });
        assert!(!output.shapes.is_empty());
    }
}
