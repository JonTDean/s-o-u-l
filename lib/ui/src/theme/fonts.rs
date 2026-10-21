//! IBM Plex Sans / Mono, embedded and registered as named egui families.
use bevy_egui::egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

macro_rules! font_in {
    ($dir:literal, $file:literal) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/fonts/",
            $dir,
            "/",
            $file
        ))
    };
}

macro_rules! font {
    ($file:literal) => {
        font_in!("plex", $file)
    };
}

/// Plex Sans Regular: body text, titles, captions.
pub const SANS: &str = "sans";
/// Plex Sans Medium: tab names.
pub const SANS_MEDIUM: &str = "sans-medium";
/// Plex Sans SemiBold: pill labels, caps labels, figure numbers.
pub const SANS_SEMIBOLD: &str = "sans-semibold";
/// Plex Mono Medium: readout values.
pub const MONO_MEDIUM: &str = "mono-medium";

const FACES: [(&str, &[u8]); 6] = [
    ("plex-sans", font!("IBMPlexSans-Regular.otf")),
    ("plex-sans-medium", font!("IBMPlexSans-Medium.otf")),
    ("plex-sans-semibold", font!("IBMPlexSans-SemiBold.otf")),
    ("plex-mono", font!("IBMPlexMono-Regular.otf")),
    ("plex-mono-medium", font!("IBMPlexMono-Medium.otf")),
    // Plex lacks ⁂ ‖ ▾ ▸ ␣ ≡ ⁿ; a seven-glyph DejaVu subset fills them.
    (
        "soul-symbols",
        font_in!("symbols", "SoulSymbols-Regular.ttf"),
    ),
];

/// Named egui family for `name`.
pub fn family(name: &str) -> FontFamily {
    FontFamily::Name(name.into())
}

/// Whether this pass can lay out the named Plex families.
///
/// `Context::set_fonts` takes effect at the start of the next pass, so UI that uses the
/// named families must wait for this rather than for the install system.
pub fn ready(ctx: &bevy_egui::egui::Context) -> bool {
    ctx.fonts(|fonts| fonts.families().contains(&family(SANS)))
}

/// egui's defaults with Plex first in every family; defaults remain as glyph fallbacks.
pub fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    for (key, bytes) in FACES {
        fonts
            .font_data
            .insert(key.into(), Arc::new(FontData::from_static(bytes)));
    }
    let mut fallback = fonts.families[&FontFamily::Proportional].clone();
    let mut mono_fallback = fonts.families[&FontFamily::Monospace].clone();
    fallback.push("soul-symbols".into());
    mono_fallback.push("soul-symbols".into());
    let chain = |first: &str, rest: &[String]| {
        std::iter::once(first.to_string())
            .chain(rest.iter().cloned())
            .collect::<Vec<_>>()
    };
    fonts
        .families
        .insert(FontFamily::Proportional, chain("plex-sans", &fallback));
    // Plex Mono lacks symbols such as ␣; Plex Sans is the last resort.
    let mut mono = chain("plex-mono", &mono_fallback);
    mono.push("plex-sans".into());
    fonts.families.insert(FontFamily::Monospace, mono);
    for (name, key) in [
        (SANS, "plex-sans"),
        (SANS_MEDIUM, "plex-sans-medium"),
        (SANS_SEMIBOLD, "plex-sans-semibold"),
    ] {
        fonts.families.insert(family(name), chain(key, &fallback));
    }
    let mut mono_medium = chain("plex-mono-medium", &mono_fallback);
    mono_medium.push("plex-sans".into());
    fonts.families.insert(family(MONO_MEDIUM), mono_medium);
    fonts
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_egui::egui::{Context, FontId, RawInput};

    #[test]
    fn every_family_leads_with_plex_and_keeps_fallbacks() {
        let defs = definitions();
        for name in [SANS, SANS_MEDIUM, SANS_SEMIBOLD, MONO_MEDIUM] {
            let chain = &defs.families[&family(name)];
            assert!(chain[0].starts_with("plex"), "{name}: {chain:?}");
            assert!(
                chain.len() > 1,
                "{name} keeps egui fallbacks for missing symbols"
            );
        }
        assert_eq!(defs.families[&FontFamily::Monospace][0], "plex-mono");
        for mono in [family(MONO_MEDIUM), FontFamily::Monospace] {
            let chain = &defs.families[&mono];
            assert_eq!(
                chain.last().map(String::as_str),
                Some("plex-sans"),
                "{mono:?}"
            );
        }
    }

    #[test]
    fn named_families_are_ready_only_from_the_pass_after_install() {
        let ctx = Context::default();
        let mut seen = Vec::new();
        for _ in 0..2 {
            let _ = ctx.run(RawInput::default(), |ctx| {
                if seen.is_empty() {
                    ctx.set_fonts(definitions());
                }
                seen.push(ready(ctx));
            });
        }
        assert_eq!(seen, [false, true], "set_fonts applies at the next pass");
    }

    #[test]
    fn fonts_load_and_cover_lab_glyphs() {
        let ctx = Context::default();
        ctx.set_fonts(definitions());
        let _ = ctx.run(RawInput::default(), |_| {});
        let _ = ctx.run(RawInput::default(), |ctx| {
            ctx.fonts(|fonts| {
                let sans = FontId::new(12.0, FontFamily::Proportional);
                let mono = FontId::new(12.0, FontFamily::Monospace);
                assert!(fonts.has_glyphs(&sans, "μσΔ⁂×"));
                assert!(fonts.has_glyphs(&sans, "▶‖↺▾▸"), "↺ only via egui fallback");
                assert!(fonts.has_glyphs(&mono, "0123456789"));
                assert!(
                    fonts.has_glyphs(&mono, "␣→"),
                    "Turing tables print the blank as ␣"
                );
                // "⏭" renders in Plex's fallback chain, so Task 6 uses it as the Step icon.
                assert!(fonts.has_glyphs(&sans, "⏭"));
                assert!(fonts.has_glyphs(&sans, "‖"), "Pause label");
            });
        });
    }

    #[test]
    fn every_authored_and_chrome_glyph_has_a_font() {
        let specimens = models::workbench::catalog::specimens::Specimens::embedded();
        let mut texts: Vec<String> = [
            "≡",
            "§",
            "⁂",
            "‖ Pause",
            "▶ Run",
            "⏭ Step",
            "↺ Reset",
            "▾ ▸",
        ]
        .map(String::from)
        .into();
        for s in &specimens.0 {
            texts.extend([s.title.clone(), s.subtitle.clone(), s.caption.clone()]);
            texts.push(s.explanation.clone());
        }
        let ctx = Context::default();
        ctx.set_fonts(definitions());
        let _ = ctx.run(RawInput::default(), |_| {});
        let _ = ctx.run(RawInput::default(), |ctx| {
            ctx.fonts(|fonts| {
                let families = [
                    FontFamily::Proportional,
                    FontFamily::Monospace,
                    family(SANS),
                    family(SANS_MEDIUM),
                    family(SANS_SEMIBOLD),
                    family(MONO_MEDIUM),
                ];
                for fam in families {
                    let id = FontId::new(12.0, fam.clone());
                    for text in &texts {
                        assert!(
                            fonts.has_glyphs(&id, text),
                            "{fam:?} lacks glyphs for {text:?}"
                        );
                    }
                }
            });
        });
    }
}
