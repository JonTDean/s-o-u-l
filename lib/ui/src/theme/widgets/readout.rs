//! Typographic primitives: sans/mono text, readouts and citation footnotes.
use crate::theme::{
    fonts::{MONO_MEDIUM, SANS, SANS_SEMIBOLD, family},
    tokens::*,
};
use bevy_egui::egui::{FontFamily, RichText, Ui};

/// Plex Sans in the default ink.
pub fn plain(text: impl Into<String>) -> RichText {
    RichText::new(text).family(family(SANS))
}

/// Plex Sans in the dim ink: labels, captions, units.
pub fn dim(text: impl Into<String>) -> RichText {
    plain(text).color(INK_DIM)
}

/// Uppercase tracked label in the dim ink.
pub fn caps(text: impl Into<String>) -> RichText {
    RichText::new(text.into().to_uppercase())
        .family(family(SANS_SEMIBOLD))
        .size(size::LABEL)
        .extra_letter_spacing(1.0)
        .color(INK_DIM)
}

/// Plex Mono in the signal colour.
pub fn mono(text: impl Into<String>) -> RichText {
    RichText::new(text)
        .family(FontFamily::Monospace)
        .color(SIGNAL)
}

/// Tick counters read as fixed-width instrument digits.
pub fn tick_digits(tick: u64) -> String {
    format!("{tick:06}")
}

/// A mono value followed by its dim unit.
pub fn readout(ui: &mut Ui, value: &str, unit: &str, big: bool) {
    ui.horizontal(|ui| {
        ui.label(mono(value).family(family(MONO_MEDIUM)).size(if big {
            size::READOUT
        } else {
            size::BODY
        }));
        ui.label(dim(unit));
    });
}

/// Superscript-numbered note with citation and DOI link; returns the URL when clicked.
pub fn footnote(
    ui: &mut Ui,
    number: u32,
    explanation: &str,
    citation: &str,
    doi: &str,
) -> Option<String> {
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        ui.label(mono(number.to_string()).size(size::LABEL).raised());
        ui.label(dim(explanation));
    });
    ui.label(dim(citation).size(size::LABEL + 1.0));
    let link = ui.link(mono(format!("doi:{doi} ↗")).size(size::LABEL + 1.0));
    if link.clicked() {
        clicked = Some(format!("https://doi.org/{doi}"));
    }
    clicked
}
