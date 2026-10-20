# Lab shell and visual language — design

Status: draft for review · 2026-10-19 · Sub-projects 1 + 2 of the lab redesign

## 1. Intent

Redesign S.O.U.L.'s UI so it looks and works like a **workbench/lab for automata**.

Priority of use, as stated by the user: **demonstrating** first, then
**experimenting**, then **exploring**, then **authoring**. Demonstration is a mix
of *presenter-driven* (someone drives the app on a projector or screen share) and
*self-guided* (a visitor uses it alone). The design therefore favours a dominant,
legible specimen view, readable typography and a strong explanatory layer, while
leaving clean extension points for experiment tooling.

### Success criteria

- The specimen viewer occupies the majority of the window (≈ 83 % of width at
  1080p), in the manner of Blender or Figma.
- CPU families and GPU Lenia are listed, selected and controlled through one
  interface; the separate Lenia window and CPU workbench window are gone.
- Every specimen shows a caption, a one-line explanation and a citation with DOI.
- Presenter mode turns any channel into a clean full-stage view driven by keys.
- Existing qualification (`verify-automata`, `verify-lenia`) keeps passing, and
  new tests cover the channel contract and metrics pipeline.

### Assumptions (not stated by the user, accepted during review)

- Family coverage and verified CPU/GPU behaviour are preserved; simulation code
  (`Session::step`, Lenia compute) is not rewritten.
- The UI stays on Bevy + egui, inside `structural-governance.toml` limits, with
  application windows on monitor 1.

## 2. Roadmap context

| Order | Sub-project | Scope |
| --- | --- | --- |
| **1 + 2 (this spec)** | Lab shell + visual language | Layout, unified channels, theme, fonts, stage, specimen metadata, metrics foundation |
| 3 | Guided layer | Specimen cards, on-stage annotations, tours (rail ¶ entry) |
| 4 + 5 | Experiment tooling; menu/scenario flow | Probes, comparison, snapshots, recording, export; main menu and scenarios reshaped around the lab |
| 6 | Authoring | Designing rules, seeds and initial conditions |

Each later sub-project receives its own spec → plan → implementation cycle.

## 3. Visual decisions

Approved mockups are kept in
[`mockups/2026-10-19-lab-shell/`](mockups/2026-10-19-lab-shell/) (fragments
written for the brainstorming companion; they render without its frame CSS).
`workbench-proportions.html` is authoritative for proportions; the family
variants in `family-variants.html` predate the narrower inspector.

| Decision | Choice |
| --- | --- |
| Direction | **Instrument** — graphite surfaces, amber signal accent |
| Layout | **L2 rail + drawer** — icon rail, library drawer, channel tabs, stage, inspector, transport |
| Surface style | **S4 Marginalia** — Tufte-style italic margin labels, superscript notes, whitespace grouping, **no hairline borders** |
| Type | **Libertinus** Serif / Sans / Mono (SIL OFL) |
| Proportions | Rail 32 px · inspector ≈ 260 px, drag 220–420 px, collapsible · stage the remainder |
| Presenter mode | Approved as mocked |
| Colour | Amber default for every family; per-channel colour selectable |

### 3.1 Tokens

| Token | Value | Use |
| --- | --- | --- |
| `bg` | `#0d0f10` | Window and stage background |
| `panel` | `#121415` | Rail, tabs, inspector, transport |
| `raised` | `#191c1d` | Pills, inactive controls |
| `track` | `#232728` | Slider tracks, grips |
| `ink` | `#c9c4b4` | Body text |
| `ink_hi` | `#ece6d4` | Titles, active tab, slider knobs |
| `ink_dim` | `#8a8476` | Margin labels, captions, notes (lightened from the mocked `#7b766a`, which measured 4.08:1 on `panel`, to meet 4.5:1) |
| `signal` | `#ffb547` | Values, selection, active state, default specimen colour |
| `signal_bg` | `#2a2416` | Selected rows |
| `ok` | `#7bd88f` | Running lamp |
| `fault` | `#ff6b5b` | Halted lamp, errors |
| `info` | `#6fb7ff` | GPU tag |

Specimen swatches: `signal`, `ink_hi`, `ok`, `info`, `fault`, plus a custom picker.

Type scale (logical px): label 9, body 11, title 15, readout 24, presenter HUD 40+.
Serif for titles, tab names, margin labels and captions; sans for body; mono
for every number, rule string and identifier.

## 4. Architecture

### 4.1 Channel contract — `engine_core::automata::channel`

The single interface between the UI (and qualification) and both backends.

- `ChannelId(u32)` — monotonically allocated, never reused within a run, and
  **internal**: the UI shows specimen names only.
