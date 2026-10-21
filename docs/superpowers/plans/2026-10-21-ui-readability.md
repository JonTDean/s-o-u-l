# UI Readability Pass Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Libertinus serif look with IBM Plex. Rebuild the inspector as a
divided list of foldable sections, rebuild the transport bar to the approved mockup,
and give every form, drawer and menu clickable-looking controls with short hover
animations.

**Architecture:** Nearly all styling flows through `lib/ui/src/theme`: fonts, global
egui `Style`, tokens and the shared widgets. So the plan works in two stages:

1. Change the theme layer: fonts → style → controls → section widget.
2. Migrate call sites screen by screen: inspector, transport, then forms and drawers.

Behaviour, shortcuts and colours stay as they are.

**Tech Stack:** Rust 2024, Bevy 0.16, bevy_egui 0.35.1 / egui 0.32. The app runs
`cargo test` through `make check`.

**Spec:** `docs/superpowers/specs/2026-10-21-ui-readability-design.md` (read it
first; the approved mockups are described there).

## Global Constraints

- Palette in `lib/ui/src/theme/tokens.rs` is unchanged. The only new tokens are
  `LINE = #2a2e30` and `LINE_HI = #4a4f52`.
- Typefaces are IBM Plex Sans (Regular, Medium, SemiBold) and IBM Plex Mono (Regular,
  Medium). They are vendored under `assets/fonts/plex/` with `LICENSE.txt` (OFL) and
  embedded with `include_bytes!`. egui's default fonts stay as glyph fallbacks.
- Type scale: `LABEL` 10, `BODY` 13, `TITLE` 15, `READOUT` 24, `HUD` 40. Section
  titles are Plex Sans SemiBold, `LABEL` size, uppercase, about 1 px extra letter
  spacing, `INK_DIM`, or `SIGNAL` when the section is active.
- Motion is 120–200 ms (use 0.15 s) via `ctx.animate_bool_with_time`. Nothing loops
  or moves on its own.
- Every text token still meets WCAG AA on `BG` and `PANEL`. The existing test stays.
- Repo structure (`structural-governance.toml`), checked by `make governance`:
  - Each directory holds 1–4 source files and at most 3 child directories.
  - Each file has at most 200 SLOC.
- Commit at every 100 changed lines or sooner (AGENTS.md). Run `make check` at the end
  of every task.
- Previews run windowed on monitor 1.

## Review Focus

1. **Narrow inspector (220 px) with a five-option choice** (Life pattern). The control
   wraps into rows; no segment is clipped or overlaps. Pinned in Task 3
   (`segment_layout_wraps_at_minimum_width`).
