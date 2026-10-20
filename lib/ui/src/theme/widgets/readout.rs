//! Typographic primitives: serif/mono text, readouts and citation footnotes.
use crate::theme::{
    fonts::{SERIF, SERIF_ITALIC, family},
    tokens::*,
};
use bevy_egui::egui::{FontFamily, RichText, Ui};

/// Upright Libertinus Serif.
pub fn serif(text: impl Into<String>) -> RichText {
    RichText::new(text).family(family(SERIF))
}

/// True-italic Libertinus Serif in the dim ink.
pub fn serif_italic(text: impl Into<String>) -> RichText {
    RichText::new(text)
        .family(family(SERIF_ITALIC))
        .color(INK_DIM)
}

/// Libertinus Mono in the signal colour.
pub fn mono(text: impl Into<String>) -> RichText {
    RichText::new(text)
        .family(FontFamily::Monospace)
        .color(SIGNAL)
}

/// Tick counters read as fixed-width instrument digits.
pub fn tick_digits(tick: u64) -> String {
    format!("{tick:06}")
}

/// A mono value followed by its italic unit.
pub fn readout(ui: &mut Ui, value: &str, unit: &str, big: bool) {
    ui.horizontal(|ui| {
        ui.label(mono(value).size(if big { size::READOUT } else { size::BODY }));
        ui.label(serif_italic(unit));
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
        ui.label(serif_italic(explanation));
    });
    ui.label(serif_italic(citation).size(size::LABEL + 1.0));
    let link = ui.link(mono(format!("doi:{doi} ↗")).size(size::LABEL + 1.0));
    if link.clicked() {
        clicked = Some(format!("https://doi.org/{doi}"));
    }
    clicked
}