- `ChannelCommand { id, action }` with `ChannelAction`:
  `Pause(bool)`, `Step`, `Reset`, `Remove`, `SetRate(Rate)`, `SetColour(Srgb)`,
  `SetInput(String)`, `Lenia(LeniaAction)`.
- `Rate` — one of 1, 10, 30, 60 steps/s; default 10 (today's fixed rate).
- `Channels` resource — ordered rows, `selected: Option<ChannelId>`, `capacity = 8`.
  Each row: id, kind id, backend (CPU/GPU), status (Running/Paused/Halted), tick,
  settings (rate, colour), headline metric and its availability.
- `AutomataCommand::{SeedPattern, Clear}` remain the spawn/clear entry points.
- Commands addressing an unknown or removed id are ignored with a debug log.

### 4.2 Metrics foundation — `engine_core::automata::channel::metrics`

Built fully now so sub-project 4 only adds reductions and consumers.

- `MetricId` — stable name, unit and display label.
- `MetricSample { tick, epoch, value: f64 }`.
- `Trace` — 200-sample ring buffer per channel and metric, with running min/max;
  cleared on Reset/Remove; samples from a stale epoch are rejected.
- `MetricAvailability` — `Live`, `Sampled { period }`, `OptIn`, `Unavailable(reason)`.
- `MetricSource` shape — *describe* then *sample when ready*.
  - **CPU sources:** pure functions over model state, run after each tick.
  - **GPU adapter:** wraps the existing `LeniaProbe` mailbox. It issues at most
    one `request(owner)` per sampling period, polls `take(owner)`, drops stale
    epochs, reduces on the CPU and publishes a sample. Readback errors become
    `Unavailable(reason)`.
- **Lenia mass** (`Σ density / 255`) ships registered as **`OptIn`, off by
  default**, preserving the invariant that ordinary rendering performs no
  readbacks. The inspector offers "sample mass (GPU readback)"; when enabled it
  samples at a throttled 4 Hz.

Headline metrics:

| Family | Metric |
| --- | --- |
| Rule 30, Rule 110 | Live cells in newest row |
| Life | Population |
| HPP | Particle count (conserved) |
| NFA, pushdown, parentheses, LBA, Turing | Step count; state and accept/halt alongside |
| Koch | String length |
| Boids | Polarisation (order parameter) |
| Ants | Food collected |
| Lenia | Tick; mass trace opt-in |

### 4.3 Backends

**CPU — `models::workbench`.** Sessions gain a `ChannelId` and a per-session clock
accumulator driven by its `Rate` (frame delta still capped at 0.5 s).
`Session::step` is unchanged. Systems: apply `ChannelCommand`s → tick → publish
rows, metric samples and traces. The `agents`, `grid` and `symbolic` directories
move under `workbench/families/` to free file slots for `channels.rs` and
`metrics.rs` under the four-file limit.

**GPU — `engine-render` command executor.** The Lenia owner entity gains a
`ChannelId` component. Commands map to existing `LeniaAction`s; `SetRate` scales
the Lenia fixed clock; `SetColour` writes a colour component extracted into the
voxel shading pass as a ramp uniform.

**Ordering.** Explicit system sets per frame: input/qualification commands →
CPU and GPU routers → CPU ticks and Lenia tick requests → publish `Channels` and
traces → UI draw. The UI is never a frame behind lifecycle state.

**UI boundary.** `lib/ui` reads `Channels`, traces, `Catalog` and `Specimens`, and
writes only `ChannelCommand` and `AutomataCommand`. It no longer mutates
`Workbench` or `LeniaSimulation`.

### 4.4 Theme — `lib/ui/src/theme/` (replaces `styles/`, keeps fade)

- `tokens.rs` — §3.1 constants and swatches.
- `fonts.rs` — Libertinus OTFs under `assets/fonts/libertinus/` with `OFL.txt`,
  embedded via `include_bytes!`, registered as named egui families.
- `visuals.rs` — installs egui `Style`/`Visuals` once: zero widget strokes, small
  rounding, the type scale, amber selection.
- S4 helpers — `margin_section`, `readout`, `footnote(n, text, doi)`,
  `pill_button`, `lamp`, themed `slider` and `choice`.

The main menu and options screens inherit the theme; their redesign belongs to
sub-project 5.

### 4.5 Shell — `lib/ui/src/lab/` (replaces `components/menus/automata` and `presentation/`)

Panels, in egui declaration order:

1. **Rail** — fixed 32 px left panel: ❦ Library, ⁂ Settings (presenter toggle,
   debug toggles). Guided (¶) arrives in sub-project 3; no placeholders now.
   If Libertinus lacks either glyph, the icons are drawn as small vector shapes.
2. **Tabs** — ≈ 28 px top bar: serif specimen names, status lamp, ×, and + to
   open the library.
3. **Transport** — ≈ 40 px bottom bar: Pause/Run, Step (disabled while
   running), Reset, metric trace sparkline, steps/s selector.
4. **Inspector** — right panel, default 260 px, drag 220–420 px, collapse with N.
   Margin sections: readout, rule/parameters, seed/input, colour, grid/arena,
   footnote with citation and DOI.
5. **Stage** — central panel with a transparent frame (§4.6).
6. **Library drawer** — egui `Area` beside the rail, dims the stage. Grouped by
   kind (cellular, machines, grammars, agents) with subtitles, a GPU tag where
   relevant, a filter field, and a footer with the channel count and Clear all.
   Picking a specimen opens a channel and closes the drawer.
7. **Presenter** — `LabMode::Presenter` skips all panels and draws large mono
   HUD readouts top-right, the serif title and caption bottom-left, and a
   transport that fades after 2 s without pointer motion.

Keyboard: Space run/pause · → step · R reset · 1–8 select tab · P presenter ·
N inspector · L library · Esc close drawer or exit presenter. Shortcuts are
suppressed while a text field has focus; conflicts with camera bindings are
resolved in the camera's favour.

Removed: "Active automata" list, Spawner window, Lenia window, CPU workbench
window, bottom-left preview caption text, pause banner, zoom/pos text, minimap,
and the debug "mark centre" button (its toggle moves into Settings).

### 4.6 Stage

Only the **selected** channel is shown; all channels keep ticking. The shell
publishes the stage rectangle each frame as `StageRect`.

**GPU (Lenia).** The world camera's `viewport` follows `StageRect` in physical
pixels, including resizes and inspector drags. Selecting a Lenia channel frames
the camera on its instance. When a CPU channel is selected the camera is
deactivated (simulation compute continues). Camera input is accepted only when
the pointer is over the stage and egui is not capturing it.

**CPU.** egui painters fit the specimen to the stage with wheel zoom and drag
pan; the overlay cursor reports cell coordinates. Painters, restyled from the
existing `canvas`, `symbols` and `koch` code and split by model type:

- Grid — 1-D space-time history, 2-D Life, HPP density ramp; faint dots mark empty cells.
- Symbolic — large tape/stack cells, head and state marked; transition table in the inspector.
- Grammar — Koch turtle path.
- Agents — boids as heading triangles; ants with food and a pheromone ramp.

**Colour.** Binary families draw with the channel colour. Continuous fields use a
four-stop ramp derived in Oklab (near-bg → dim → colour → light tint). Lenia
receives the same ramp as a uniform in voxel shading.

**Chrome.** Top-right mono overlay: size, boundary, backend, zoom, cursor.
Bottom-left serif caption: "Fig. n." plus the specimen caption. Both are
replaced by the HUD in presenter mode. With no channels, the stage reads
*"Open a specimen from the library (L)"* and the library opens on first launch.

### 4.7 Specimen metadata — `assets/specimens/specimens.toml`

Embedded with `include_str!`, parsed once into a `Specimens` resource (adds the
`toml` crate). One entry per catalog id, including `lenia`:

```toml
[[specimen]]
id          = "life:conway"
title       = "Conway's Game of Life"
group       = "cellular"            # cellular | machines | grammars | agents
subtitle    = "B3/S23"
caption     = "A deterministic seed evolving under B3/S23."
explanation = "A dead cell with exactly three live neighbours is born; a live cell with two or three survives."
metric      = "population"
citation    = "M. Gardner, \"Mathematical Games\", Scientific American 223(4), 120–123 (1970)."
doi         = "10.1038/scientificamerican1070-120"
```

- Copy stays consistent with existing honesty statements: Lenia is described as
  a quantized, independent-XY-plane variant and not isotropic 3-D Lenia; the
  Turing example claims no universality.
- Captions are static; live numbers stay in readouts.
- Planned references, each **verified against doi.org before commit**: Wolfram
  1983 (Rule 30), Cook 2004 (Rule 110), Gardner 1970 (Life),
  Hardy–Pomeau–de Pazzis 1973 (HPP), Rabin–Scott 1959 (NFA), Chomsky 1956
  (pushdown, parentheses), Lindenmayer 1968 (Koch), Kuroda 1964 (LBA), Turing
  1937 (Turing), Reynolds 1987 (Boids), Deneubourg et al. 1990 (Ants), Chan
  2019 (Lenia).
- DOI links open in the system browser.
- Seed and input controls appear only where they exist today: Lenia Blob/Ring
  and editable symbolic input applied on Reset. Grid and agent seeds are shown
  read-only.

## 5. Error handling

| Condition | Response |
| --- | --- |
| Ninth channel requested | Drawer footer note: "8 of 8 open — close one to add another" |
| Input over 256 symbols | Margin note under the input; Reset refused (existing rule) |
| DOI fails to open | Short serif toast at the stage bottom and a log entry |
| GPU metric readback error | `Unavailable(reason)` shown in place of the trace |
| Unknown or removed `ChannelId` | Command ignored, debug log |
| Malformed `specimens.toml` | Caught by unit test; fonts and metadata are compile-time embedded |

## 6. Testing and qualification

**Unit (no GPU).** Token contrast (`ink`, `ink_dim` and `signal` on both `bg`
and `panel` ≥ 4.5:1); Oklab ramp endpoints and monotonic lightness; `StageRect` → physical
viewport including scale factor; shortcut map and text-focus suppression;
presenter fade timer; inspector width clamping; specimen metadata completeness,
field presence, DOI syntax (`10.\d{4,9}/…`) and known groups; `ChannelId`
non-reuse; `Trace` capacity, min/max and epoch clearing; GPU adapter throttling
and error mapping against a fake probe.

**Integration (`lib/tests`).** `ChannelCommand` routing on CPU sessions for every
action; per-channel clocks at 1 Hz and 60 Hz over simulated time; `Channels`
snapshot after each command; the same command set on a Lenia entity using the
existing Vulkan/Mesa infrastructure; Lenia mass reduction against the CPU
reference density.

**Native qualification.** `verify-automata` and `verify-lenia` keep their current
flows and gain a channel-command pass performing the same lifecycle. The image
check counts signal-colour pixels inside `StageRect` instead of cyan across the
window, and asserts that no panel overlaps the stage. Evidence is recaptured on
the RTX 3090 and on Mesa lavapipe, with before/after screenshots. The window
stays 1280 × 720 at (80, 80) on monitor 1.

**Repository practice.** Commit at least every 100 changed source lines;
`make check` per completed work item; kanban tickets in `docs/kanban/` for this
sub-project; README updated for controls, shortcuts and screenshots;
`.superpowers/` added to `.gitignore`.

## 7. Out of scope

Guided tours and annotations (3); probes, comparison, snapshots, recording,
export and multi-metric readouts (4); main menu and scenario redesign (5);
rule and seed authoring, including new seed choices such as R-pentomino (6);
history rewinding; split or multiple simultaneous stages.

## 8. Amendment — rebased onto `main` (2026-10-19)

The spec above was written against a branch 49 commits behind `main`. `main`
already ships an experiment workbench: validated per-family `Settings`
(rate 1–60 Hz, seed patterns, rule edits, populations), a comparison window
(pair stepping, side-by-side views, export bundles), project save/load with
exact GPU checkpoints, and a `verify-experiment` qualification flow. The user
chose to revise the design onto `main` and keep all of it. Changes:

- **Experiment settings move into the inspector** as a `parameters` margin
  section (restyled fields, *Apply live*, *Apply and reset*, *Defaults*). The
  UI sends a models-level `ConfigureSession { id, settings, reset }` event; the
  CPU router calls `Session::configure` and stores any refusal as a per-session
  note shown under the section. `experiment::metrics::describe` appears as a
  `measured` line in the readout section.
- **Comparison and Project become rail drawers** beside the Library (one
  drawer open at a time): *Compare* keeps left/right selection (by channel),
  pair stepping, measurements, side-by-side painters and exports; *Project*
  keeps path, pause-all, save (with GPU checkpoint capture) and load.
- **Rate** follows `Settings::rate_hz` (continuous 1–60). The transport offers
  presets 1 / 10 / 30 / 60, highlighting an exact match. `ChannelAction::SetRate`
  carries `f32`; Lenia rounds to whole Hz for its integer step clock and keeps
  its 60 Hz default. The CPU per-session clock already exists (`Session::elapse`),
  so no new CPU clock type is added.
- **Seed choices exist on `main`** (`Pattern`), so the inspector exposes them
  where valid instead of showing seeds read-only.
- **Persistence compatibility.** `ChannelId` and the reset epoch live on
  `Session` as `#[serde(skip)]` fields; the specimen colour lives in `Settings`
  with `#[serde(default)]`, so existing project files still load. Because
  project load replaces sessions wholesale, the CPU router reconciles ids each
  frame (adopts unassigned sessions, releases vanished ids); `RestoreLenia`
  allocates ids like `SeedPattern`.
- **Shared readback mailbox.** Project save and the opt-in mass metric both use
  `LeniaProbe`. A `ReadbackBusy` resource set while a save is pending stops the
  mass sampler from requesting or taking snapshots.
- **Module placement.** `workbench/` already holds `evolution/`, `experiment/`
  and `project/`; `experiment` and `project` move under `workbench/operations/`
  (with re-exports preserving their public paths) beside the new `channels/`,
  and `catalog.rs` becomes `catalog/` holding the specimen metadata.
- **Qualification.** `verify-experiment` keeps passing; it imports the project
  `Files`/`Pending` types from their new `ui::lab` location.
