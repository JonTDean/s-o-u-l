# UI readability verification

Captured on the `feature/render_pipeline/threedim_gpu` branch with the built-in
verification modes (`SOUL_LENIA_VERIFY`, `SOUL_EXPERIMENT_VERIFY`, `SOUL_AUTHORING_VERIFY`,
`SOUL_RULES_VERIFY`) at 1280x720 (Lenia) and 1760x960 (experiment and rules) on
monitor 1. "Before" images come from commit `1f500c2`, built in a separate worktree.

| Image | What changed / what to look at |
| --- | --- |
| `inspector-lenia.png` (before: `inspector-lenia-before.png`) | Libertinus serif and italic labels are gone. Sections are divided by hairlines with small-caps titles and a fold chevron; model and seed are segmented controls with the selected option in amber; sliders have boxed mono values. |
| `inspector-authored.png` (before: `inspector-authored-before.png`) | Authored-rules form: bordered pills, framed code editor, Union/Intersection/XOR segmented control, grouped action buttons. The inspector sits at its 220 px minimum here. |
| `inspector-narrow.png` | Right 240 px of the authored-rules capture: the segmented control wraps (XOR drops to a second row), pills wrap, and nothing overlaps or clips. |
| `transport.png` (before: `transport-before.png`) | Keyed Run/Step/Reset buttons (Space, ., R), a captioned trace with the amber current value at its end, and the steps/s rates as one flush-right segmented group. |
| `library.png` (before: `library-before.png`) | Library drawer with divided, small-caps groups, bordered Save and Import controls and the IBM Plex stack. |
| `legacy-main-menu.png`, `legacy-options.png` | Legacy menus render in IBM Plex with no tofu or clipped text. The main menu buttons no longer wrap and sit 30 percent above the bottom edge. |

## Not captured

- Dragging the live inspector to 220 px: synthetic pointer drags did not engage the panel
  resize handle. The authored-rules capture lands on the 220 px minimum by itself and is
  used instead (`inspector-narrow.png`).
- The Lenia inspector scrolled to the folded Grid and "About this specimen" sections: the
  verification modes do not scroll the panel.
