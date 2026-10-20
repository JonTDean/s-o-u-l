//! Libertinus Serif / Sans / Mono, embedded and registered as named egui families.
use bevy_egui::egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

macro_rules! font {
    ($file:literal) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/fonts/libertinus/",
            $file
        ))
    };
}

/// Upright serif: titles, tab names.
pub const SERIF: &str = "serif";
/// True italic serif: margin labels, captions, notes.
pub const SERIF_ITALIC: &str = "serif-italic";
/// Semibold serif: figure numbers.
pub const SERIF_SEMIBOLD: &str = "serif-semibold";
/// Bold sans: primary pill labels.
pub const SANS_BOLD: &str = "sans-bold";

const FACES: [(&str, &[u8]); 6] = [
    ("libertinus-serif", font!("LibertinusSerif-Regular.otf")),
    (
        "libertinus-serif-italic",
        font!("LibertinusSerif-Italic.otf"),
    ),
    (
        "libertinus-serif-semibold",
        font!("LibertinusSerif-Semibold.otf"),
    ),
    ("libertinus-sans", font!("LibertinusSans-Regular.otf")),
    ("libertinus-sans-bold", font!("LibertinusSans-Bold.otf")),
    ("libertinus-mono", font!("LibertinusMono-Regular.otf")),
];

/// Named egui family for `name`.
pub fn family(name: &str) -> FontFamily {
    FontFamily::Name(name.into())
}

/// egui's defaults with Libertinus first in every family; defaults remain as glyph fallbacks.
pub fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    for (key, bytes) in FACES {
        fonts
            .font_data
            .insert(key.into(), Arc::new(FontData::from_static(bytes)));
    }
    let fallback = fonts.families[&FontFamily::Proportional].clone();
    let mono_fallback = fonts.families[&FontFamily::Monospace].clone();
    let chain = |first: &str, rest: &[String]| {
        std::iter::once(first.to_string())
            .chain(rest.iter().cloned())
            .collect::<Vec<_>>()
    };
    fonts.families.insert(
        FontFamily::Proportional,
        chain("libertinus-sans", &fallback),
    );
    // Libertinus Mono lacks symbols such as ␣; Libertinus Sans is the last resort.
    let mut mono = chain("libertinus-mono", &mono_fallback);
    mono.push("libertinus-sans".into());
    fonts.families.insert(FontFamily::Monospace, mono);
    for (name, key) in [
        (SERIF, "libertinus-serif"),
        (SERIF_ITALIC, "libertinus-serif-italic"),
        (SERIF_SEMIBOLD, "libertinus-serif-semibold"),
        (SANS_BOLD, "libertinus-sans-bold"),
    ] {
        fonts.families.insert(family(name), chain(key, &fallback));
    }
    fonts
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_egui::egui::{Context, FontId, RawInput};

    #[test]
    fn every_family_leads_with_libertinus_and_keeps_fallbacks() {
        let defs = definitions();
        for name in [SERIF, SERIF_ITALIC, SERIF_SEMIBOLD, SANS_BOLD] {
            let chain = &defs.families[&family(name)];
            assert!(chain[0].starts_with("libertinus"), "{name}: {chain:?}");
            assert!(
                chain.len() > 1,
                "{name} keeps egui fallbacks for missing symbols"
            );
        }
        assert!(defs.families[&FontFamily::Monospace][0].starts_with("libertinus"));
    }

    #[test]
    fn fonts_load_and_cover_lab_glyphs() {
        let ctx = Context::default();
        ctx.set_fonts(definitions());
        let _ = ctx.run(RawInput::default(), |_| {});
        let _ = ctx.run(RawInput::default(), |ctx| {
            ctx.fonts(|fonts| {
                assert!(fonts.has_glyphs(&FontId::new(12.0, family(SERIF_ITALIC)), "μσΔ⁂×"));
                assert!(fonts.has_glyphs(&FontId::new(12.0, FontFamily::Monospace), "0123456789"));
                assert!(
                    fonts.has_glyphs(&FontId::new(12.0, FontFamily::Monospace), "␣→"),
                    "Turing tables print the blank as ␣"
                );
                assert!(
                    fonts.has_glyphs(&FontId::new(12.0, FontFamily::Proportional), "▶‖↺"),
                    "↺ only via egui fallback"
                );
            });
        });
    }
}
