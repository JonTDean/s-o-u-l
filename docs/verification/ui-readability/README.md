# UI readability verification

Captured on the `feature/render_pipeline/threedim_gpu` branch with the built-in
verification modes (`SOUL_LENIA_VERIFY`, `SOUL_EXPERIMENT_VERIFY`, `SOUL_AUTHORING_VERIFY`,
`SOUL_RULES_VERIFY`) at 1280x720 (Lenia) and 1760x960 (experiment and rules) on
monitor 1. "Before" images come from commit `1f500c2`, built in a separate worktree.

| Image | What changed / what to look at |
| --- | --- |
| `inspector-lenia.png` (before: `inspector-lenia-before.png`) | Libertinus serif and italic labels are gone. Sections are divided by hairlines with small-caps titles and a fold chevron; model and seed are segmented controls (Quantized XY / Continuous, Blob / Ring / Orbium) with the selected option in amber; sliders have boxed mono values wide enough for `10.000`; the rail Compare icon (`≡`) renders. Grid is folded with its summary. |
| `inspector-lenia-folded.png` | Same capture with every section folded (the inspector does not scroll in verification, so the folds were forced for this image only): Grid and "About this specimen" show their one-line summaries, elided with an ellipsis rather than touching the title. |
| `inspector-authored.png` (before: `inspector-authored-before.png`) | Authored-rules form: bordered pills, framed code editor, Union/Intersection/XOR segmented control, grouped action buttons. The inspector sits at its 220 px minimum here. |
| `inspector-narrow.png` | Right 240 px of the authored-rules capture: the segmented control wraps (XOR drops to a second row), pills wrap, and nothing overlaps or clips. |
| `transport.png` (before: `transport-before.png`) | Keyed Run/Step/Reset buttons (Space, ., R), a captioned trace with the amber current value at its end, and the steps/s rates as one flush-right segmented group. |
| `library.png` (before: `library-before.png`) | Library drawer with divided, small-caps groups, a framed filter field, bordered Save and Import controls and the IBM Plex stack. The specimen subtitles `aⁿbⁿ` and `aⁿbⁿcⁿ` render their superscript `ⁿ` from the bundled SoulSymbols subset. |
| `legacy-main-menu.png`, `legacy-options.png` | Legacy menus render in IBM Plex with no missing-glyph boxes or clipped text in these captures. The main menu buttons no longer wrap and sit 30 percent above the bottom edge. |

## Not captured

- Dragging the live inspector to 220 px: synthetic pointer drags did not engage the panel
  resize handle. The authored-rules capture lands on the 220 px minimum by itself and is
  used instead (`inspector-narrow.png`).
- The Lenia inspector scrolled to the folded Grid and "About this specimen" sections: the
  verification modes do not scroll the panel, so `inspector-lenia-folded.png` forces every
  section closed instead.
- The two specimen warnings below the About header (edited-rule and continuous-model notes)
  are covered by a unit test; no verification mode edits a catalog specimen into that state.

## Glyph coverage

`≡` and `ⁿ` were missing from the first symbol subset and rendered as boxes; both are now in
`assets/fonts/symbols/SoulSymbols-Regular.ttf`, and `theme::fonts` tests check every specimen
title, subtitle, caption and explanation plus rail and transport glyphs against each font family.
