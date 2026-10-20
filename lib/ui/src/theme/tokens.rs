//! Instrument palette and type scale; the only colour constants the UI uses.
use bevy_egui::egui::Color32;
use engine_core::automata::channel::Srgb;

/// Window and stage background.
pub const BG: Color32 = Color32::from_rgb(0x0d, 0x0f, 0x10);
/// Rail, tabs, inspector and transport surfaces.
pub const PANEL: Color32 = Color32::from_rgb(0x12, 0x14, 0x15);
/// Pills and inactive controls.
pub const RAISED: Color32 = Color32::from_rgb(0x19, 0x1c, 0x1d);
/// Slider tracks and grips.
pub const TRACK: Color32 = Color32::from_rgb(0x23, 0x27, 0x28);
/// Body text.
pub const INK: Color32 = Color32::from_rgb(0xc9, 0xc4, 0xb4);
/// Titles, active tab, slider knobs.
pub const INK_HI: Color32 = Color32::from_rgb(0xec, 0xe6, 0xd4);
/// Margin labels, captions, notes.
pub const INK_DIM: Color32 = Color32::from_rgb(0x8a, 0x84, 0x76);
/// Values, selection, active state.
pub const SIGNAL: Color32 = Color32::from_rgb(0xff, 0xb5, 0x47);
/// Selected rows.
pub const SIGNAL_BG: Color32 = Color32::from_rgb(0x2a, 0x24, 0x16);
/// Running lamp.
pub const OK: Color32 = Color32::from_rgb(0x7b, 0xd8, 0x8f);
/// Halted lamp and errors.
pub const FAULT: Color32 = Color32::from_rgb(0xff, 0x6b, 0x5b);
/// GPU tag.
pub const INFO: Color32 = Color32::from_rgb(0x6f, 0xb7, 0xff);

/// Logical-pixel type scale.
pub mod size {
    /// Tracked-caps labels.
    pub const LABEL: f32 = 9.0;
    /// Body text and controls.
    pub const BODY: f32 = 11.0;
    /// Titles and tab names.
    pub const TITLE: f32 = 15.0;
    /// Inspector readouts.
    pub const READOUT: f32 = 24.0;
    /// Presenter HUD readouts.
    pub const HUD: f32 = 40.0;
}

/// Channel colour as an egui colour.
pub fn color(srgb: Srgb) -> Color32 {
    let [r, g, b] = srgb.0;
    Color32::from_rgb(r, g, b)
}

/// WCAG 2 contrast ratio between two opaque colours.
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let luminance = |c: Color32| {
        let [r, g, b] = Srgb([c.r(), c.g(), c.b()]).linear();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    };
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_tokens_meet_wcag_aa_on_both_surfaces() {
        for surface in [BG, PANEL] {
            for (name, ink) in [("ink", INK), ("ink_dim", INK_DIM), ("signal", SIGNAL)] {
                let ratio = contrast(ink, surface);
                assert!(ratio >= 4.5, "{name} on {surface:?}: {ratio:.2}");
            }
        }
    }

    #[test]
    fn signal_matches_the_channel_default_colour() {
        assert_eq!(color(engine_core::automata::channel::SIGNAL), SIGNAL);
    }
}
