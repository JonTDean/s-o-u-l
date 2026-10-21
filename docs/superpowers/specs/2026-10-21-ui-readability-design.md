# UI readability pass: type, inspector, transport, forms

Date: 2026-10-21 · Status: approved 2026-10-21

## Intent

The lab is hard to read in long sessions. The Libertinus Serif faces (upright and
italic) strain the eyes and look bulky at small sizes. The right-hand inspector packs a
lot in at once with no visible boundaries between sections. The bottom transport bar's
buttons, type and steps/s rate choices are bulky and look static. The rates in
particular read as plain text rather than as something you can click.

The goal is a calmer, more legible instrument: one screen-oriented sans face, clear
edges between sections, controls that look clickable, and short micro-animations that
help the eye find what changed or what is under the pointer.

## What stays

- The left rail and the top tab bar keep their layout and behaviour. They only take
  the new typeface.
- The transport trace line stays as it is.
- The colour palette (`lib/ui/src/theme/tokens.rs`) stays exactly as it is. The one
  addition is a hairline colour, described under Visual language.
- Keyboard shortcuts and all behaviour stay the same. This is a presentation change
  only.

## Decisions made with the user

1. **Typeface: IBM Plex Sans for text, IBM Plex Mono for numbers and code.** These
   replace Libertinus everywhere: serif, serif italic, sans and mono. Section titles
   become small tracked capitals (Plex Sans SemiBold, about 10 px, letter-spacing about
   0.09 em, `INK_DIM`) instead of italic serif.
2. **Inspector layout: a "divided list".** Each section is flat:
   - The title sits above the content instead of in the 56 px left margin column, so
     content gets the full width.
   - A 1 px hairline separates consecutive sections.
   - Hovering anywhere in a section shows a 2 px amber (`SIGNAL`) edge at the left of
     its title row.
   - A section's header row can carry one quiet action on the right, e.g. "Restore
     defaults" as a dim dotted-underline link that turns amber on hover.

   The user chose this over boxed cards.
3. **Less on screen at once.** Secondary sections start folded, with a chevron that
   rotates on fold. While folded, the header row shows a one-line summary on its right:
   - Grid, e.g. `64×64×8 · GPU`.
   - "About this specimen", which holds the footnote, citation and DOI.

   Fold state is per section and kept for the session.
4. **Single choices become segmented controls.** This covers model, seed, union /
   intersection / XOR, and every other use of `choice`. The control is:
   - A `BG`-filled track with a hairline border and 2 px padding.
   - Equal-width segments.
   - The selected segment drawn in `SIGNAL_BG` with an inset amber-brown border and
     `SIGNAL` text.
   - Hover lifts an unselected segment to `RAISED` / `INK`.
5. **Sliders.** These keep the thin track with an amber fill and a round knob, and gain:
   - A fixed label column (Greek symbols such as μ, σ, Δt in Plex Sans).
   - The value in a small bordered Plex Mono box on the right, in `SIGNAL`.
   - A knob that scales up slightly on hover.