2. **The rate is not a preset** (the inspector's steps/s slider set 12.5). No steps/s
   segment is highlighted, and clicking one sets it. Pinned in Task 6
   (`rate_choice_is_none_off_preset`).
3. **A citation without a parsable author or year.** The About summary falls back to
   `Fig. N`. Pinned in Task 5 (`about_summary_falls_back_to_figure`).
4. **An icon glyph missing from Plex and the fallbacks** would draw as tofu. Every
   icon used is asserted. Pinned in Task 1 (`fonts_load_and_cover_lab_glyphs`).
5. **Fold state across frames and channel switches.** A section folded by the user
   stays folded when another channel is selected and the panel redraws. Pinned in
   Task 4 (`fold_state_survives_frames`).

---

## File Structure

```
assets/fonts/plex/                     NEW: 5 Plex .otf files + LICENSE.txt; libertinus/ deleted at end of Task 1
lib/ui/src/theme/
  fonts.rs                             families: SANS, SANS_MEDIUM, SANS_SEMIBOLD, MONO_MEDIUM
  tokens.rs                            + LINE, LINE_HI; new sizes
  visuals.rs                           bordered widgets, radius 6, disabled alpha, hover expansion
  widgets/
    mod.rs                             re-exports + headless render test
    readout.rs                         text primitives: plain, dim, caps, mono, readout, footnote
    controls/                          NEW dir (replaces controls.rs)
      mod.rs                           pill, pill_with_key, lamp, lamp_colour, pill_fill
      segmented.rs                     segmented, segment_layout
      slider.rs                        slider, value_box
    section/                           NEW dir (replaces margin.rs)
      mod.rs                           Section builder, section(), note()
lib/ui/src/lab/inspector/{mod.rs,lenia.rs,sections.rs,parameters/**}
lib/ui/src/lab/chrome/{transport.rs,rail.rs,tabs.rs,drawers/**}
lib/ui/src/lab/stage/{overlay.rs,presenter.rs,editing/controls.rs}
```

---

### Task 1: Plex fonts and text primitives replace Libertinus

**Files:**
- Create: `assets/fonts/plex/` containing `IBMPlexSans-Regular.otf`,
  `IBMPlexSans-Medium.otf`, `IBMPlexSans-SemiBold.otf`, `IBMPlexMono-Regular.otf`,
  `IBMPlexMono-Medium.otf` and `LICENSE.txt`.
  - Sans source: `https://github.com/IBM/plex/raw/master/packages/plex-sans/fonts/complete/otf/<file>`
  - Mono source: same pattern under `plex-mono`.
  - License source: `https://github.com/IBM/plex/raw/master/LICENSE.txt`.
  - These URLs were verified working on 2026-10-21.
- Modify: `lib/ui/src/theme/fonts.rs`, `lib/ui/src/theme/tokens.rs` (`size` module),
  `lib/ui/src/theme/visuals.rs` (Heading family and its test),
  `lib/ui/src/theme/widgets/readout.rs` and `lib/ui/src/theme/widgets/mod.rs`.
- Modify every call site of `serif`, `serif_italic`, `SERIF*` and `SANS_BOLD`, listed
  by `grep -rnE 'serif|SERIF|SANS_BOLD' lib/ui/src src`. Today these are in
  `transport.rs`, `rail.rs`, `tabs.rs`, `overlay.rs`, `presenter.rs`, `fields.rs`,
  `lenia.rs`, `inspector/mod.rs`, `library/*`, `compare/mod.rs`, `project/mod.rs`,
  `exports.rs`, `margin.rs` and `controls.rs`.
- Delete: `assets/fonts/libertinus/`.

**Interfaces:**
- Produces in `theme::fonts`:
  - `pub const SANS: &str = "sans"` (Plex Sans Regular)
  - `SANS_MEDIUM`
  - `SANS_SEMIBOLD`
  - `MONO_MEDIUM`
  - `family(name) -> FontFamily` (unchanged)
  - `ready(ctx)`, which now checks `family(SANS)`.

  `FontFamily::Proportional` leads with Plex Sans Regular and `Monospace` with Plex
  Mono Regular. Plex Sans is the mono chain's last resort, as Libertinus Sans was.
- Produces in `theme::widgets`, replacing `serif` and `serif_italic`:
  - `plain(text) -> RichText`: Plex Sans, default ink.
  - `dim(text) -> RichText`: Plex Sans, `INK_DIM`.
  - `caps(text) -> RichText`: uppercase, `SANS_SEMIBOLD`, `size::LABEL`,
    `.extra_letter_spacing(1.0)`, `INK_DIM`.
  - `mono(text)`: unchanged.
  - `readout(ui, value, unit, big)`: the value uses `MONO_MEDIUM`; the unit uses
    `dim`.

  Mapping at call sites:
  - `serif` → `plain`
  - `serif_italic` → `dim`
  - `SERIF_SEMIBOLD` / `SANS_BOLD` → `SANS_SEMIBOLD`
  - `SERIF` used for tab titles → `SANS_MEDIUM`
  - `SERIF` / `SERIF_ITALIC` used elsewhere → `SANS`

- [ ] **Step 1: Update the font tests in `fonts.rs`.**
  - `every_family_leads_with_plex_and_keeps_fallbacks`: for `[SANS, SANS_MEDIUM,
    SANS_SEMIBOLD, MONO_MEDIUM]`, assert `chain[0].starts_with("plex")` and
    `chain.len() > 1`. Also assert the `Monospace` chain leads with `"plex-mono"`.
  - `fonts_load_and_cover_lab_glyphs`: assert `has_glyphs` for:
    - Proportional at 12 px: `"μσΔ⁂×"` and `"▶‖↺▾▸"`.
    - Monospace: `"0123456789"` and `"␣→"`.
    - The transport Step icon, chosen in Task 6: `"⏭"`, or the fallback `"»"` if
      `"⏭"` fails here. Record the result in the test.
  - `named_families_are_ready_only_from_the_pass_after_install`: unchanged.
- [ ] **Step 2: Run the tests and watch them fail.** Run `cargo test -p ui theme::fonts`.
  Expected: FAIL to compile (`SANS` is undefined).
- [ ] **Step 3: Vendor the fonts, then rewrite `FACES`, the constants and
  `definitions()` in `fonts.rs`.** Key names are `plex-sans`, `plex-sans-medium`,
  `plex-sans-semibold`, `plex-mono` and `plex-mono-medium`. Update the `size` module to
  `LABEL` 10.0 and `BODY` 13.0. Set the Heading text style to `family(SANS_SEMIBOLD)`
  and update `type_scale_uses_spec_sizes` to match.
- [ ] **Step 4: Rename the text primitives in `readout.rs` and migrate every call
  site.** Add `caps`. Delete `assets/fonts/libertinus/`.
- [ ] **Step 5: Run the checks.** Run `cargo test -p ui` and then
  `grep -rniE 'serif|libertinus' lib src assets`. Expected: the tests PASS, and grep
  finds nothing outside git history.
- [ ] **Step 6: Commit.** Commit at least once mid-way at the 100-line checkpoint. Use
  the final message `feat(ui): replace Libertinus with IBM Plex`.

---

### Task 2: Bordered global style and hairline tokens

**Files:**
- Modify: `lib/ui/src/theme/tokens.rs` and `lib/ui/src/theme/visuals.rs`.

**Interfaces:**
- Produces `tokens::LINE` (`#2a2e30`) and `tokens::LINE_HI` (`#4a4f52`).

- [ ] **Step 1: Replace `no_widget_or_window_draws_a_hairline` with
  `widgets_draw_hairlines_and_focus_in_signal`.** After `apply`:
  - `w.inactive.bg_stroke == Stroke::new(1.0, LINE)`
  - `w.hovered.bg_stroke == Stroke::new(1.0, LINE_HI)`
  - `w.active.bg_stroke.color == SIGNAL`
  - `w.inactive.corner_radius == CornerRadius::same(6)`
  - `w.hovered.expansion == 1.0`
  - `visuals.selection.stroke.color == SIGNAL`, which egui uses for the focused
    `TextEdit` border.
  - `visuals.extreme_bg_color == BG`, the text-input fill.
  - `visuals.window_stroke.width == 0.0`
  - `panel_fill == PANEL`
  - `override_text_color == Some(INK)`
- [ ] **Step 2: Run the test and watch it fail.** Run
  `cargo test -p ui theme::visuals`. Expected: FAIL.
- [ ] **Step 3: Implement it in `visuals.rs`.**
  - Set `noninteractive.bg_stroke` to `Stroke::NONE`, so labels and frames stay
    borderless.
  - Set `inactive`, `hovered` and `active` as asserted. Set `open` like `hovered`.
  - Set the corner radius to 6 on every state.
  - Set `hovered.expansion` to 1.0. This is egui's own hover growth for the slider
    knob and buttons.
  - If egui 0.32 has `Visuals::disabled_alpha`, set it to 0.4, which fades the
    disabled Step button.
  - Keep `window_stroke` and the shadows off.
- [ ] **Step 4: Run the tests and see them pass.** Run `cargo test -p ui`. Expected:
  PASS.
- [ ] **Step 5: Commit.** Message: `feat(ui): bordered widget style with hairline
  tokens`.

---

### Task 3: Controls (pill, segmented control, slider)

**Files:**
- Delete: `lib/ui/src/theme/widgets/controls.rs`.
- Create: `lib/ui/src/theme/widgets/controls/mod.rs`, which holds `pill`,
  `pill_with_key`, `pill_fill`, `lamp` and `lamp_colour`, re-exporting the other two
  files.
- Create: `lib/ui/src/theme/widgets/controls/segmented.rs` and
  `lib/ui/src/theme/widgets/controls/slider.rs`.
- Modify: `lib/ui/src/theme/widgets/mod.rs` (re-exports; replace `choice` with
  `segmented`).
- Modify the call sites of `choice`: `lenia.rs` (3), `fields.rs` (pattern; drop its
  `horizontal_wrapped`), `world.rs` and `stage/editing/controls.rs`. Also modify
  `authored/layers.rs`, where the blend `selectable_value` loop becomes `segmented`.

**Interfaces:**
- `pub fn pill(ui: &mut Ui, label: &str, primary: bool, enabled: bool) -> Response`
  keeps its signature.
  - The button is a 6 px-radius rect with a `LINE` border, or `LINE_HI` on hover.
  - Primary uses a `SIGNAL` fill with `BG` text in `SANS_SEMIBOLD`.
  - On hover, the fill and border animate over 0.15 s.
  - While the pointer is down, the content is offset 1 px downward.
  - A disabled button is drawn at 40 % opacity and does not respond to clicks.
- `pub fn pill_with_key(ui: &mut Ui, label: &str, key: &str, primary: bool, enabled:
  bool) -> Response` adds a trailing key chip: Plex Mono at `LABEL` size, a 1 px
  border, radius 3. The chip border is `LINE` on a normal button and `#c98a2c` on a
  primary one.
- `pub fn segment_layout(available: f32, widest: f32, count: usize) -> (usize, f32)`
  returns `(per_row, segment_width)`.
  - `per_row = ((available - 4.0 + 2.0) / (widest + 12.0 + 2.0)).floor().clamp(1, count)`
  - `segment_width = (available - 4.0 - 2.0 * (per_row - 1) as f32) / per_row as f32`
- `pub fn segmented<T: PartialEq + Copy>(ui: &mut Ui, current: &mut T, options: &[(T,
  &str)]) -> bool` takes the same arguments as the old `choice`.
  - The track is one `BG`-filled, `LINE`-bordered, radius-6 rect around all rows, with
    2 px padding and 2 px gaps.
  - The selected segment uses a `SIGNAL_BG` fill with a 1 px inset `#6a4f22` border
    and `SIGNAL` text.
  - An unselected segment is `INK_DIM`, animating to a `RAISED` fill and `INK` text on
    hover.
  - It returns true only when a different option was clicked.
- `pub fn slider(ui: &mut Ui, label: &str, value: &mut f32, range: RangeInclusive<f32>,
  log: bool) -> bool` keeps its signature.
  - The layout is a 72 px label column (`plain`, `INK`), the egui slider filling the
    rest, then a 56 px value box.
  - `pub fn value_box(ui: &mut Ui, text: &str)` draws the value box: a `BG` fill, a
    `LINE` border, radius 4, margin (5, 1), and right-aligned `mono` text.

- [ ] **Step 1: Write the tests in `segmented.rs`.**
  - `segment_layout_fits_three_at_default_width`:
    `segment_layout(286.0, 80.0, 3) == (3, (286.0 - 4.0 - 4.0) / 3.0)`.
  - `segment_layout_wraps_at_minimum_width`: with `available = 206.0` (a 220 px panel
    minus margins), `widest = 60.0` and `count = 5`, assert `per_row == 2`,
    `segment_width >= 60.0`, and `per_row * segment_width + gaps <= 206.0 - 4.0`.
  - `segment_layout_never_returns_zero_columns`:
    `segment_layout(10.0, 200.0, 4).0 == 1`.
  - `segmented_click_changes_only_on_new_value`: run a headless `Context`. Simulate a
    pointer click on segment 2 by feeding `RawInput` events at that segment's rect,
    which is captured from a first pass. Assert the function returns true and
    `current == 2`. Clicking the selected segment returns false.
- [ ] **Step 2: Run the tests and watch them fail.** Run
  `cargo test -p ui theme::widgets`. Expected: FAIL (undefined).
- [ ] **Step 3: Implement the `controls/` files.** Update the headless render test in
  `widgets/mod.rs` to call `segmented`, `pill_with_key` and `slider`. Migrate the
  `choice` and blend call sites. Keep the full labels; the spec allows shortening but
  doesn't require it.
- [ ] **Step 4: Run the checks and see them pass.** Run `cargo test -p ui` and
  `make governance`. Expected: PASS.
- [ ] **Step 5: Commit.** Message: `feat(ui): segmented control, bordered pills and
  boxed slider values`.

---

### Task 4: Section widget replaces margin sections

**Files:**
- Delete: `lib/ui/src/theme/widgets/margin.rs`.
- Create: `lib/ui/src/theme/widgets/section/mod.rs`, holding the `Section` builder,
  `section()` and `note()`, with `note` moved here unchanged except for using `dim`.
- Modify: `lib/ui/src/theme/widgets/mod.rs`.
- Modify all 26 `margin_section` call sites, found with
  `grep -rn margin_section lib/ui/src`.

**Interfaces:**

```rust
pub fn section(label: &str) -> Section<'_>;
impl<'a> Section<'a> {
    pub fn active(self, on: bool) -> Self;            // title in SIGNAL
    pub fn foldable(self, open_by_default: bool) -> Self;
    pub fn summary(self, text: impl Into<String>) -> Self; // shown on the header's right while folded
    pub fn action(self, label: &'a str) -> Self;      // dim dotted-underline link on the header's right
    pub fn show(self, ui: &mut Ui, body: impl FnOnce(&mut Ui)) -> SectionResponse;
}
pub struct SectionResponse { pub action_clicked: bool, pub open: bool }
```

Behaviour:
- **Divider.** A 1 px `LINE` hairline spans the full width above the section, except
  when it is the first thing in its `Ui`, i.e. `ui.min_rect().height() <= 0.0` at
  entry. Vertical padding is 11 px.
- **Header row.** The `caps(label)` title sits on the left. On the right is the
  chevron (`▾`, rotated −90° by fold openness) for foldable sections, the summary
  (`mono`, `INK_DIM`) while folded, or the action link.
- **Hover edge.** A 2 px `SIGNAL` bar sits at the left of the header row, 8 px into
  the margin. Its alpha is `animate_bool_with_time(id, pointer in last frame's section
  rect, 0.15)`. Store the section rect in `ui.data_mut` temp memory under the section
  id.
- **Folding.** Use `egui::collapsing_header::CollapsingState::load_with_default_open`.
  It persists in egui memory and animates the open and close. The id is
  `ui.make_persistent_id(("section", label))`. That is per label, not per channel:
  folding Grid stays folded across channels.
- Non-foldable sections always show their body. A label of `""` draws no header row.

- [ ] **Step 1: Write the tests in `section/mod.rs`.**
  - `first_section_draws_no_divider`: in a headless pass, show two sections in a fresh
    `CentralPanel`. Collect the `output.shapes` line segments with colour `LINE` and
    assert exactly one.
  - `fold_state_survives_frames`: show `section("grid").foldable(false)` over three
    passes. In pass 2, toggle its `CollapsingState` to open and store it, as a click
    would. Assert `SectionResponse.open` is true in pass 3. Then show a different
    section in between, simulating a channel switch, and assert "grid" is still open.
  - `summary_only_while_folded`: assert a folded section with `.summary("64×64×8 ·
    GPU")` paints that text (search the shapes' galleys for the string), and that an
    open one doesn't.
  - `action_reports_click`: click the action link's rect, captured from a first pass;
    assert `action_clicked`.
- [ ] **Step 2: Run the tests and watch them fail.** Run
  `cargo test -p ui theme::widgets::section`. Expected: FAIL.
- [ ] **Step 3: Implement `Section` and migrate every call site.** Each call site maps
  as follows:
  - `margin_section(ui, l, a, f)` becomes `section(l).active(a).show(ui, f)`.
  - Inspector sections other than `readout` add `.foldable(true)`. Task 5 sets the
    folded defaults and summaries.
  - Drawer sections (library, compare, project, exports, author) stay non-foldable.
  - Delete `MARGIN_WIDTH`, and remove the inspector panel's asymmetric
    `Margin { left: 2, .. }` in favour of `Margin::symmetric(14, 14)`.
- [ ] **Step 4: Run the checks and see them pass.** Run `cargo test -p ui` and
  `make governance`. Expected: PASS.
- [ ] **Step 5: Commit.** Commit at the 100-line checkpoints. Use the final message
  `feat(ui): divided, foldable sections replace margin sections`.

---

### Task 5: Inspector content (folded Grid and About, warnings, header actions)

**Files:**
- Modify: `lib/ui/src/lab/inspector/mod.rs`, `lib/ui/src/lab/inspector/lenia.rs` and
  `lib/ui/src/lab/inspector/sections.rs`.

**Interfaces:**
- Consumes: `section`, `Section::{foldable, summary, action, active}` and
  `SectionResponse` from Task 4. Also consumes `footnote` and `note`.
- Produces `sections::about_summary(figure: u32, citation: &str) -> String`. It returns
  `"Fig. {figure} · {Surname} {year}"`:
  - `Surname` is the last whitespace-separated word before the citation's first `,`.
  - `year` is the last `(dddd)` group.
  - If either can't be found, it returns `"Fig. {figure}"`.

- [ ] **Step 1: Write the tests in `sections.rs`.**
  - `about_summary_names_first_author_and_year`: the Chan citation from
    `assets/specimens/specimens.toml` at figure 5 gives `"Fig. 5 · Chan 2019"`. The
    Hardy, Pomeau and de Pazzis citation at figure 4 gives `"Fig. 4 · Hardy 1973"`.
  - `about_summary_falls_back_to_figure`: `about_summary(3, "Unpublished notes")`
    gives `"Fig. 3"`.
- [ ] **Step 2: Run the tests and watch them fail.** Run
  `cargo test -p ui lab::inspector::sections`. Expected: FAIL.
- [ ] **Step 3: Implement `about_summary` and restructure `inspector/mod.rs`.**
  - `readout`: `.active(true)`, not foldable.
  - Both `grid` sections (CPU and GPU): `.foldable(false)` with
    `.summary(<the arena label they already print>)`.
  - Specimen block: the two amber warning notes ("Catalog specimen described below…"
    and "Continuous XYZ model…") are drawn as plain `note(.., true)` calls before the
    section. The Orbium explanation and the footnote go inside
    `section("About this specimen").foldable(false).summary(about_summary(figure,
    &specimen.citation))`.
  - In `lenia.rs`, `growth` uses `.action("Restore defaults")` instead of the inline
    link; `action_clicked` resets `LeniaParams::default()`.
  - In `colour`, each swatch's radius animates from 5 to 6 px on hover over 0.15 s.
    The selection ring is unchanged.
- [ ] **Step 4: Run the tests and see them pass.** Run `cargo test -p ui`. Expected:
  PASS.
- [ ] **Step 5: Commit.** Message: `feat(ui): fold secondary inspector sections behind
  summaries`.

---

### Task 6: Transport bar to the approved mockup

**Files:**
- Modify: `lib/ui/src/lab/chrome/transport.rs`.

**Interfaces:**
- Consumes `pill_with_key`, `segmented` and `caps` (Tasks 1 and 3), and
  `sections::format_metric` (existing).
- Produces `pub fn rate_choice(rate_hz: f32) -> Option<f32>`, which returns the
  preset in `RATE_PRESETS` that `rate_selected` matches.

Layout, left to right, at `exact_height(44.0)` with a 1 px `LINE` top border:
1. `pill_with_key("▶ Run" | "‖ Pause", "Space", primary = true, runnable)`.
2. `pill_with_key("<step icon from Task 1> Step", ".", false, paused)`.
3. `pill_with_key("↺ Reset", "R", false, row.is_some())`.
4. A 1 px × 22 px `LINE` divider.
5. The trace: `caps(metric.id.label)` on its left, then the sparkline (unchanged,
   1.4 px `SIGNAL`) filling the space, then the latest value via `format_metric` in
   `mono` at `BODY` size on its right.
6. A divider, then `caps("steps/s")`, then `segmented` over `RATE_PRESETS` with labels
   `format!("{preset:.0}")`. The current value is `rate_choice(rate)`. If it is `None`,
   pass a sentinel that matches no option, so nothing is highlighted. A changed value
   sends `SetRate`.

- [ ] **Step 1: Write the tests.**
  - `rate_choice_is_none_off_preset`: `rate_choice(12.5) == None`,
    `rate_choice(10.2) == Some(10.0)` and `rate_choice(60.0) == Some(60.0)`.
  - Keep the existing sparkline and `rate_presets_mark_the_nearest_rate` tests.
- [ ] **Step 2: Run the tests and watch them fail.** Run
  `cargo test -p ui lab::chrome::transport`. Expected: FAIL.
- [ ] **Step 3: Implement the layout above.**
- [ ] **Step 4: Run the tests and see them pass.** Run `cargo test -p ui`. Expected:
  PASS.
- [ ] **Step 5: Commit.** Message: `feat(ui): transport bar with keyed buttons, labelled
  trace and rate selector`.

---

### Task 7: Forms, drawers and menus

**Files:**
- Modify: `lib/ui/src/lab/inspector/parameters/authored/{mod.rs,layers.rs,transfer.rs}`.
- Modify: `lib/ui/src/lab/inspector/parameters/{mod.rs,energy.rs}`.
- Modify: `lib/ui/src/lab/chrome/drawers/{exports.rs,project/mod.rs,library/mod.rs,library/guides.rs}`.
- Modify: `lib/ui/src/lab/chrome/rail.rs`.

**Interfaces:**
- Consumes `pill`, `segmented`, `note`, `dim` and `mono`.
- Produces `pub fn code_frame() -> egui::Frame` in `theme/widgets/readout.rs`: a `BG`
  fill, a 1 px `LINE` stroke, radius 6 and a 6 px inner margin.

Rules applied:
- Every `ui.button` / `egui::Button` in these files becomes `pill(.., false, enabled)`,
  except that the form's commit action ("Validate and prepare rule") is
  `primary = true`.
- Related buttons share one `horizontal_wrapped` row:
  - Load current draft · Copy-west example · Use birth/survival masks
  - Add copy-west layer · Remove second layer
  - Import rule · Export new file
- The authored form's three explanation lines become one `dim` paragraph.
- The JSON editor sits in `code_frame()`, with `desired_width(ui.available_width())`.
  This replaces `.min(240.0)`.
- `editor.status` becomes `note(.., is_error)`, where `is_error` is true when it isn't
  one of the success strings. "Prepared: …" uses `mono`.
- The transfer path field, Import and Export share one row; the field fills the
  remaining width.
- Drawer text fields keep their hints, now using `dim` from Task 1. The global style
  from Task 2 gives them their border and focus colour.
- The legacy menus (`components/menus/meta/**`) take no code change unless the visual
  check in Task 8 shows a problem.

- [ ] **Step 1: Write the test.** In `authored/mod.rs` tests, or create a test module
  there if none exists, add `authored_form_renders_headless`: one pass drawing
  `draw(ui, &mut Settings::for_kind(Kind::Life), &mut Editor::default())` inside a
  `CentralPanel` must not panic and must produce shapes.
- [ ] **Step 2: Run the test.** Run `cargo test -p ui authored`. Expected: it passes
  already, because it is a regression guard for the restyle. Confirm it runs.
- [ ] **Step 3: Apply the rules above.**
- [ ] **Step 4: Run the checks.** Run `cargo test -p ui` and `make governance`.
  Expected: PASS.
- [ ] **Step 5: Commit.** Message: `feat(ui): grouped form actions and framed rule
  editor`.

---

### Task 8: Visual verification and evidence

**Files:**
- Create: `docs/verification/ui-readability/` containing `inspector-lenia.png`,
  `inspector-authored.png`, `transport.png`, `library.png` and `README.md`.

- [ ] **Step 1: Before images.** From a checkout of the commit before Task 1
  (`git worktree add <scratch> 1f500c2`), run `make lenia` and `make automata`. Keep
  the window on monitor 1. Capture the inspector, transport and authored-rules form as
  `*-before.png`.
- [ ] **Step 2: After images.** Capture the same views on this branch. Also open the
  legacy main menu and options menu, and confirm no clipped text or missing glyphs.
  Fix anything found, either in its owning task's file or with a new commit.
- [ ] **Step 3: Check at minimum width.** Drag the inspector to 220 px. Confirm the
  segmented controls wrap and nothing overlaps.
- [ ] **Step 4: Write the README.** In `README.md`, list each image with one line on
  what changed and what to look at.
- [ ] **Step 5: Run `make check`.** Expected: all tests pass and there are 0
  governance violations.
- [ ] **Step 6: Commit.** Message: `docs: verify UI readability pass`.