6. **Transport bar: exactly the approved mockup.** The approved mockup is saved under
   `.superpowers/brainstorm/` as `transport.html`; that directory is gitignored.
   - Buttons become 6 px-radius rects with a hairline border, an icon and a shortcut
     key chip: `▶ Run  Space` / `‖ Pause  Space`, `⏭ Step  .` and `↺ Reset  R`.
   - Run / Pause is the amber primary button.
   - Hover brightens the border and fill, and a press nudges the button down by 1 px.
   - Disabled buttons, e.g. Step while the run isn't paused, fade to about 40 %
     opacity.
   - A vertical hairline divides the buttons, the trace and the rates.
   - The trace gets a `MASS`-style small-caps label (the metric's label) on its left
     and the current value in Plex Mono `SIGNAL` on its right.
   - The steps/s rates become the same segmented control as in the inspector.
7. **The treatment extends to the forms, drawers and menus.** This covers the
   authored-rules form, the library, compare, project and exports drawers, the rail
   popup, the options menu and the legacy main, pause and scenario menus.
   - Related action buttons sit together in wrapped rows instead of a tall stack of
     separate buttons. For example, "Load current draft · Copy-west example · Use
     birth/survival masks" form one row, and "Import rule · Export new file" another.
   - Text inputs, including the multiline JSON rule editor, get a `BG` fill, a
     hairline border, an amber border on focus, and Plex Mono text where the content
     is code or numbers.
   - Plain `ui.button` / `egui::Button` calls pick up the new button look through the
     global style, so the same treatment applies without per-site restyling where
     possible.

## Visual language (builder may tune within these)

- **Hairline:** add one token, e.g. `LINE ≈ #2a2e30`, for section dividers and
  control borders. Add a slightly brighter hover border, e.g. `#4a4f52`. These are
  the only new colours.
- **Type scale:** body text goes from 11 px to about 13 px, labels from 9 px to about
  10 px, and titles stay about 15 px. Plex has a larger x-height; tune sizes by eye so
  the inspector at its default width still fits a three-segment control without
  clipping. Readout sizes (24 px and 40 px) stay.
- **Motion:** about 120–200 ms with ease-out, via egui's `animate_bool` /
  `animate_value_with_time`. Animate hover fills and borders, the section hover edge,
  the chevron rotation, the fold open and close, and the slider knob scale. Nothing
  should loop or move on its own; nothing should compete with the simulation.
- **Contrast:** every text token must still meet WCAG AA on `BG` and `PANEL`. The
  existing `text_tokens_meet_wcag_aa_on_both_surfaces` test stays.

## Constraints

- **Fonts:** vendor the IBM Plex files under `assets/fonts/plex/` with their OFL
  license. Needed faces are Sans Regular, Medium and SemiBold, and Mono Regular and
  Medium (add an italic only if something still needs one). Keep them embedded via
  `include_bytes!` like the current faces. Keep egui's default fonts as glyph
  fallbacks: the existing glyph test covers `μσΔ⁂×`, `␣→` and `▶‖↺`, so extend it to
  cover the new icons (`⏭`, chevrons) or pick icons that render. Libertinus may be
  removed once nothing references it.
- **The current style test must change.** `no_widget_or_window_draws_a_hairline` in
  `theme/visuals.rs` asserts the old borderless look and must be replaced by tests for
  the new rules. The same goes for any test asserting `SERIF` families.
- **Repo structure** (`structural-governance.toml`):
  - Directory depth is at most 3.
  - Each directory holds 1–4 files.
  - Each file has at most 200 SLOC.
  - New widgets (section, segmented control, button, text field) go in
    `lib/ui/src/theme/widgets/`, split into subdirectories if it would exceed four
    files.
- **Commits:** at every 100 changed lines or sooner (see AGENTS.md); run `make check`
  on completed work items.
- **Previews:** keep any running preview windowed on monitor 1.

## Where the code is

- Theme entry points: `lib/ui/src/theme/fonts.rs` (families), `visuals.rs` (global
  egui style) and `tokens.rs` (palette and sizes).
- Shared widgets: `lib/ui/src/theme/widgets/margin.rs` (`margin_section`, `note`),
  `controls.rs` (`pill`, `slider`, `choice`, `lamp`) and `readout.rs` (`serif`,
  `serif_italic`, `mono`, `readout`, `footnote`). Most screens go through these:
  `margin_section` has about 26 call sites, `note` about 34, `pill` about 24, so
  restyling them carries most of the change.
- Inspector: `lib/ui/src/lab/inspector/` (`mod.rs`, `lenia.rs`, `machine.rs`,
  `parameters/`, including `parameters/authored/` for the rules form).
- Transport: `lib/ui/src/lab/chrome/transport.rs`. Tabs and rail: `chrome/tabs.rs` and
  `chrome/rail.rs`. Drawers: `chrome/drawers/`.
- Legacy menus: `lib/ui/src/components/menus/meta/`.

## Calls made by Claude (not discussed with the user)

These settle questions the builder check raised. The user can overrule any of them.

- **Warnings stay visible.** The amber notes in the specimen block ("Catalog specimen
  described below; this channel runs …" and "Continuous XYZ model: catalog text below
  …") move out of the fold. They sit just above the folded "About this specimen"
  header, so a mismatch between the catalog text and the running model is never
  hidden.
- **About's summary.** The folded "About this specimen" header shows the short source,
  e.g. `Fig. 5 · Chan 2019`, built from the figure number and the citation's first
  author and year. If that can't be derived, it shows the figure number alone.
- **Which sections fold.**
  - Every inspector section can fold.
  - Grid (both the CPU and GPU versions) and About start folded; all others start
    open.
  - Readout cannot fold, so the live numbers are always visible.
  - Drawer sections (library, compare, project, exports, author) get the new header
    style but no chevron, because they're already shown on demand.
- **The `active` flag stays.** An active section (Readout, Compare, Project) keeps its
  small-caps title in `SIGNAL` instead of `INK_DIM`, as today. The amber hover edge is
  separate and applies to every section.
- **Too many options for one row.** When a segmented control's options don't fit, it
  wraps onto further rows. Segments stay equal width within a row, so it reads as a
  grid of choices. Labels may be shortened where the meaning stays clear, e.g.
  "continuous density" → "Continuous". The full label goes in a hover tooltip. At the
  220 px minimum width, wrapping is expected and is not a failure.

## Left to the builder

- The exact API of the new section widget, e.g. replacing `margin_section(label,
  active, body)` with a `section(label).folded_by_default().summary(..).action(..)`
  builder or similar, and whether `margin_section` survives as a thin alias during
  migration.
- How the fold state is stored, using egui memory keyed by an id per section and
  channel is fine.
- The exact icon glyphs, as long as they render in the bundled fonts.
- Fine pixel values within the ranges above.

## Done means

- No Libertinus face is rendered anywhere in the lab or menus.
- The inspector shows divided sections with titles on top, hover edges, segmented
  choices, boxed slider values, and folded Grid and About sections with summaries.
- The transport matches the approved mockup.
- The forms, drawers and menus use grouped button rows and bordered inputs.
- `make check` passes, with tests updated for the new style rules.
- Before/after screenshots of the inspector, the transport and the authored-rules form
  are captured under `docs/verification/` for review.
