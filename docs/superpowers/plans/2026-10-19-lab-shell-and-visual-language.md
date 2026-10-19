# Lab Shell and Visual Language Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace S.O.U.L.'s floating egui windows with a viewer-first automata lab (rail, library drawer, channel tabs, stage, inspector, transport, presenter mode) in the Instrument / S4 Marginalia / Libertinus visual language, driven by one backend-neutral channel contract for CPU sessions and GPU Lenia.

**Architecture:** A new `engine_core::automata::channel` module defines `ChannelId`, `ChannelCommand`, the `Channels` snapshot, pacing clocks, colour ramps and a metrics pipeline. The CPU workbench (`models`) and the GPU Lenia command executor (`engine-render`) route commands and publish rows in explicitly ordered system sets. `lib/ui` becomes a theme module plus a `lab` shell that only reads snapshots and writes commands. The world camera renders into the stage rectangle the shell publishes.

**Tech Stack:** Rust 2024, Bevy 0.16.1, bevy_egui 0.35.1 (egui 0.31.1), wgpu/WGSL, `toml` 0.9, Libertinus 7.051 (SIL OFL), Python 3.11 structure audit.

**Spec:** `docs/superpowers/specs/2026-10-19-lab-shell-and-visual-language-design.md`

## Revision for `main` (read before any task)

The tasks below were drafted against a stale branch. Spec §8 records the
rebase onto `main`; these deltas override the task text where they conflict.

| Task | Change |
| --- | --- |
| 1 | Drop `TimeClock` (CPU sessions already pace themselves with `Session::elapse`). Replace `Rate` with `pub const RATE_PRESETS: [f32; 4] = [1.0, 10.0, 30.0, 60.0]`; `ChannelSettings { rate_hz: f32, colour: Srgb }`; `ChannelAction::SetRate(f32)`. `Srgb` derives `Serialize`/`Deserialize`. Add `ChannelId::UNASSIGNED` and `impl Default for ChannelId` (= unassigned). |
| 2 | Unchanged. |
| 3 | Replaced: move `workbench/{experiment,project}` under `workbench/operations/` with `pub use operations::{experiment, project};`, and turn `catalog.rs` into `catalog/mod.rs`. `evolution/` already groups the model families. |
| 4 | `Session` gains `#[serde(skip)] channel`, `#[serde(skip)] epoch`, `#[serde(skip)] note: Option<String>`; `Settings` gains `#[serde(default = ...)] colour: Srgb`. Router lives in `workbench/operations/channels/`; `SetRate` writes `settings.rate_hz` clamped to 1–60; ticking stays `session.elapse`. Add `ConfigureSession { id, settings: Box<Settings>, reset }` event and an id-reconcile system (adopt `UNASSIGNED`, release vanished). Tests adapt (`rate_hz`). |
| 5 | As written; module path `workbench/operations/channels/metrics.rs`. |
| 6 | `RestoreLenia` also allocates `(ChannelId, ChannelSettings::new(60.0), StepClock)`; `StepClock::due` takes `rate_hz.round() as u32`. |
| 7 | Sampler skips request/take while `Res<ReadbackBusy>` is true (new `engine_core::automata::lenia::ReadbackBusy(pub bool)`); project save sets it while pending. |
| 9 | Specimens live in `workbench/catalog/specimens.rs` (+ `tests.rs`). |
| 13 | `LabState.drawer: Option<Drawer>` with `Drawer::{Library, Compare, Project}` replaces `drawer_open`. Drawers live in `lab/chrome/drawers/{mod.rs,library.rs,compare.rs,project.rs}`; `lab/library.rs` is not created. The rail gets Compare (`≡`) and Project (`§`) entries. |
| 14 | Inspector adds `parameters` (restyled experiment fields + Apply live / Apply and reset / Defaults → `ConfigureSession`), the session `note`, and a `measured` line from `experiment::metrics::describe`. |
| 16 | Comparison and persistence code moves into the Compare/Project drawers; `Files`, `Pending`, `Comparison` keep their names under `ui::lab::chrome::drawers`. Update `src/app/preview/experiment/checkpoint.rs` imports. |
| 17 | Also run `make verify-experiment` (both drivers) and keep its evidence current. |

## Global Constraints

- Structural governance (`structural-governance.toml`): every governed directory holds 0–3 child directories and 1–4 `.rs`/`.wgsl`/`.py`/`.ps1` files; no source file above **200** non-comment source lines; run `python3 tools/structure/audit.py` after every task.
- Commit at every **100** changed source lines (added plus removed nonblank, non-comment) or sooner; end each commit message with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- `make check` (governance self-tests + audit, `cargo fmt --check`, workspace Clippy with all targets/features and `-D warnings`, native tests) must pass at the end of every task.
- Public types and functions carry doc comments; invariants and synchronisation are explained in comments; `snake_case` files, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants.
- Palette tokens (exact): bg `#0d0f10`, panel `#121415`, raised `#191c1d`, track `#232728`, ink `#c9c4b4`, ink_hi `#ece6d4`, ink_dim `#8a8476`, signal `#ffb547`, signal_bg `#2a2416`, ok `#7bd88f`, fault `#ff6b5b`, info `#6fb7ff`.
- Type scale (logical px): label 9, body 11, title 15, readout 24, presenter HUD 40. Serif = titles, tab names, margin labels, captions; sans = body; mono = numbers, rule strings, identifiers.
- No hairline borders anywhere: every egui widget/window/separator stroke is zero width.
- Layout: rail 32 px; inspector default 260 px, drag range 220–420 px, collapsible; tabs ≈ 28 px; transport ≈ 40 px; stage is the remainder.
- Rates: 1, 10, 30, 60 steps/s. CPU channels default to 10 (today's rate). Lenia channels default to 60, which equals the fixed simulation rate they run at today (see "Spec clarifications").
- Channel capacity is 8 open channels across both backends. `ChannelId`s are never reused within a run and are never shown in the UI.
- Specimen copy must not overclaim: Lenia is "quantized, independent XY planes, zero boundaries, not isotropic 3-D Lenia"; the Turing example "is not a universal machine".
- Application windows stay on monitor 1 at (80, 80), 1280 × 720.
- Do not modify unrelated working-tree changes.

## Spec clarifications made while planning

These follow from code read during planning; each is the smallest change consistent with the approved spec. Mention them in the PR description.

1. **Step key is `.` (period), not `→`.** The arrow keys already pan the camera (`engine-common` input), and the spec resolves conflicts in the camera's favour. `Space` stays run/pause, but is left to the camera while a `FreeCam` exists.
2. **Lenia's default rate is 60 steps/s.** Lenia currently advances once per fixed `SimulationStep` (60 Hz from `AppConfig::simulation_rate_hz`). Defaulting it to 10 would silently slow every verified Lenia flow.
3. **Per-channel Lenia colour is resolved in the fragment shader by world rectangle.** All Lenia instances share one indirect draw, so the palette uniform holds up to 8 `(world rect, 4 ramp stops)` entries. The selected channel's entry comes first, so it wins wherever instances overlap.
4. **The rail's library icon is painted** (three book spines), because Libertinus has no `❦`. `⁂` (settings) is present in Libertinus Serif. egui's default fonts stay as fallbacks for symbols Libertinus lacks.
5. **The symbolic inspector shows a transition table for the Turing machine** (the only symbolic model with an explicit table) and a state/stack/tape summary for the others.
6. **The UI reads `Workbench` and `LeniaSimulation` immutably** to paint models and show Lenia parameters. All mutation goes through `ChannelCommand`/`AutomataCommand`.

## Review Focus

Spec-implied inputs and failure modes that no task's main tests would otherwise catch. Each line has a pinning test in the task named.

1. **A ninth spawn across mixed backends** (for example 7 CPU + 2 Lenia) must be refused, without leaking an id or a GPU slice. Pinned in Task 6 (`lenia_spawn_refused_when_full`).
2. **A GPU readback that never completes** (no adapter, lost device) must turn into `Unavailable("GPU readback timed out")` within 2 s, not wait forever. Pinned in Task 7 (`stalled_readback_times_out`).
3. **Removing the selected channel** must select its predecessor (or the first row), never `None` while rows remain, and the stage must not keep framing a despawned instance. Pinned in Task 1 (`settle_selects_predecessor_after_removal`) and Task 12 (`frame_selected_ignores_missing_slice`).
4. **Pressing Escape while the drawer is open or in presenter mode** must close the drawer or exit presenter, and must not also open the pause menu. Pinned in Task 13 (`escape_is_swallowed_only_when_lab_consumes_it`).
5. **A zero-sized or off-window stage** (inspector dragged wide on a narrow window, or a minimised window) must not produce an invalid camera viewport, which panics wgpu. Pinned in Task 12 (`degenerate_stage_has_no_viewport`).

---
## File Structure

```
lib/engine/engine-core/src/automata/
  mod.rs                         (modify: `pub mod channel;`)
  channel/
    mod.rs                       ChannelId, ChannelIds, Rate, Backend, Status, ChannelSettings, ChannelRow, Channels, ChannelSet, ChannelPlugin
    command.rs                   ChannelAction, ChannelCommand
    clock.rs                     StepClock (integer, fixed steps), TimeClock (wall time)
    colour.rs                    Srgb, SIGNAL, BACKGROUND, SWATCHES, ramp (Oklab), linear()
    metrics/
      mod.rs                     MetricId, MetricSample, MetricAvailability, ChannelMetric, Metrics, MetricSource, mass()
      trace.rs                   Trace ring buffer
lib/engine/engine-core/src/automata/lenia/commands.rs   (modify: apply_commands in ChannelSet::Tick)

lib/models/src/workbench/
  mod.rs, catalog.rs             (modify mod.rs)
  plugin.rs                      (modify: ids, lifecycle, system sets)
  session.rs                     (modify: channel, settings, epoch, clock)
  families/{agents,grid,symbolic}/   (moved unchanged from workbench/)
  families/mod.rs
  channels/
    mod.rs                       route, tick, publish
    metrics.rs                   family MetricIds, SessionMetric, value(), publish
    tests.rs                     metric unit tests
  specimens/
    mod.rs                       Specimen, Group, Specimens, valid_doi
    tests.rs
assets/specimens/specimens.toml  (data, outside source limits)
assets/fonts/libertinus/*.otf, OFL.txt

lib/engine/engine-render/src/command_executor/
  mod.rs                         (modify: allocate ids on Lenia spawn, release on clear)
  channels.rs                    route, publish for Lenia owners
  metrics.rs                     MASS, MassSampling, SnapshotMailbox, toggle, sample
  tests.rs                       adapter tests with a fake mailbox
lib/engine/engine-gpu/src/plugin/setup/ticks.rs        (modify: per-channel StepClock)
lib/engine/engine-gpu/src/plugin/drawing/palette.rs    PaletteEntry, PaletteUniform, build, extract
lib/engine/engine-gpu/src/plugin/drawing/{pipeline.rs,voxel_draw_node.rs}  (modify: binding 1)
lib/engine/engine-gpu/assets/voxel_mesh.wgsl           (modify: palette ramp)

lib/engine/engine-common/src/controls/camera/manager/
  stage.rs                       StageRect, StageCamera, physical_viewport, apply_stage, frame_selected, run conditions

lib/ui/src/
  lib.rs, plugin.rs              (modify)
  theme/  mod.rs tokens.rs fonts.rs visuals.rs  fade/mod.rs  widgets/{mod.rs,margin.rs,readout.rs,controls.rs}
  lab/    mod.rs state.rs shortcuts.rs library.rs
          chrome/{mod.rs,rail.rs,tabs.rs,transport.rs}
          inspector/{mod.rs,sections.rs,machine.rs,lenia.rs}
          stage/{mod.rs,view.rs,overlay.rs,presenter.rs}  stage/painters/{mod.rs,grid.rs,symbolic.rs,agents.rs}
  components/menus/meta/pause_menu/{mod.rs,plugin.rs}   (moved from presentation/panels/world)
  removed: styles/, presentation/, components/banners/, components/menus/automata/

lib/tests/src/channels/{mod.rs,cpu.rs,gpu.rs}
src/app/preview/{scene.rs,automata/*,lenia/verification/*}   (modify: StageCamera, labels, channel pass, stage pixel check)
```

---

### Task 1: Channel contract core (ids, rates, rows, commands, clocks, colour, plugin)

**Files:**
- Create: `lib/engine/engine-core/src/automata/channel/mod.rs`
- Create: `lib/engine/engine-core/src/automata/channel/command.rs`
- Create: `lib/engine/engine-core/src/automata/channel/clock.rs`
- Create: `lib/engine/engine-core/src/automata/channel/colour.rs`
- Create: `lib/engine/engine-core/src/automata/channel/metrics/mod.rs` (stub here, filled in Task 2)
- Modify: `lib/engine/engine-core/src/automata/mod.rs` (add `pub mod channel;` after `pub mod lenia;`)

**Interfaces:**
- Consumes: `engine_core::automata::lenia::LeniaAction` (existing, `Copy`), `engine_core::systems::schedule::MainSet` (existing; import path `crate::systems::schedule::MainSet`, as used in `engine-render`).
- Produces:
  - `ChannelId(pub u32)`: `Component + Copy + Eq + Ord + Hash`.
  - `ChannelIds::{CAPACITY: usize = 8, allocate(&mut self) -> Option<ChannelId>, release(&mut self, ChannelId), open(&self) -> usize}`.
  - `Rate::{Hz1, Hz10, Hz30, Hz60}`, with `Rate::ALL` and `Rate::hz(self) -> u32`.
  - `Backend::{Cpu, Gpu}` and `Status::{Running, Paused, Halted}`.
  - `ChannelSettings { rate: Rate, colour: Srgb }`: a `Component`, with `ChannelSettings::new(rate)` defaulting the colour to `SIGNAL`.
  - `ChannelRow { id, kind: String, backend, status, tick: u64, epoch: u64, settings }`.
  - `Channels { rows: Vec<ChannelRow>, selected: Option<ChannelId> }`, with `publish(&mut self, Backend, rows)`, `get(ChannelId)`, `selected_row()`, `index_of(ChannelId)` and `is_full()`.
  - `ChannelSet::{Route, Tick, Publish, Settle}`: chained in `Update`, with `Route` after `MainSet::Input`.
  - `ChannelPlugin`: idempotent via `app.is_plugin_added`.
  - `ChannelAction::{Select, Pause(bool), Step, Reset, Remove, SetRate(Rate), SetColour(Srgb), SetInput(String), SampleMetric(bool), Lenia(LeniaAction)}` and `ChannelCommand { id, action }` (an `Event`).
  - `StepClock::due(&mut self, steps: u64, rate_hz: u32, base_hz: u32) -> u64`.
  - `TimeClock::{due(&mut self, elapsed_secs: f64, rate_hz: u32) -> u64, reset(&mut self)}`.
  - `Srgb(pub [u8; 3])` with `Srgb::linear(self) -> [f32; 3]` and `Srgb::lightness(self) -> f32`.
  - `SIGNAL`, `BACKGROUND`, `SWATCHES: [Srgb; 5]` and `ramp(Srgb) -> [Srgb; 4]`.

- [ ] **Step 1: Write the failing tests**

Create `lib/engine/engine-core/src/automata/channel/clock.rs` with only the tests first:

```rust
//! Pacing clocks that convert elapsed simulation steps or wall time into channel ticks.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_clock_passes_through_at_base_rate() {
        let mut clock = StepClock::default();
        let total: u64 = (0..1000).map(|_| clock.due(1, 60, 60)).sum();
        assert_eq!(total, 1000);
    }

    #[test]
    fn step_clock_divides_exactly_with_integer_carry() {
        let mut clock = StepClock::default();
        let total: u64 = (0..60).map(|_| clock.due(1, 10, 60)).sum();
        assert_eq!(total, 10);
        assert_eq!(clock.due(5, 10, 60), 0, "5/6 of a tick carried");
        assert_eq!(clock.due(1, 10, 60), 1);
    }

    #[test]
    fn time_clock_matches_rate_over_irregular_frames() {
        let mut clock = TimeClock::default();
        let total: u64 = (0..144).map(|_| clock.due(1.0 / 144.0, 60)).sum();
        assert_eq!(total, 60);
        let mut ten = TimeClock::default();
        let total: u64 = (0..10).map(|_| ten.due(0.1, 10)).sum();
        assert_eq!(total, 10, "exact tenths must not lose a tick to rounding");
    }

    #[test]
    fn time_clock_caps_a_stalled_frame() {
        let mut clock = TimeClock::default();
        assert_eq!(clock.due(5.0, 10), 5, "elapsed is capped at 0.5 s");
        clock.reset();
        assert_eq!(clock.due(0.05, 10), 0);
    }
}
```

Create `lib/engine/engine-core/src/automata/channel/colour.rs` with only tests:

```rust
//! sRGB colour currency and perceptually even ramps shared by CPU painters and the voxel shader.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp_keeps_the_chosen_colour_as_its_third_stop() {
        for colour in SWATCHES {
            let stop = ramp(colour)[2].0;
            for (a, b) in stop.iter().zip(colour.0) {
                assert!(a.abs_diff(b) <= 1, "{colour:?} round-trips through Oklab");
            }
        }
    }

    #[test]
    fn ramp_lightness_is_strictly_increasing() {
        for colour in SWATCHES {
            let l: Vec<f32> = ramp(colour).iter().map(|s| s.lightness()).collect();
            assert!(l.windows(2).all(|w| w[0] < w[1]), "{colour:?}: {l:?}");
            assert!(l[0] < 0.3, "first stop sits near the background");
        }
    }

    #[test]
    fn linear_conversion_hits_known_points() {
        assert_eq!(Srgb([0, 0, 0]).linear(), [0.0, 0.0, 0.0]);
        let white = Srgb([255, 255, 255]).linear();
        assert!(white.iter().all(|c| (c - 1.0).abs() < 1e-6));
    }
}
```

Create `lib/engine/engine-core/src/automata/channel/mod.rs` with the module skeleton and its tests:

```rust
//! Backend-neutral channel contract shared by the lab UI, qualification and both backends.
//!
//! Commands flow UI → `ChannelCommand` → backend routers (`ChannelSet::Route`); backends
//! tick (`Tick`), publish rows into [`Channels`] (`Publish`), and `Settle` repairs selection.
mod clock;
mod colour;
mod command;
pub mod metrics;

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn row(id: u32, backend: Backend) -> ChannelRow {
        ChannelRow {
            id: ChannelId(id),
            kind: "life:conway".into(),
            backend,
            status: Status::Paused,
            tick: 0,
            epoch: 0,
            settings: ChannelSettings::new(Rate::Hz10),
        }
    }

    #[test]
    fn ids_are_never_reused_and_capacity_is_shared() {
        let mut ids = ChannelIds::default();
        let first: Vec<_> = (0..ChannelIds::CAPACITY).map(|_| ids.allocate().unwrap()).collect();
        assert_eq!(ids.allocate(), None, "ninth channel refused");
        ids.release(first[3]);
        let next = ids.allocate().unwrap();
        assert_eq!(next, ChannelId(8), "released ids are not recycled");
        assert_eq!(ids.open(), ChannelIds::CAPACITY);
        ids.release(ChannelId(999));
        assert_eq!(ids.open(), ChannelIds::CAPACITY, "unknown release is ignored");
    }

    #[test]
    fn publish_replaces_one_backend_and_keeps_creation_order() {
        let mut channels = Channels::default();
        channels.publish(Backend::Gpu, [row(1, Backend::Gpu)]);
        channels.publish(Backend::Cpu, [row(2, Backend::Cpu), row(0, Backend::Cpu)]);
        let order: Vec<_> = channels.rows.iter().map(|r| r.id.0).collect();
        assert_eq!(order, [0, 1, 2]);
        channels.publish(Backend::Cpu, []);
        assert_eq!(channels.rows.len(), 1);
        assert!(!channels.is_full());
    }

    fn settle_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ChannelPlugin));
        app
    }

    #[test]
    fn settle_selects_predecessor_after_removal() {
        let mut app = settle_app();
        let mut channels = app.world_mut().resource_mut::<Channels>();
        channels.publish(Backend::Cpu, [row(0, Backend::Cpu), row(2, Backend::Cpu), row(5, Backend::Cpu)]);
        channels.selected = Some(ChannelId(2));
        app.update();
        let mut channels = app.world_mut().resource_mut::<Channels>();
        channels.publish(Backend::Cpu, [row(0, Backend::Cpu), row(5, Backend::Cpu)]);
        app.update();
        assert_eq!(app.world().resource::<Channels>().selected, Some(ChannelId(0)));
        let mut channels = app.world_mut().resource_mut::<Channels>();
        channels.publish(Backend::Cpu, [row(5, Backend::Cpu)]);
        app.update();
        assert_eq!(app.world().resource::<Channels>().selected, Some(ChannelId(5)), "falls back to first");
        app.world_mut().resource_mut::<Channels>().publish(Backend::Cpu, []);
        app.update();
        assert_eq!(app.world().resource::<Channels>().selected, None);
    }

    #[test]
    fn select_command_targets_only_existing_rows() {
        let mut app = settle_app();
        app.world_mut()
            .resource_mut::<Channels>()
            .publish(Backend::Cpu, [row(0, Backend::Cpu), row(1, Backend::Cpu)]);
        app.world_mut().send_event(ChannelCommand { id: ChannelId(1), action: ChannelAction::Select });
        app.update();
        assert_eq!(app.world().resource::<Channels>().selected, Some(ChannelId(1)));
        app.world_mut().send_event(ChannelCommand { id: ChannelId(9), action: ChannelAction::Select });
        app.update();
        assert_eq!(app.world().resource::<Channels>().selected, Some(ChannelId(1)));
    }

    #[test]
    fn rates_map_to_hertz() {
        assert_eq!(Rate::ALL.map(Rate::hz), [1, 10, 30, 60]);
        assert_eq!(Rate::default(), Rate::Hz10);
    }
}
```

Create `lib/engine/engine-core/src/automata/channel/metrics/mod.rs` containing only:

```rust
//! Metric contract shared by CPU sources and the GPU adapter (filled in by Task 2).
```

Add `pub mod channel;` to `lib/engine/engine-core/src/automata/mod.rs` directly below `pub mod lenia;`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p engine-core channel`
Expected: compile errors such as `cannot find type StepClock`, `cannot find function ramp`, `cannot find type ChannelIds`.

- [ ] **Step 3: Implement the clocks**

Prepend to `clock.rs`, above the tests:

```rust
use bevy::prelude::Component;

/// Converts fixed `SimulationStep`s into channel ticks exactly.
///
/// Integer carry avoids the float drift that would drop ticks when the
/// channel rate equals the base rate (e.g. 60 of 60 Hz must pass through).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepClock {
    carry: u64,
}

impl StepClock {
    /// Ticks due after `steps` base steps at `base_hz`, for a channel at `rate_hz`.
    pub fn due(&mut self, steps: u64, rate_hz: u32, base_hz: u32) -> u64 {
        let base = u64::from(base_hz.max(1));
        let total = self.carry + steps * u64::from(rate_hz);
        self.carry = total % base;
        total / base
    }
}

/// Converts frame wall time into channel ticks for CPU sessions.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TimeClock {
    carry: f64,
}

impl TimeClock {
    /// Longest frame credited at once, matching the original 10 Hz loop's guard.
    pub const MAX_FRAME_SECS: f64 = 0.5;

    /// Ticks due after `elapsed_secs` at `rate_hz`; a small epsilon keeps exact
    /// multiples of the period (0.1 s × 10) from rounding down.
    pub fn due(&mut self, elapsed_secs: f64, rate_hz: u32) -> u64 {
        let period = 1.0 / f64::from(rate_hz.max(1));
        self.carry += elapsed_secs.clamp(0.0, Self::MAX_FRAME_SECS);
        let ticks = ((self.carry + 1e-9) / period).floor();
        self.carry = (self.carry - ticks * period).max(0.0);
        ticks as u64
    }

    /// Forget accumulated time, e.g. after a reset.
    pub fn reset(&mut self) {
        self.carry = 0.0;
    }
}
```

- [ ] **Step 4: Implement colour**

Prepend to `colour.rs`, above the tests:

```rust
/// Display colour as sRGB bytes: the lab's colour currency across crates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Srgb(pub [u8; 3]);

/// Amber signal colour and default specimen colour.
pub const SIGNAL: Srgb = Srgb([0xff, 0xb5, 0x47]);
/// Window and stage background.
pub const BACKGROUND: Srgb = Srgb([0x0d, 0x0f, 0x10]);
/// Palette offered by the inspector: signal, ink-hi, ok, info, fault.
pub const SWATCHES: [Srgb; 5] = [
    SIGNAL,
    Srgb([0xec, 0xe6, 0xd4]),
    Srgb([0x7b, 0xd8, 0x8f]),
    Srgb([0x6f, 0xb7, 0xff]),
    Srgb([0xff, 0x6b, 0x5b]),
];

impl Srgb {
    /// Linear-light RGB in 0..=1, as shaders and Oklab expect.
    pub fn linear(self) -> [f32; 3] {
        self.0.map(|u| {
            let c = f32::from(u) / 255.0;
            if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
        })
    }

    /// Oklab perceptual lightness in 0..=1.
    pub fn lightness(self) -> f32 {
        oklab(self)[0]
    }
}

/// Four perceptually even stops: near-background, dim, the colour, a light tint.
pub fn ramp(colour: Srgb) -> [Srgb; 4] {
    let (bg, c, white) = (oklab(BACKGROUND), oklab(colour), oklab(Srgb([255; 3])));
    [mix(bg, c, 0.12), mix(bg, c, 0.55), c, mix(c, white, 0.45)].map(from_oklab)
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
}

fn oklab(colour: Srgb) -> [f32; 3] {
    let [r, g, b] = colour.linear();
    let l = (0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

fn from_oklab([l, a, b]: [f32; 3]) -> Srgb {
    let l_ = (l + 0.396_337_78 * a + 0.215_803_76 * b).powi(3);
    let m_ = (l - 0.105_561_346 * a - 0.063_854_17 * b).powi(3);
    let s_ = (l - 0.089_484_18 * a - 1.291_485_5 * b).powi(3);
    let gamma = |x: f32| {
        let c = if x <= 0.003_130_8 { 12.92 * x } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 };
        (c.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    Srgb([
        gamma(4.076_741_7 * l_ - 3.307_711_6 * m_ + 0.230_969_94 * s_),
        gamma(-1.268_438 * l_ + 2.609_757_4 * m_ - 0.341_319_38 * s_),
        gamma(-0.004_196_086_3 * l_ - 0.703_418_6 * m_ + 1.707_614_7 * s_),
    ])
}
```

- [ ] **Step 5: Implement commands**

Create `command.rs`:

```rust
//! The single command vocabulary accepted by every channel backend.
use super::{ChannelId, Rate, Srgb};
use crate::automata::lenia::LeniaAction;
use bevy::prelude::Event;

/// A requested change to one channel; backends ignore actions that do not apply to them.
#[derive(Clone, Debug)]
pub enum ChannelAction {
    /// Make this channel the one shown on the stage.
    Select,
    /// Suspend (`true`) or resume (`false`) automatic ticks; halted channels stay paused.
    Pause(bool),
    /// Advance exactly one tick, only while paused.
    Step,
    /// Restore the initial condition, pause, and start a new metric epoch.
    Reset,
    /// Close the channel and release its id and resources.
    Remove,
    /// Change automatic stepping rate.
    SetRate(Rate),
    /// Change the specimen colour used by painters and the voxel shader.
    SetColour(Srgb),
    /// Replace symbolic input; applied by the next `Reset`.
    SetInput(String),
    /// Enable or disable an opt-in metric source (Lenia GPU mass readback).
    SampleMetric(bool),
    /// Lenia-specific parameter and seed actions.
    Lenia(LeniaAction),
}

/// Channel command event, written by the lab UI and native qualification.
#[derive(Event, Clone, Debug)]
pub struct ChannelCommand {
    /// Target channel; unknown or removed ids are ignored by every backend.
    pub id: ChannelId,
    /// Requested change.
    pub action: ChannelAction,
}
```

- [ ] **Step 6: Implement the contract types and plugin**

Insert into `channel/mod.rs` between the `pub mod metrics;` line and the tests:

```rust
use crate::systems::schedule::MainSet;
use bevy::prelude::*;
pub use clock::{StepClock, TimeClock};
pub use colour::{BACKGROUND, SIGNAL, SWATCHES, Srgb, ramp};
pub use command::{ChannelAction, ChannelCommand};

/// Stable handle for one open session; monotonically allocated, never reused, never shown.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChannelId(pub u32);

/// Run-wide id allocator; also enforces the shared eight-channel capacity.
#[derive(Resource, Debug, Default)]
pub struct ChannelIds {
    next: u32,
    open: Vec<ChannelId>,
}

impl ChannelIds {
    /// Maximum simultaneously open channels across all backends.
    pub const CAPACITY: usize = 8;

    /// Allocate a fresh id, or `None` when eight channels are open.
    pub fn allocate(&mut self) -> Option<ChannelId> {
        if self.open.len() >= Self::CAPACITY {
            return None;
        }
        let id = ChannelId(self.next);
        self.next += 1;
        self.open.push(id);
        Some(id)
    }

    /// Return capacity; ids that are not open are ignored.
    pub fn release(&mut self, id: ChannelId) {
        self.open.retain(|open| *open != id);
    }

    /// Number of currently open channels.
    pub fn open(&self) -> usize {
        self.open.len()
    }
}

/// Automatic stepping rate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Rate {
    /// One step per second.
    Hz1,
    /// Ten steps per second (the CPU default).
    #[default]
    Hz10,
    /// Thirty steps per second.
    Hz30,
    /// Sixty steps per second (the Lenia default).
    Hz60,
}

impl Rate {
    /// Every selectable rate, slowest first.
    pub const ALL: [Self; 4] = [Self::Hz1, Self::Hz10, Self::Hz30, Self::Hz60];

    /// Steps per second.
    pub fn hz(self) -> u32 {
        match self {
            Self::Hz1 => 1,
            Self::Hz10 => 10,
            Self::Hz30 => 30,
            Self::Hz60 => 60,
        }
    }
}

/// Where a channel executes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    /// Inspectable CPU session in `models::workbench`.
    Cpu,
    /// GPU Lenia owner entity.
    Gpu,
}

/// Lifecycle state shown by tab lamps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Advancing automatically.
    Running,
    /// Waiting for resume or single steps.
    Paused,
    /// Terminal or resource-bounded; only Reset continues.
    Halted,
}

/// Per-channel pacing and colour, owned by the backend.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct ChannelSettings {
    /// Automatic stepping rate.
    pub rate: Rate,
    /// Specimen colour.
    pub colour: Srgb,
}

impl ChannelSettings {
    /// Settings at `rate` in the default amber.
    pub fn new(rate: Rate) -> Self {
        Self { rate, colour: SIGNAL }
    }
}

/// One read-only row published by a backend each frame.
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelRow {
    /// Stable handle.
    pub id: ChannelId,
    /// Catalog id, e.g. `"life:conway"` or `"lenia"`.
    pub kind: String,
    /// Executing backend.
    pub backend: Backend,
    /// Lamp state.
    pub status: Status,
    /// Ticks since the latest reset (requested ticks for Lenia).
    pub tick: u64,
    /// Reset generation; metric samples from older epochs are discarded.
    pub epoch: u64,
    /// Pacing and colour.
    pub settings: ChannelSettings,
}

/// Snapshot of every open channel plus the stage selection.
#[derive(Resource, Debug, Default)]
pub struct Channels {
    /// Rows in creation (id) order.
    pub rows: Vec<ChannelRow>,
    /// Channel shown on the stage; repaired by `ChannelSet::Settle`.
    pub selected: Option<ChannelId>,
}

impl Channels {
    /// Replace every row of `backend`, keeping creation order across backends.
    pub fn publish(&mut self, backend: Backend, rows: impl IntoIterator<Item = ChannelRow>) {
        self.rows.retain(|row| row.backend != backend);
        self.rows.extend(rows);
        self.rows.sort_by_key(|row| row.id);
    }

    /// Row for `id`, if open.
    pub fn get(&self, id: ChannelId) -> Option<&ChannelRow> {
        self.rows.iter().find(|row| row.id == id)
    }

    /// Position of `id` in tab order.
    pub fn index_of(&self, id: ChannelId) -> Option<usize> {
        self.rows.iter().position(|row| row.id == id)
    }

    /// Row shown on the stage.
    pub fn selected_row(&self) -> Option<&ChannelRow> {
        self.selected.and_then(|id| self.get(id))
    }

    /// Whether the shared capacity is exhausted.
    pub fn is_full(&self) -> bool {
        self.rows.len() >= ChannelIds::CAPACITY
    }
}

/// Per-frame ordering: route commands → tick → publish rows → settle selection.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChannelSet {
    /// Apply `ChannelCommand`s and spawn/clear requests.
    Route,
    /// Advance clocks and simulations.
    Tick,
    /// Publish rows and metric samples.
    Publish,
    /// Repair selection and drop state of closed channels.
    Settle,
}

/// Registers the contract; every backend adds it, so registration is idempotent.
pub struct ChannelPlugin;

impl Plugin for ChannelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChannelCommand>()
            .init_resource::<Channels>()
            .init_resource::<ChannelIds>()
            .init_resource::<metrics::Metrics>()
            .configure_sets(
                Update,
                (ChannelSet::Route, ChannelSet::Tick, ChannelSet::Publish, ChannelSet::Settle)
                    .chain()
                    .after(MainSet::Input),
            )
            .add_systems(Update, (select.in_set(ChannelSet::Route), settle.in_set(ChannelSet::Settle)));
    }
}

/// Add [`ChannelPlugin`] unless another backend already did.
pub fn ensure_plugin(app: &mut App) {
    if !app.is_plugin_added::<ChannelPlugin>() {
        app.add_plugins(ChannelPlugin);
    }
}

fn select(mut commands: EventReader<ChannelCommand>, mut channels: ResMut<Channels>) {
    for command in commands.read() {
        if matches!(command.action, ChannelAction::Select) && channels.get(command.id).is_some() {
            channels.selected = Some(command.id);
        }
    }
}

/// After removal, select the closest earlier channel, else the first; never point at a closed id.
fn settle(mut channels: ResMut<Channels>, mut metrics: ResMut<metrics::Metrics>) {
    let missing = channels.selected.filter(|id| channels.get(*id).is_none());
    if let Some(gone) = missing {
        let previous = channels.rows.iter().rev().find(|row| row.id < gone);
        channels.selected = previous.or(channels.rows.first()).map(|row| row.id);
    } else if channels.selected.is_none() {
        channels.selected = channels.rows.first().map(|row| row.id);
    }
    let open: Vec<ChannelId> = channels.rows.iter().map(|row| row.id).collect();
    metrics.retain(|id| open.contains(&id));
}
```

`settle` calls `metrics::Metrics::retain`, which Task 2 implements. To make Task 1 compile on its own, put this temporary minimal type in `metrics/mod.rs`; Task 2 replaces the file:

```rust
//! Metric contract shared by CPU sources and the GPU adapter (completed in Task 2).
use super::ChannelId;
use bevy::prelude::Resource;

/// Metric state keyed by channel.
#[derive(Resource, Debug, Default)]
pub struct Metrics(pub std::collections::HashMap<ChannelId, ()>);

impl Metrics {
    /// Keep only channels for which `open` holds.
    pub fn retain(&mut self, open: impl Fn(ChannelId) -> bool) {
        self.0.retain(|id, _| open(*id));
    }
}
```

`MainSet` is declared in `systems/execution/schedule/mod.rs`. Confirm the public path with `grep -rn "pub use\|pub mod" lib/engine/engine-core/src/systems/mod.rs`. `engine-render` uses `engine_core::systems::schedule::MainSet`, so `crate::systems::schedule::MainSet` is correct inside the crate.

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test -p engine-core channel`
Expected: all 12 tests in `automata::channel` pass.

- [ ] **Step 8: Check structure, format and lint**

Run: `python3 tools/structure/audit.py && cargo fmt --all && cargo clippy -p engine-core --all-targets -- -D warnings`
Expected: the audit reports no violations, and Clippy is clean. `channel/` holds exactly four files (`mod.rs`, `command.rs`, `clock.rs`, `colour.rs`) and one directory (`metrics/`). If `mod.rs` exceeds 200 SLOC, move `Rate`, `Backend` and `Status` (with their impls) into `command.rs` and re-export them from `mod.rs`.

- [ ] **Step 9: Commit**

```bash
git add lib/engine/engine-core/src/automata
git commit -m "feat: add backend-neutral channel contract with pacing clocks and colour ramps

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---
### Task 2: Metrics foundation (samples, traces, availability, sources)

**Files:**
- Modify (replace): `lib/engine/engine-core/src/automata/channel/metrics/mod.rs`
- Create: `lib/engine/engine-core/src/automata/channel/metrics/trace.rs`

**Interfaces:**
- Consumes: `ChannelId` (Task 1).
- Produces:
  - `MetricId { name: &'static str, label: &'static str, unit: &'static str }` (`Copy + Eq + Hash`).
  - `MetricSample { tick: u64, epoch: u64, value: f64 }`.
  - `MetricAvailability::{Live, Sampled { period_secs: f32 }, OptIn, Unavailable(String)}`.
  - `Trace::{CAPACITY = 200, new(capacity), push(&mut self, MetricSample) -> bool, clear(), samples() -> impl Iterator<Item = &MetricSample>, len(), is_empty(), range() -> Option<(f64, f64)>}`.
  - `ChannelMetric { id, availability, latest: Option<MetricSample>, trace: Trace }`.
  - `Metrics(pub HashMap<ChannelId, ChannelMetric>)`, with `declare(channel, id, availability)`, `record(channel, sample) -> bool`, `get(channel) -> Option<&ChannelMetric>` and `retain(Fn(ChannelId) -> bool)`.
  - `trait MetricSource { fn describe(&self) -> (MetricId, MetricAvailability); fn sample(&mut self) -> Option<MetricSample>; }`.
  - `fn mass(densities: &[u8]) -> f64` (sum / 255).

- [ ] **Step 1: Write the failing tests**

`trace.rs` (tests only):

```rust
//! Fixed-capacity per-channel metric history with epoch isolation.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automata::channel::metrics::MetricSample;

    fn s(tick: u64, epoch: u64, value: f64) -> MetricSample {
        MetricSample { tick, epoch, value }
    }

    #[test]
    fn evicts_oldest_beyond_capacity() {
        let mut trace = Trace::new(3);
        for t in 0..5 {
            assert!(trace.push(s(t, 0, t as f64)));
        }
        let ticks: Vec<u64> = trace.samples().map(|x| x.tick).collect();
        assert_eq!(ticks, [2, 3, 4]);
    }

    #[test]
    fn newer_epoch_clears_and_older_epoch_is_rejected() {
        let mut trace = Trace::new(10);
        trace.push(s(5, 1, 1.0));
        assert!(!trace.push(s(9, 0, 9.0)), "stale epoch");
        assert!(trace.push(s(0, 2, 2.0)));
        assert_eq!(trace.len(), 1);
    }

    #[test]
    fn same_tick_replaces_and_older_tick_is_rejected() {
        let mut trace = Trace::new(10);
        trace.push(s(4, 0, 1.0));
        assert!(trace.push(s(4, 0, 2.0)));
        assert_eq!(trace.len(), 1);
        assert_eq!(trace.samples().last().unwrap().value, 2.0);
        assert!(!trace.push(s(3, 0, 0.0)));
    }

    #[test]
    fn range_spans_min_and_max() {
        let mut trace = Trace::new(10);
        assert_eq!(trace.range(), None);
        for (t, v) in [(0, 3.0), (1, -1.0), (2, 7.5)] {
            trace.push(s(t, 0, v));
        }
        assert_eq!(trace.range(), Some((-1.0, 7.5)));
    }
}
```

At the bottom of the new `metrics/mod.rs` (full file content follows in Step 3), add:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const POP: MetricId = MetricId { name: "population", label: "alive", unit: "cells" };

    #[test]
    fn record_requires_declaration_and_tracks_latest() {
        let mut metrics = Metrics::default();
        let id = ChannelId(3);
        assert!(!metrics.record(id, MetricSample { tick: 0, epoch: 0, value: 1.0 }));
        metrics.declare(id, POP, MetricAvailability::Live);
        assert!(metrics.record(id, MetricSample { tick: 1, epoch: 0, value: 4.0 }));
        assert_eq!(metrics.get(id).unwrap().latest.unwrap().value, 4.0);
        metrics.declare(id, POP, MetricAvailability::Unavailable("lost".into()));
        assert_eq!(metrics.get(id).unwrap().trace.len(), 1, "redeclaring keeps history");
        metrics.retain(|open| open != id);
        assert!(metrics.get(id).is_none());
    }

    #[test]
    fn mass_is_density_sum_over_255() {
        assert_eq!(mass(&[255; 8]), 8.0);
        assert_eq!(mass(&[]), 0.0);
        assert!((mass(&[51, 51, 51, 51, 51]) - 1.0).abs() < 1e-12);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p engine-core metrics`
Expected: compile errors for `Trace`, `MetricId`, `mass`.

- [ ] **Step 3: Implement**

Prepend to `trace.rs`:

```rust
use super::MetricSample;
use std::collections::VecDeque;

/// Ring buffer of one channel's headline metric, isolated per reset epoch.
#[derive(Clone, Debug, PartialEq)]
pub struct Trace {
    samples: VecDeque<MetricSample>,
    capacity: usize,
    epoch: Option<u64>,
}

impl Trace {
    /// Samples kept per channel (the transport sparkline width).
    pub const CAPACITY: usize = 200;

    /// Empty trace holding at most `capacity` samples.
    pub fn new(capacity: usize) -> Self {
        Self { samples: VecDeque::with_capacity(capacity), capacity: capacity.max(1), epoch: None }
    }

    /// Append; a newer epoch clears history, an older epoch or tick is rejected,
    /// and a repeated tick (e.g. while paused) replaces the last value.
    pub fn push(&mut self, sample: MetricSample) -> bool {
        match self.epoch {
            Some(epoch) if sample.epoch < epoch => return false,
            Some(epoch) if sample.epoch > epoch => self.samples.clear(),
            _ => {}
        }
        self.epoch = Some(sample.epoch);
        if let Some(last) = self.samples.back_mut() {
            if sample.tick < last.tick {
                return false;
            }
            if sample.tick == last.tick {
                *last = sample;
                return true;
            }
        }
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
        true
    }

    /// Forget all samples and the epoch.
    pub fn clear(&mut self) {
        self.samples.clear();
        self.epoch = None;
    }

    /// Samples oldest first.
    pub fn samples(&self) -> impl Iterator<Item = &MetricSample> {
        self.samples.iter()
    }

    /// Number of samples held.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether no samples are held.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Minimum and maximum value, for scaling the sparkline.
    pub fn range(&self) -> Option<(f64, f64)> {
        self.samples.iter().fold(None, |acc, s| match acc {
            None => Some((s.value, s.value)),
            Some((lo, hi)) => Some((lo.min(s.value), hi.max(s.value))),
        })
    }
}
```

Replace `metrics/mod.rs` with:

```rust
//! Metric contract shared by CPU sources and the GPU adapter.
//!
//! Backends `declare` a channel's headline metric with its availability, then
//! `record` samples; the UI only reads. Sub-project 4 adds reductions and
//! consumers (probes, export, comparison) on top of these types.
use super::ChannelId;
use bevy::prelude::Resource;
use std::collections::HashMap;
mod trace;
pub use trace::Trace;

/// Stable metric identity and display text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MetricId {
    /// Machine name used by `specimens.toml`, e.g. `"population"`.
    pub name: &'static str,
    /// Short readout label, e.g. `"alive"`.
    pub label: &'static str,
    /// Unit shown in notes.
    pub unit: &'static str,
}

/// One measurement tied to the tick and reset epoch it describes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetricSample {
    /// Channel tick the value was measured at.
    pub tick: u64,
    /// Reset generation; traces never mix epochs.
    pub epoch: u64,
    /// Measured value.
    pub value: f64,
}

/// How (and whether) a metric is currently produced.
#[derive(Clone, Debug, PartialEq)]
pub enum MetricAvailability {
    /// Sampled synchronously after every tick.
    Live,
    /// Sampled asynchronously at most once per period.
    Sampled {
        /// Seconds between requests.
        period_secs: f32,
    },
    /// Available but disabled until the viewer opts in.
    OptIn,
    /// Not producible; the reason is shown in the inspector.
    Unavailable(String),
}

/// Headline metric state for one channel.
#[derive(Clone, Debug)]
pub struct ChannelMetric {
    /// Which metric.
    pub id: MetricId,
    /// Current production mode.
    pub availability: MetricAvailability,
    /// Most recent accepted sample.
    pub latest: Option<MetricSample>,
    /// Recent history.
    pub trace: Trace,
}

/// Metric state keyed by channel; backends write, the UI reads.
#[derive(Resource, Debug, Default)]
pub struct Metrics(pub HashMap<ChannelId, ChannelMetric>);

impl Metrics {
    /// Create or update a channel's metric; history survives redeclaration of the same id.
    pub fn declare(&mut self, channel: ChannelId, id: MetricId, availability: MetricAvailability) {
        let entry = self.0.entry(channel).or_insert_with(|| ChannelMetric {
            id,
            availability: availability.clone(),
            latest: None,
            trace: Trace::new(Trace::CAPACITY),
        });
        if entry.id != id {
            entry.id = id;
            entry.latest = None;
            entry.trace.clear();
        }
        entry.availability = availability;
    }

    /// Record a sample for a declared channel; returns whether the trace accepted it.
    pub fn record(&mut self, channel: ChannelId, sample: MetricSample) -> bool {
        let Some(metric) = self.0.get_mut(&channel) else {
            return false;
        };
        let accepted = metric.trace.push(sample);
        if accepted {
            metric.latest = Some(sample);
        }
        accepted
    }

    /// Metric state for `channel`.
    pub fn get(&self, channel: ChannelId) -> Option<&ChannelMetric> {
        self.0.get(&channel)
    }

    /// Keep only channels for which `open` holds.
    pub fn retain(&mut self, open: impl Fn(ChannelId) -> bool) {
        self.0.retain(|id, _| open(*id));
    }
}

/// Shape shared by CPU sources and the GPU adapter: describe, then sample when ready.
pub trait MetricSource {
    /// Metric identity and how it is produced.
    fn describe(&self) -> (MetricId, MetricAvailability);
    /// A sample if one is ready now.
    fn sample(&mut self) -> Option<MetricSample>;
}

/// Total Lenia mass: quantized densities summed and rescaled to 0..=1 per cell.
pub fn mass(densities: &[u8]) -> f64 {
    densities.iter().map(|d| f64::from(*d)).sum::<f64>() / 255.0
}
```

Then append the Step 1 `#[cfg(test)] mod tests` block.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p engine-core channel`
Expected: Task 1 and Task 2 tests all pass.

- [ ] **Step 5: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy -p engine-core --all-targets -- -D warnings
git add lib/engine/engine-core/src/automata/channel/metrics
git commit -m "feat: add epoch-isolated metric traces and source contract

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Move workbench model directories under `families/`

A mechanical move that frees file slots in `workbench/`. There are no behaviour changes, and existing tests are the safety net.

**Files:**
- Move: `lib/models/src/workbench/{agents,grid,symbolic}` → `lib/models/src/workbench/families/{agents,grid,symbolic}`
- Create: `lib/models/src/workbench/families/mod.rs`
- Modify: `lib/models/src/workbench/mod.rs`, plus every `use` of `workbench::{agents,grid,symbolic}`

**Interfaces:**
- Produces: the same public paths through re-exports (`models::workbench::grid`, `models::workbench::symbolic`, `models::workbench::agents` keep working), so nothing downstream changes.

- [ ] **Step 1: Record the passing baseline**

Run: `cargo test -p models workbench 2>&1 | tail -3`
Expected: `test result: ok.` Note the passed count.

- [ ] **Step 2: Move and re-export**

```bash
mkdir -p lib/models/src/workbench/families
git mv lib/models/src/workbench/agents lib/models/src/workbench/families/agents
git mv lib/models/src/workbench/grid lib/models/src/workbench/families/grid
git mv lib/models/src/workbench/symbolic lib/models/src/workbench/families/symbolic
```

Create `lib/models/src/workbench/families/mod.rs`:

```rust
//! Executable model families: cellular grids, symbolic machines and agent arenas.
pub mod agents;
pub mod grid;
pub mod symbolic;
```

Replace `lib/models/src/workbench/mod.rs` with:

```rust
//! Runnable automata catalog and isolated, bounded CPU workbench sessions.
pub mod catalog;
pub mod families;
pub mod plugin;
pub mod session;
pub use families::{agents, grid, symbolic};
```

Inside the moved files, `super::catalog::Kind` and `super::super::...` paths now sit one level deeper. Fix them by running:

```bash
grep -rn "super::catalog\|super::super::catalog\|crate::workbench::catalog" lib/models/src/workbench/families
```

Replace each `super::catalog` (written from a family root such as `grid/mod.rs`) with `crate::workbench::catalog`.

- [ ] **Step 3: Run the tests to verify nothing changed**

Run: `cargo test -p models workbench 2>&1 | tail -3 && cargo build --workspace`
Expected: the same passed count as Step 1, and the workspace builds.

- [ ] **Step 4: Audit and commit**

```bash
python3 tools/structure/audit.py
git add -A lib/models/src/workbench
git commit -m "refactor: group workbench model families under one directory

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: CPU channel routing and per-session clocks

**Files:**
- Modify: `lib/models/src/workbench/session.rs`
- Modify: `lib/models/src/workbench/plugin.rs`
- Create: `lib/models/src/workbench/channels/mod.rs`
- Create: `lib/models/src/workbench/channels/metrics.rs` (stub `pub(in crate::workbench) fn publish() {}`, completed in Task 5)
- Modify: `lib/models/src/workbench/mod.rs` (`pub mod channels;`)
- Modify: `lib/tests/src/families.rs` (new `spawn`/`clear` signatures)
- Create: `lib/tests/src/channels/mod.rs`, `lib/tests/src/channels/cpu.rs`
- Modify: `lib/tests/src/lib.rs` (`#[cfg(test)] mod channels;`)

**Interfaces:**
- Consumes: Task 1 `ChannelId`, `ChannelIds`, `ChannelSettings`, `Rate`, `TimeClock`, `Channels`, `ChannelRow`, `ChannelCommand`, `ChannelAction`, `ChannelSet`, `ensure_plugin`.
- Produces:
  - `Session` gains `pub channel: ChannelId`, `pub settings: ChannelSettings`, `pub epoch: u64` and `pub clock: TimeClock`. `Session::new(kind)` sets `channel = ChannelId::UNASSIGNED` (`ChannelId(u32::MAX)`, a constant added in Task 1's `mod.rs`).
  - `Session::with_channel(self, ChannelId) -> Self`, and `Session::status(&self) -> Status`.
  - `Workbench::spawn(&mut self, kind, ids: &mut ChannelIds) -> Option<ChannelId>` and `Workbench::clear(&mut self, ids: &mut ChannelIds)`.
  - `models::workbench::channels::{route, tick, publish}` systems, `pub(in crate::workbench)`.
  - `pub const FULL_NOTE: &str = "8 of 8 open — close one to add another";` in `plugin.rs`.

- [ ] **Step 1: Write the failing integration tests**

Add to `lib/tests/Cargo.toml` `[dependencies]` (needed by Task 6 too):

```toml
engine-render = { path = "../engine/engine-render" }
```

Create `lib/tests/src/channels/mod.rs`:

```rust
//! Channel-contract routing against the CPU workbench and the GPU Lenia executor.
mod cpu;
mod gpu;
```

Create `lib/tests/src/channels/gpu.rs` holding only `//! Lenia channel routing (Task 6).` for now.

Create `lib/tests/src/channels/cpu.rs`:

```rust
//! Every ChannelAction against real CPU sessions, with simulated wall time.
use bevy::{prelude::*, state::app::StatesPlugin, time::TimeUpdateStrategy};
use engine_core::{
    automata::channel::{ChannelAction, ChannelCommand, ChannelId, ChannelRow, Channels, Rate, SWATCHES, Status},
    events::AutomataCommand,
    prelude::AppState,
};
use models::workbench::plugin::{Workbench, WorkbenchPlugin};
use std::time::Duration;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_event::<AutomataCommand>()
        .add_plugins(WorkbenchPlugin)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(100)));
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::InGame);
    app.update();
    app.update();
    app
}

fn spawn(app: &mut App, id: &str) -> Option<ChannelId> {
    let before = app.world().resource::<Channels>().rows.len();
    app.world_mut().send_event(AutomataCommand::SeedPattern { id: id.into() });
    app.update();
    let channels = app.world().resource::<Channels>();
    (channels.rows.len() > before).then(|| channels.rows.last().unwrap().id)
}

fn send(app: &mut App, id: ChannelId, action: ChannelAction) {
    app.world_mut().send_event(ChannelCommand { id, action });
    app.update();
}

fn row(app: &App, id: ChannelId) -> ChannelRow {
    app.world().resource::<Channels>().get(id).cloned().expect("row published")
}

#[test]
fn spawn_publishes_a_paused_selected_row() {
    let mut app = app();
    let id = spawn(&mut app, "life:conway").unwrap();
    let r = row(&app, id);
    assert_eq!((r.status, r.tick, r.kind.as_str()), (Status::Paused, 0, "life:conway"));
    assert_eq!(r.settings.rate, Rate::Hz10);
    assert_eq!(app.world().resource::<Channels>().selected, Some(id));
}

#[test]
fn running_channels_advance_at_their_own_rate() {
    let mut app = app();
    let fast = spawn(&mut app, "life:conway").unwrap();
    let slow = spawn(&mut app, "wolfram:rule30").unwrap();
    send(&mut app, fast, ChannelAction::SetRate(Rate::Hz60));
    send(&mut app, slow, ChannelAction::SetRate(Rate::Hz1));
    send(&mut app, fast, ChannelAction::Pause(false));
    send(&mut app, slow, ChannelAction::Pause(false));
    let (f0, s0) = (row(&app, fast).tick, row(&app, slow).tick);
    for _ in 0..10 {
        app.update(); // 10 × 100 ms = 1 s
    }
    assert_eq!(row(&app, fast).tick - f0, 60);
    assert_eq!(row(&app, slow).tick - s0, 1);
    assert_eq!(row(&app, fast).status, Status::Running);
}

#[test]
fn step_only_while_paused_and_reset_bumps_epoch() {
    let mut app = app();
    let id = spawn(&mut app, "life:conway").unwrap();
    send(&mut app, id, ChannelAction::Step);
    assert_eq!(row(&app, id).tick, 1);
    send(&mut app, id, ChannelAction::Pause(false));
    let running = row(&app, id).tick;
    app.world_mut().send_event(ChannelCommand { id, action: ChannelAction::Step });
    app.update();
    assert_eq!(row(&app, id).tick, running + 1, "only the 10 Hz clock advanced");
    send(&mut app, id, ChannelAction::Reset);
    let r = row(&app, id);
    assert_eq!((r.tick, r.status, r.epoch), (0, Status::Paused, 1));
}

#[test]
fn remove_releases_shared_capacity() {
    let mut app = app();
    let ids: Vec<_> = (0..8).map(|_| spawn(&mut app, "wolfram:rule30").unwrap()).collect();
    assert_eq!(spawn(&mut app, "wolfram:rule30"), None);
    assert!(app.world().resource::<Workbench>().error.is_some());
    send(&mut app, ids[2], ChannelAction::Remove);
    assert!(app.world().resource::<Channels>().get(ids[2]).is_none());
    let again = spawn(&mut app, "wolfram:rule30").unwrap();
    assert!(again.0 > ids[7].0, "ids are not reused");
}

#[test]
fn colour_and_input_apply_and_unknown_ids_are_ignored() {
    let mut app = app();
    let id = spawn(&mut app, "regular:nfa").unwrap();
    send(&mut app, id, ChannelAction::SetColour(SWATCHES[3]));
    send(&mut app, id, ChannelAction::SetInput("ab".into()));
    send(&mut app, id, ChannelAction::Reset);
    assert_eq!(row(&app, id).settings.colour, SWATCHES[3]);
    assert_eq!(app.world().resource::<Workbench>().sessions[0].input, "ab");
    send(&mut app, ChannelId(999), ChannelAction::Remove);
    assert_eq!(app.world().resource::<Channels>().rows.len(), 1);
}

#[test]
fn halted_machines_report_halted_and_refuse_resume() {
    let mut app = app();
    let id = spawn(&mut app, "regular:nfa").unwrap();
    for _ in 0..8 {
        send(&mut app, id, ChannelAction::Step);
    }
    assert_eq!(row(&app, id).status, Status::Halted);
    send(&mut app, id, ChannelAction::Pause(false));
    assert_eq!(row(&app, id).status, Status::Halted);
}

#[test]
fn clear_and_main_menu_release_every_id() {
    let mut app = app();
    spawn(&mut app, "life:conway");
    spawn(&mut app, "swarm:boids");
    app.world_mut().send_event(AutomataCommand::Clear);
    app.update();
    assert!(app.world().resource::<Channels>().rows.is_empty());
    assert_eq!(app.world().resource::<engine_core::automata::channel::ChannelIds>().open(), 0);
}
```

Update `lib/tests/src/families.rs` `capacity_and_invalid_inputs_do_not_mutate_other_sessions`:
- add `let mut ids = engine_core::automata::channel::ChannelIds::default();` as the first line;
- replace `bench.spawn(kind);` with `bench.spawn(kind, &mut ids);`, `bench.clear();` with `bench.clear(&mut ids);`, and `bench.spawn(Kind::Ants);` with `bench.spawn(Kind::Ants, &mut ids);`.

Add `#[cfg(test)] mod channels;` to `lib/tests/src/lib.rs`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p tests channels::cpu`
Expected: compile errors, such as `spawn` taking 1 argument, or `no field settings on ChannelRow` errors coming from missing CPU publishing.

- [ ] **Step 3: Extend `Session`**

In `lib/engine/engine-core/src/automata/channel/mod.rs`, add to `impl ChannelId` (create the impl block):

```rust
impl ChannelId {
    /// Placeholder for sessions built outside a workbench (tests, examples).
    pub const UNASSIGNED: Self = Self(u32::MAX);
}
```

In `session.rs`, add the fields and methods (keep the existing ones unchanged):

```rust
use engine_core::automata::channel::{ChannelId, ChannelSettings, Rate, Status, TimeClock};

pub struct Session {
    pub kind: Kind,
    pub input: String,
    pub model: Model,
    pub ticks: u64,
    pub paused: bool,
    /// Channel handle; `ChannelId::UNASSIGNED` until a workbench adopts the session.
    pub channel: ChannelId,
    /// Pacing and colour.
    pub settings: ChannelSettings,
    /// Reset generation for metric traces.
    pub epoch: u64,
    /// Per-session wall-time clock driven by `settings.rate`.
    pub clock: TimeClock,
}
```

In `Session::new`, add `channel: ChannelId::UNASSIGNED, settings: ChannelSettings::new(Rate::Hz10), epoch: 0, clock: TimeClock::default(),`. In `reset`, after `self.paused = true;`, add `self.epoch += 1; self.clock.reset();`. Add:

```rust
    /// Adopt a workbench-allocated channel id.
    pub fn with_channel(mut self, channel: ChannelId) -> Self {
        self.channel = channel;
        self
    }
    /// Lamp state for the channel row.
    pub fn status(&self) -> Status {
        if self.halted() {
            Status::Halted
        } else if self.paused {
            Status::Paused
        } else {
            Status::Running
        }
    }
```

- [ ] **Step 4: Route, tick, publish**

Create `lib/models/src/workbench/channels/mod.rs`:

```rust
//! Channel-contract routing, per-session clocks and row publishing for CPU sessions.
use super::{plugin::Workbench, session::Session};
use bevy::prelude::*;
use engine_core::{
    automata::channel::{Backend, ChannelAction, ChannelCommand, ChannelIds, ChannelRow, Channels},
    prelude::AppState,
};
pub mod metrics;
#[cfg(test)]
mod tests;

/// Apply commands addressed to CPU sessions; ids owned by other backends are skipped.
pub(in crate::workbench) fn route(
    mut commands: EventReader<ChannelCommand>,
    mut bench: ResMut<Workbench>,
    mut ids: ResMut<ChannelIds>,
) {
    for command in commands.read() {
        let Some(index) = bench.sessions.iter().position(|s| s.channel == command.id) else {
            continue;
        };
        let session = &mut bench.sessions[index];
        match &command.action {
            ChannelAction::Pause(paused) if !session.halted() => session.paused = *paused,
            ChannelAction::Step => session.single_step(),
            // Oversized input is refused by Session::reset; the inspector explains why.
            ChannelAction::Reset => drop(session.reset()),
            ChannelAction::SetRate(rate) => session.settings.rate = *rate,
            ChannelAction::SetColour(colour) => session.settings.colour = *colour,
            ChannelAction::SetInput(text) => session.input = text.clone(),
            ChannelAction::Remove => {
                let removed = bench.sessions.remove(index);
                ids.release(removed.channel);
                bench.selected = bench.selected.min(bench.sessions.len().saturating_sub(1));
            }
            _ => {}
        }
    }
}

/// Advance each running session by whole ticks of its own rate.
pub(in crate::workbench) fn tick(
    mut bench: ResMut<Workbench>,
    time: Option<Res<Time>>,
    state: Option<Res<State<AppState>>>,
) {
    if state.is_some_and(|s| *s.get() != AppState::InGame) {
        return;
    }
    let elapsed = time.map_or(0.0, |t| f64::from(t.delta_secs()));
    for session in &mut bench.sessions {
        let due = session.clock.due(elapsed, session.settings.rate.hz());
        for _ in 0..due {
            session.advance();
        }
    }
}

/// Replace the CPU rows of the channel snapshot.
pub(in crate::workbench) fn publish(bench: Res<Workbench>, mut channels: ResMut<Channels>) {
    channels.publish(Backend::Cpu, bench.sessions.iter().map(row));
}

fn row(session: &Session) -> ChannelRow {
    ChannelRow {
        id: session.channel,
        kind: session.kind.id().into(),
        backend: Backend::Cpu,
        status: session.status(),
        tick: session.ticks,
        epoch: session.epoch,
        settings: session.settings,
    }
}
```

`channels/tests.rs` is created in Task 5. For now, omit the `#[cfg(test)] mod tests;` line, or create the file containing only a `//!` doc line.

Create `channels/metrics.rs` (stub):

```rust
//! Headline metric per CPU family (completed in Task 5).
use crate::workbench::plugin::Workbench;
use bevy::prelude::*;

/// Publish CPU metric samples.
pub(in crate::workbench) fn publish(_bench: Res<Workbench>) {}
```

Replace the body of `plugin.rs` (keep its doc comment):

```rust
use super::{catalog::Kind, channels, session::Session};
use bevy::prelude::*;
use engine_core::{
    automata::channel::{ChannelId, ChannelIds, ChannelSet, Channels, ensure_plugin},
    events::AutomataCommand,
    prelude::AppState,
};

/// Shown when the shared eight-channel capacity is exhausted.
pub const FULL_NOTE: &str = "8 of 8 open — close one to add another";

/// Isolated CPU sessions; selection here is legacy, the stage uses `Channels::selected`.
#[derive(Resource, Default)]
pub struct Workbench {
    pub sessions: Vec<Session>,
    pub selected: usize,
    pub error: Option<String>,
}
impl Workbench {
    /// Add a paused instance unless the shared capacity is exhausted.
    pub fn spawn(&mut self, kind: Kind, ids: &mut ChannelIds) -> Option<ChannelId> {
        let Some(id) = ids.allocate() else {
            self.error = Some(FULL_NOTE.into());
            return None;
        };
        self.sessions.push(Session::new(kind).with_channel(id));
        self.selected = self.sessions.len() - 1;
        self.error = None;
        Some(id)
    }
    /// Release every instance and its channel id.
    pub fn clear(&mut self, ids: &mut ChannelIds) {
        for session in self.sessions.drain(..) {
            ids.release(session.channel);
        }
        self.selected = 0;
        self.error = None;
    }
}

/// Installs spawn/clear routing, channel commands and per-session clocks.
pub struct WorkbenchPlugin;
impl Plugin for WorkbenchPlugin {
    fn build(&self, app: &mut App) {
        ensure_plugin(app);
        app.init_resource::<Workbench>().add_systems(
            Update,
            (
                (lifecycle, channels::route).chain().in_set(ChannelSet::Route),
                channels::tick.in_set(ChannelSet::Tick),
                (channels::publish, channels::metrics::publish).in_set(ChannelSet::Publish),
            ),
        );
    }
}

fn lifecycle(
    mut commands: EventReader<AutomataCommand>,
    mut bench: ResMut<Workbench>,
    mut ids: ResMut<ChannelIds>,
    mut channels: ResMut<Channels>,
    state: Option<Res<State<AppState>>>,
) {
    if state.is_some_and(|s| *s.get() == AppState::MainMenu) {
        commands.clear();
        bench.clear(&mut ids);
        return;
    }
    for command in commands.read() {
        match command {
            AutomataCommand::Clear => bench.clear(&mut ids),
            AutomataCommand::SeedPattern { id } => {
                if let Some(kind) = Kind::parse(id)
                    && let Some(channel) = bench.spawn(kind, &mut ids)
                {
                    channels.selected = Some(channel);
                }
            }
        }
    }
}
```

`lifecycle` reads `AutomataCommand`, which `models::ComputationalIntelligencePlugin` registers. The integration test adds the event explicitly.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p tests channels::cpu && cargo test -p tests families && cargo test -p models`
Expected: all pass. `running_channels_advance_at_their_own_rate` asserts exactly 60 and 1. If the first manual-time update reports zero delta, the `app()` helper's double `update()` absorbs it.

- [ ] **Step 6: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy --workspace --all-targets --all-features -- -D warnings
git add lib/models lib/tests lib/engine/engine-core
git commit -m "feat: route channel commands to CPU sessions with per-session clocks

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

The old `ui` crate still compiles, because it only uses `bench.sessions`, `remove` and `Session` methods. If Clippy flags `bench.sessions.remove` in `lib/ui/.../workbench/mod.rs` leaking ids, leave it: Task 16 deletes that file.

---

### Task 5: CPU headline metrics

**Files:**
- Modify (replace stub): `lib/models/src/workbench/channels/metrics.rs`
- Create: `lib/models/src/workbench/channels/tests.rs`

**Interfaces:**
- Consumes: Task 2 `MetricId`, `MetricSample`, `MetricAvailability`, `MetricSource`, `Metrics`. Task 4 `Session.channel/epoch`.
- Produces: `pub const LIVE_CELLS, POPULATION, PARTICLES, STEPS, LENGTH, POLARISATION, FOOD: MetricId`; `pub fn metric_for(Kind) -> MetricId`; `pub fn value(&Session) -> f64`; `pub struct SessionMetric<'a>(pub &'a Session)` implementing `MetricSource`. Metric `name`s: `live_cells`, `population`, `particles`, `steps`, `length`, `polarisation`, `food` (Task 9's TOML must match).

- [ ] **Step 1: Write the failing tests**

`lib/models/src/workbench/channels/tests.rs`:

```rust
//! Headline metrics are computed from model state exactly as documented.
use super::metrics::{metric_for, value};
use crate::workbench::{catalog::Kind, session::Session};

#[test]
fn every_kind_has_a_named_metric() {
    for kind in Kind::ALL {
        assert!(!metric_for(kind).name.is_empty(), "{kind:?}");
    }
    assert_eq!(metric_for(Kind::Life).name, "population");
    assert_eq!(metric_for(Kind::Turing).name, "steps");
}

#[test]
fn life_counts_the_glider_and_rule30_its_newest_row() {
    assert_eq!(value(&Session::new(Kind::Life)), 5.0);
    let mut rule = Session::new(Kind::Rule30);
    assert_eq!(value(&rule), 1.0);
    rule.single_step();
    assert_eq!(value(&rule), 3.0, "rule 30 from one cell: 111");
}

#[test]
fn hpp_particle_count_is_conserved_before_walls_reach_edges() {
    let mut gas = Session::new(Kind::Hpp);
    let initial = value(&gas);
    assert_eq!(initial, 48.0);
    for _ in 0..12 {
        gas.single_step();
        assert_eq!(value(&gas), initial);
    }
}

#[test]
fn koch_length_symbolic_steps_boids_and_ants() {
    let mut koch = Session::new(Kind::Koch);
    koch.single_step();
    assert_eq!(value(&koch), 8.0, "F → F+F--F+F");
    let mut nfa = Session::new(Kind::Nfa);
    nfa.single_step();
    nfa.single_step();
    assert_eq!(value(&nfa), 2.0);
    let order = value(&Session::new(Kind::Boids));
    assert!((0.0..=1.0).contains(&order));
    assert_eq!(value(&Session::new(Kind::Ants)), 0.0);
}
```

Add `#[cfg(test)] mod tests;` to `channels/mod.rs`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p models channels`
Expected: `cannot find function metric_for`.

- [ ] **Step 3: Implement**

Replace `channels/metrics.rs`:

```rust
//! Headline metric per CPU family, sampled synchronously after each tick.
use crate::{
    automata::dynamical::swarm::boids::Boid,
    workbench::{catalog::Kind, plugin::Workbench, session::{Model, Session}, symbolic::machine::Symbolic},
};
use bevy::prelude::*;
use engine_core::automata::channel::metrics::{MetricAvailability, MetricId, MetricSample, MetricSource, Metrics};

/// Live cells in the newest elementary row.
pub const LIVE_CELLS: MetricId = MetricId { name: "live_cells", label: "alive", unit: "cells" };
/// Live cells of a 2-D grid.
pub const POPULATION: MetricId = MetricId { name: "population", label: "alive", unit: "cells" };
/// HPP particles (one per set direction bit); conserved by collisions.
pub const PARTICLES: MetricId = MetricId { name: "particles", label: "particles", unit: "particles" };
/// Machine transitions performed.
pub const STEPS: MetricId = MetricId { name: "steps", label: "steps", unit: "transitions" };
/// L-system word length.
pub const LENGTH: MetricId = MetricId { name: "length", label: "symbols", unit: "symbols" };
/// Boid alignment: |mean unit heading|, 0 (disordered) to 1 (aligned).
pub const POLARISATION: MetricId = MetricId { name: "polarisation", label: "order", unit: "0–1" };
/// Ant food deliveries home.
pub const FOOD: MetricId = MetricId { name: "food", label: "food", unit: "deliveries" };

/// Headline metric for each family.
pub fn metric_for(kind: Kind) -> MetricId {
    match kind {
        Kind::Rule30 | Kind::Rule110 => LIVE_CELLS,
        Kind::Life => POPULATION,
        Kind::Hpp => PARTICLES,
        Kind::Koch => LENGTH,
        Kind::Boids => POLARISATION,
        Kind::Ants => FOOD,
        _ => STEPS,
    }
}

/// Current metric value from model state.
pub fn value(session: &Session) -> f64 {
    match &session.model {
        Model::Grid(grid) => match session.kind {
            Kind::Rule30 | Kind::Rule110 => grid
                .history
                .back()
                .map_or(0, |row| row.iter().filter(|v| **v != 0).count()) as f64,
            Kind::Hpp => grid.bytes().iter().map(|v| v.count_ones()).sum::<u32>() as f64,
            _ => grid.bytes().iter().filter(|v| **v != 0).count() as f64,
        },
        Model::Symbolic(Symbolic::Koch { word, .. }) => word.chars().count() as f64,
        Model::Symbolic(_) => session.ticks as f64,
        Model::Boids(flock) => polarisation(&flock.boids),
        Model::Ants(arena) => arena.deliveries as f64,
    }
}

fn polarisation(boids: &[Boid]) -> f64 {
    if boids.is_empty() {
        return 0.0;
    }
    let sum: Vec2 = boids.iter().map(|b| b.velocity.normalize_or_zero()).sum();
    f64::from(sum.length() / boids.len() as f32)
}

/// One session viewed as a live metric source.
pub struct SessionMetric<'a>(pub &'a Session);

impl MetricSource for SessionMetric<'_> {
    fn describe(&self) -> (MetricId, MetricAvailability) {
        (metric_for(self.0.kind), MetricAvailability::Live)
    }
    fn sample(&mut self) -> Option<MetricSample> {
        Some(MetricSample { tick: self.0.ticks, epoch: self.0.epoch, value: value(self.0) })
    }
}

/// Declare and sample every CPU session's headline metric.
pub(in crate::workbench) fn publish(bench: Res<Workbench>, mut metrics: ResMut<Metrics>) {
    for session in &bench.sessions {
        let mut source = SessionMetric(session);
        let (id, availability) = source.describe();
        metrics.declare(session.channel, id, availability);
        if let Some(sample) = source.sample() {
            metrics.record(session.channel, sample);
        }
    }
}
```

The `Vec2` used for boids comes from `glam`, which `bevy::prelude` re-exports. If `Boid.velocity` is `glam::Vec2` from a different glam version, use `glam::Vec2` explicitly (`models` depends on `glam` 0.29; check `Boid` in `automata/dynamical/swarm/boids`).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p models channels`
Expected: 4 tests pass. If `hpp_particle_count…` reports a different initial count, look at the `E`/`W` constants in `hpp.rs`. Each wall cell sets one bit, so the count is 2 walls × 24 = 48, and it is the `count_ones` mapping that must hold.

- [ ] **Step 5: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy -p models --all-targets -- -D warnings
git add lib/models/src/workbench/channels
git commit -m "feat: publish headline metrics and traces for CPU families

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---
### Task 6: GPU Lenia channel routing and per-channel rate

**Files:**
- Modify: `lib/engine/engine-render/src/command_executor/mod.rs` (allocate/release ids, register systems)
- Create: `lib/engine/engine-render/src/command_executor/channels.rs`
- Modify: `lib/engine/engine-gpu/src/plugin/setup/ticks.rs` (StepClock pacing + test)
- Modify: `lib/engine/engine-gpu/src/plugin/mod.rs:33` (`request_ticks.in_set(ChannelSet::Tick)`)
- Modify: `lib/engine/engine-core/src/automata/lenia/commands.rs` (`apply_commands.in_set(ChannelSet::Tick)`)
- Modify: `lib/tests/src/channels/gpu.rs`

**Interfaces:**
- Consumes: Task 1 contract; existing `LeniaCommand`, `LeniaAction`, `LeniaSimulation`, `AutomataRegistry`, `AutomatonId`, `AutomatonRemoved`, `FixedStepConfig`.
- Produces:
  - `pub const LENIA_RATE: Rate = Rate::Hz60;`.
  - `engine_render::command_executor::channels::{route, publish}`, both `pub` so tests can schedule them.
  - Lenia owner entities carry `(ChannelId, ChannelSettings, StepClock)`.

- [ ] **Step 1: Write the failing tests**

Append to the `tests` module of `lib/engine/engine-gpu/src/plugin/setup/ticks.rs`:

```rust
    #[test]
    fn paced_lenia_owner_requests_ticks_at_its_channel_rate() {
        use engine_core::automata::channel::{ChannelSettings, Rate, StepClock};
        use engine_core::systems::simulation::FixedStepConfig;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
            .init_state::<AppState>()
            .insert_resource(FixedStepConfig::from_hz(60, 5))
            .add_event::<SimulationStep>()
            .add_systems(Update, request_ticks);
        let owner = app
            .world_mut()
            .spawn((LeniaSimulation::default(), ChannelSettings::new(Rate::Hz10), StepClock::default()))
            .id();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::InGame);
        app.update();
        for _ in 0..60 {
            app.world_mut().send_event(SimulationStep);
        }
        app.update();
        assert_eq!(app.world().get::<LeniaSimulation>(owner).unwrap().target_tick, 10);
    }
```

Replace `lib/tests/src/channels/gpu.rs`:

```rust
//! Lenia channel routing without a GPU: owners are plain entities with the real components.
use bevy::prelude::*;
use engine_core::{
    automata::{
        channel::{Backend, ChannelAction, ChannelCommand, ChannelId, ChannelIds, ChannelPlugin, ChannelSet, ChannelSettings, Channels, Rate, SWATCHES, Status, StepClock},
        lenia::{LeniaControlPlugin, LeniaSimulation},
    },
    events::{AutomatonId, AutomatonRemoved},
    prelude::AppState,
};
use engine_render::command_executor::channels;

fn app() -> (App, ChannelId, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, ChannelPlugin, LeniaControlPlugin))
        .init_resource::<engine_core::prelude::AutomataRegistry>()
        .add_event::<AutomatonRemoved>()
        .add_systems(Update, (channels::route.in_set(ChannelSet::Route), channels::publish.in_set(ChannelSet::Publish)));
    let id = app.world_mut().resource_mut::<ChannelIds>().allocate().unwrap();
    let owner = app
        .world_mut()
        .spawn((AutomatonId(7), id, ChannelSettings::new(Rate::Hz60), StepClock::default(), LeniaSimulation { paused: true, ..default() }))
        .id();
    app.update();
    (app, id, owner)
}

fn send(app: &mut App, id: ChannelId, action: ChannelAction) {
    app.world_mut().send_event(ChannelCommand { id, action });
    app.update();
}

#[test]
fn lenia_rows_publish_as_gpu_channels() {
    let (app, id, _) = app();
    let row = app.world().resource::<Channels>().get(id).cloned().unwrap();
    assert_eq!((row.backend, row.status, row.kind.as_str()), (Backend::Gpu, Status::Paused, "lenia"));
    assert_eq!(row.settings.rate, Rate::Hz60);
}

#[test]
fn lifecycle_actions_map_to_lenia_actions() {
    let (mut app, id, owner) = app();
    send(&mut app, id, ChannelAction::Step);
    assert_eq!(app.world().get::<LeniaSimulation>(owner).unwrap().target_tick, 1);
    send(&mut app, id, ChannelAction::Reset);
    let sim = app.world().get::<LeniaSimulation>(owner).unwrap();
    assert_eq!((sim.target_tick, sim.epoch), (0, 1));
    send(&mut app, id, ChannelAction::Pause(false));
    assert!(!app.world().get::<LeniaSimulation>(owner).unwrap().paused);
    send(&mut app, id, ChannelAction::SetRate(Rate::Hz1));
    send(&mut app, id, ChannelAction::SetColour(SWATCHES[2]));
    let settings = app.world().get::<ChannelSettings>(owner).unwrap();
    assert_eq!((settings.rate, settings.colour), (Rate::Hz1, SWATCHES[2]));
}

#[test]
fn remove_emits_automaton_removed_and_releases_the_id() {
    let (mut app, id, _) = app();
    send(&mut app, id, ChannelAction::Remove);
    let events = app.world().resource::<Events<AutomatonRemoved>>();
    let mut cursor = events.get_cursor();
    let ids: Vec<u32> = cursor.read(events).map(|e| e.id.0).collect();
    assert_eq!(ids, [7], "exactly one removal for the owner's automaton");
    assert_eq!(app.world().resource::<ChannelIds>().open(), 0);
}
```

Also add the Review Focus test. It needs the real spawn path, so put it in the same file using the full executor plugin:

```rust
#[test]
fn lenia_spawn_refused_when_full() {
    use engine_core::{events::AutomataCommand, prelude::RuleRegistry};
    use engine_render::command_executor::CommandExecutorPlugin;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<AppState>()
        .init_resource::<engine_core::prelude::AutomataRegistry>()
        .init_resource::<RuleRegistry>()
        .add_event::<AutomataCommand>()
        .add_event::<engine_core::events::AutomatonAdded>()
        .add_event::<AutomatonRemoved>()
        .add_plugins((LeniaControlPlugin, CommandExecutorPlugin));
    models::automata::dynamical::lenia::plugin::register_rules(app.world_mut());
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::InGame);
    app.update();
    for _ in 0..ChannelIds::CAPACITY {
        app.world_mut().resource_mut::<ChannelIds>().allocate();
    }
    app.world_mut().send_event(AutomataCommand::SeedPattern { id: "lenia".into() });
    app.update();
    assert_eq!(app.world_mut().query::<&LeniaSimulation>().iter(app.world()).count(), 0);
    assert!(app.world().resource::<engine_core::prelude::AutomataRegistry>().list().is_empty());
}
```

`register_rules` stands for whatever the Lenia rule plugin uses. Open `lib/models/src/automata/dynamical/lenia/plugin.rs`. If registration happens inline in `Plugin::build`, call `reg.register_with_seed("lenia", LeniaRule::boxed(), seed_lenia)` directly on `app.world_mut().resource_mut::<RuleRegistry>()` with the same imports instead.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p engine-gpu ticks && cargo test -p tests channels::gpu`
Expected: the tick test fails (`target_tick` is 60, not 10), and the GPU-routing tests fail to compile (`channels` is not public).

- [ ] **Step 3: Implement pacing**

Replace `request_ticks` in `ticks.rs`:

```rust
use engine_core::{
    automata::{channel::{ChannelSettings, StepClock}, lenia::LeniaSimulation},
    prelude::AppState,
    systems::simulation::{FixedStepConfig, SimulationStep},
};

pub(in crate::plugin) fn request_ticks(
    mut events: EventReader<SimulationStep>,
    state: Res<State<AppState>>,
    config: Option<Res<FixedStepConfig>>,
    mut simulations: Query<(&mut LeniaSimulation, Option<(&ChannelSettings, &mut StepClock)>)>,
) {
    let steps = events.read().count() as u64;
    if *state.get() != AppState::InGame {
        return;
    }
    // Base rate of SimulationStep; channels at the base rate pass through exactly.
    let base_hz = config.map_or(60, |c| (1.0 / c.dt.as_secs_f64()).round() as u32);
    for (mut simulation, pacing) in &mut simulations {
        if simulation.paused {
            continue;
        }
        simulation.target_tick += match pacing {
            Some((settings, mut clock)) => clock.due(steps, settings.rate.hz(), base_hz),
            None => steps,
        };
    }
}
```

In `engine-gpu/src/plugin/mod.rs`, change `.add_systems(Update, setup::ticks::request_ticks)` to `.add_systems(Update, setup::ticks::request_ticks.in_set(engine_core::automata::channel::ChannelSet::Tick))`.

In `lenia/commands.rs` `LeniaControlPlugin::build`, register `apply_commands.in_set(crate::automata::channel::ChannelSet::Tick)` and call `crate::automata::channel::ensure_plugin(app);` first. Because `ChannelSet::Tick` runs after `Route` and before `Publish`, the row published in the same frame reflects the command.

- [ ] **Step 4: Implement routing and publishing**

Create `command_executor/channels.rs`:

```rust
//! Channel routing and row publishing for GPU-backed Lenia owners.
use bevy::prelude::*;
use engine_core::{
    automata::{
        channel::{Backend, ChannelAction, ChannelCommand, ChannelId, ChannelIds, ChannelRow, ChannelSettings, Channels, Rate, Status},
        lenia::{LeniaAction, LeniaCommand, LeniaSimulation},
    },
    events::{AutomatonId, AutomatonRemoved},
    prelude::AutomataRegistry,
};

/// Lenia keeps the fixed simulation rate it ran at before channels existed.
pub const LENIA_RATE: Rate = Rate::Hz60;

/// Translate channel commands into Lenia commands, settings changes and removals.
#[allow(clippy::too_many_arguments)]
pub fn route(
    mut commands: EventReader<ChannelCommand>,
    mut owners: Query<(Entity, &ChannelId, &AutomatonId, &mut ChannelSettings), With<LeniaSimulation>>,
    mut lenia: EventWriter<LeniaCommand>,
    mut registry: ResMut<AutomataRegistry>,
    mut removed: EventWriter<AutomatonRemoved>,
    mut ids: ResMut<ChannelIds>,
) {
    for command in commands.read() {
        let Some((entity, id, automaton, mut settings)) = owners.iter_mut().find(|(_, id, ..)| **id == command.id) else {
            continue;
        };
        let mut send = |action| {
            lenia.write(LeniaCommand { entity, action });
        };
        match &command.action {
            ChannelAction::Pause(paused) => send(LeniaAction::Pause(*paused)),
            ChannelAction::Step => send(LeniaAction::Step),
            ChannelAction::Reset => send(LeniaAction::Reset),
            ChannelAction::Lenia(action) => send(*action),
            ChannelAction::SetRate(rate) => settings.rate = *rate,
            ChannelAction::SetColour(colour) => settings.colour = *colour,
            ChannelAction::Remove => {
                // The atlas allocator despawns the owner when it sees AutomatonRemoved.
                registry.remove(*automaton);
                removed.write(AutomatonRemoved { id: *automaton });
                ids.release(*id);
            }
            _ => {}
        }
    }
}

/// Replace the GPU rows of the channel snapshot.
pub fn publish(owners: Query<(&ChannelId, &LeniaSimulation, &ChannelSettings)>, mut channels: ResMut<Channels>) {
    channels.publish(
        Backend::Gpu,
        owners.iter().map(|(id, sim, settings)| ChannelRow {
            id: *id,
            kind: "lenia".into(),
            backend: Backend::Gpu,
            status: if sim.paused { Status::Paused } else { Status::Running },
            tick: sim.target_tick,
            epoch: sim.epoch,
            settings: *settings,
        }),
    );
}
```

In `command_executor/mod.rs`:
- add `pub mod channels;` and make the module public in `engine-render/src/lib.rs` (`pub mod command_executor;`, checking its current visibility);
- in `CommandExecutorPlugin::build`, call `engine_core::automata::channel::ensure_plugin(app);` first, then add `.add_systems(Update, (channels::route.in_set(ChannelSet::Route), channels::publish.in_set(ChannelSet::Publish)))`;
- in `handle_commands`, add the parameters `mut ids: ResMut<ChannelIds>, mut channels: ResMut<Channels>, owners: Query<&ChannelId, With<LeniaSimulation>>`;
- in the Lenia `SeedPattern` branch, **before** `registry.register(info)`:

```rust
                let Some(channel) = ids.allocate() else {
                    warn!("{}", "8 of 8 open — close one to add another");
                    continue;
                };
```

- after `owner.insert(LeniaSimulation { .. })`, also insert `(channel, ChannelSettings::new(channels::LENIA_RATE), StepClock::default())` and set `channels.selected = Some(channel);`. The local `channels` resource shadows the module name, so import the module as `use self::channels as lenia_channels;` and use `lenia_channels::LENIA_RATE`;
- in the `AutomataCommand::Clear` branch and in `clear_on_main_menu` (add the same `owners` and `ids` params), release ids with `for id in &owners { ids.release(*id); }`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p engine-gpu ticks && cargo test -p tests channels`
Expected: all pass, including the existing `lenia_fixed_ticks_respect_both_pause_layers` (unpaced owners pass through).

- [ ] **Step 6: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy --workspace --all-targets --all-features -- -D warnings
git add lib/engine lib/tests
git commit -m "feat: route channel commands to GPU Lenia owners with per-channel pacing

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Opt-in GPU mass metric adapter

**Files:**
- Create: `lib/engine/engine-render/src/command_executor/metrics.rs`
- Create: `lib/engine/engine-render/src/command_executor/tests.rs`
- Modify: `lib/engine/engine-render/src/command_executor/mod.rs` (register; `#[cfg(test)] mod tests;`)
- Modify: `lib/engine/engine-gpu/src/render/compute/lenia/tests/evolution/mod.rs` (mass parity assertion)

**Interfaces:**
- Consumes: Task 2 `Metrics`, `MetricAvailability`, `MetricSample`, `MetricId`, `mass`. Existing `engine_gpu::LeniaProbe::{request, take}` returning `Result<LeniaSnapshot { epoch, tick, densities, .. }, String>`.
- Produces: `pub const MASS: MetricId` (`name: "mass"`), `pub const PERIOD_SECS: f32 = 0.25`, `pub const TIMEOUT_SECS: f32 = 2.0`, `MassSampling` resource, `trait SnapshotMailbox`, and the systems `toggle` and `sample::<M: SnapshotMailbox + Resource>`.

- [ ] **Step 1: Write the failing tests**

`command_executor/tests.rs`:

```rust
//! Mass adapter behaviour against a scripted mailbox; no GPU required.
use super::metrics::{MASS, MassSampling, SnapshotMailbox, sample, toggle};
use bevy::{prelude::*, time::TimeUpdateStrategy};
use engine_core::automata::{
    channel::{ChannelAction, ChannelCommand, ChannelId, ChannelPlugin, metrics::{MetricAvailability, Metrics}},
    lenia::LeniaSimulation,
};
use std::{sync::Mutex, time::Duration};

type Snap = Result<(u64, u64, Vec<u8>), String>;

#[derive(Resource, Default)]
struct Fake {
    requests: Mutex<Vec<Entity>>,
    ready: Mutex<Option<Snap>>,
}
impl SnapshotMailbox for Fake {
    fn request(&self, owner: Entity) {
        self.requests.lock().unwrap().push(owner);
    }
    fn take(&self, _owner: Entity) -> Option<Snap> {
        self.ready.lock().unwrap().take()
    }
}

fn app() -> (App, ChannelId) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, ChannelPlugin))
        .init_resource::<Fake>()
        .init_resource::<MassSampling>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(100)))
        .add_systems(Update, (toggle, sample::<Fake>).chain());
    let id = ChannelId(4);
    app.world_mut().spawn((id, LeniaSimulation::default()));
    app.update();
    (app, id)
}

fn availability(app: &App, id: ChannelId) -> MetricAvailability {
    app.world().resource::<Metrics>().get(id).unwrap().availability.clone()
}

fn enable(app: &mut App, id: ChannelId) {
    app.world_mut().send_event(ChannelCommand { id, action: ChannelAction::SampleMetric(true) });
}

#[test]
fn off_by_default_declares_opt_in_and_never_requests() {
    let (mut app, id) = app();
    for _ in 0..10 {
        app.update();
    }
    assert_eq!(availability(&app, id), MetricAvailability::OptIn);
    assert!(app.world().resource::<Fake>().requests.lock().unwrap().is_empty());
}

#[test]
fn enabled_requests_are_throttled_and_results_recorded() {
    let (mut app, id) = app();
    enable(&mut app, id);
    for _ in 0..10 {
        app.update(); // 1 s
    }
    let requests = app.world().resource::<Fake>().requests.lock().unwrap().len();
    assert_eq!(requests, 1, "waiting for the first result blocks further requests");
    *app.world().resource::<Fake>().ready.lock().unwrap() = Some(Ok((0, 3, vec![255; 4])));
    app.update();
    let metric = app.world().resource::<Metrics>().get(id).unwrap().clone();
    assert_eq!(metric.id, MASS);
    assert_eq!(metric.latest.unwrap().value, 4.0);
    assert!(matches!(metric.availability, MetricAvailability::Sampled { .. }));
}

#[test]
fn readback_error_becomes_unavailable() {
    let (mut app, id) = app();
    enable(&mut app, id);
    for _ in 0..4 {
        app.update();
    }
    *app.world().resource::<Fake>().ready.lock().unwrap() = Some(Err("map failed".into()));
    app.update();
    assert_eq!(availability(&app, id), MetricAvailability::Unavailable("map failed".into()));
}

#[test]
fn stalled_readback_times_out() {
    let (mut app, id) = app();
    enable(&mut app, id);
    for _ in 0..30 {
        app.update(); // 3 s with no result
    }
    assert_eq!(availability(&app, id), MetricAvailability::Unavailable("GPU readback timed out".into()));
}

#[test]
fn stale_epoch_snapshots_are_dropped() {
    let (mut app, id) = app();
    app.world_mut().query::<&mut LeniaSimulation>().single_mut(app.world_mut()).unwrap().epoch = 2;
    enable(&mut app, id);
    for _ in 0..4 {
        app.update();
    }
    *app.world().resource::<Fake>().ready.lock().unwrap() = Some(Ok((1, 9, vec![255; 4])));
    app.update();
    assert!(app.world().resource::<Metrics>().get(id).unwrap().latest.is_none());
}
```

Append inside `gpu_lenia_matches_cpu_for_odd_and_even_tick_batches`, after the existing `assert_eq!(values, expected, …)`:

```rust
        use engine_core::automata::channel::metrics::mass;
        assert_eq!(mass(&values), mass(&expected), "GPU mass equals CPU reference mass");
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p engine-render command_executor`
Expected: `unresolved import super::metrics`.

- [ ] **Step 3: Implement**

`command_executor/metrics.rs`:

```rust
//! Opt-in GPU metric adapter: throttled `LeniaProbe` readbacks reduced to total mass.
//!
//! Ordinary rendering performs no readbacks; sampling starts only after a
//! `SampleMetric(true)` command for that channel.
use bevy::prelude::*;
use engine_core::automata::{
    channel::{ChannelAction, ChannelCommand, ChannelId, metrics::{MetricAvailability, MetricId, MetricSample, Metrics, mass}},
    lenia::LeniaSimulation,
};
use std::collections::HashMap;

/// Lenia mass: Σ density / 255.
pub const MASS: MetricId = MetricId { name: "mass", label: "mass", unit: "density" };
/// Minimum seconds between readback requests (4 Hz).
pub const PERIOD_SECS: f32 = 0.25;
/// Seconds after which an unanswered request is reported as unavailable.
pub const TIMEOUT_SECS: f32 = 2.0;

/// Readback mailbox seam; implemented by `engine_gpu::LeniaProbe` and by test fakes.
pub trait SnapshotMailbox {
    /// Ask for one asynchronous readback of `owner`.
    fn request(&self, owner: Entity);
    /// Finished `(epoch, tick, densities)` or the GPU error, if any.
    fn take(&self, owner: Entity) -> Option<Result<(u64, u64, Vec<u8>), String>>;
}

#[cfg(feature = "gpu-compute")]
impl SnapshotMailbox for engine_gpu::LeniaProbe {
    fn request(&self, owner: Entity) {
        engine_gpu::LeniaProbe::request(self, owner);
    }
    fn take(&self, owner: Entity) -> Option<Result<(u64, u64, Vec<u8>), String>> {
        engine_gpu::LeniaProbe::take(self, owner).map(|r| r.map(|s| (s.epoch, s.tick, s.densities)))
    }
}

#[derive(Clone, Debug, Default)]
struct Cycle {
    elapsed: f32,
    waiting: bool,
    error: Option<String>,
}

/// Channels that opted in, and where each is in its request cycle.
#[derive(Resource, Default)]
pub struct MassSampling(HashMap<ChannelId, Cycle>);

/// Apply `SampleMetric` toggles.
pub fn toggle(mut commands: EventReader<ChannelCommand>, mut sampling: ResMut<MassSampling>) {
    for command in commands.read() {
        match command.action {
            ChannelAction::SampleMetric(true) => {
                sampling.0.entry(command.id).or_insert(Cycle { elapsed: PERIOD_SECS, ..default() });
            }
            ChannelAction::SampleMetric(false) => {
                sampling.0.remove(&command.id);
            }
            _ => {}
        }
    }
}

/// Request, collect, reduce and publish mass for opted-in owners; declare the rest `OptIn`.
pub fn sample<M: SnapshotMailbox + Resource>(
    time: Res<Time>,
    probe: Res<M>,
    owners: Query<(Entity, &ChannelId, &LeniaSimulation)>,
    mut sampling: ResMut<MassSampling>,
    mut metrics: ResMut<Metrics>,
) {
    let dt = time.delta_secs();
    for (entity, id, sim) in &owners {
        let Some(cycle) = sampling.0.get_mut(id) else {
            metrics.declare(*id, MASS, MetricAvailability::OptIn);
            continue;
        };
        cycle.elapsed += dt;
        if cycle.waiting {
            match probe.take(entity) {
                Some(Ok((epoch, tick, densities))) => {
                    cycle.waiting = false;
                    cycle.error = None;
                    if epoch >= sim.epoch {
                        metrics.declare(*id, MASS, MetricAvailability::Sampled { period_secs: PERIOD_SECS });
                        metrics.record(*id, MetricSample { tick, epoch, value: mass(&densities) });
                    }
                }
                Some(Err(error)) => (cycle.waiting, cycle.error) = (false, Some(error)),
                None if cycle.elapsed >= TIMEOUT_SECS => {
                    (cycle.waiting, cycle.error) = (false, Some("GPU readback timed out".into()));
                }
                None => {}
            }
        } else if cycle.elapsed >= PERIOD_SECS {
            probe.request(entity);
            (cycle.waiting, cycle.elapsed) = (true, 0.0);
        }
        let availability = match &cycle.error {
            Some(error) => MetricAvailability::Unavailable(error.clone()),
            None => MetricAvailability::Sampled { period_secs: PERIOD_SECS },
        };
        metrics.declare(*id, MASS, availability);
    }
    sampling.0.retain(|id, _| owners.iter().any(|(_, owner, _)| owner == id));
}
```

After a timeout or error, the next request is issued once `elapsed ≥ PERIOD_SECS` again, so a recovered device resumes. The error stays shown until a sample succeeds.

`throttled` test check: the first request fires on the first update (because the initial `elapsed` is `PERIOD_SECS`). The system then waits, and `TIMEOUT_SECS` (2 s) is not reached within 1 s, so exactly one request is made.

Register in `CommandExecutorPlugin::build`:

```rust
        app.init_resource::<metrics::MassSampling>().add_systems(
            Update,
            (metrics::toggle.in_set(ChannelSet::Route), metrics::sample::<engine_gpu::LeniaProbe>.in_set(ChannelSet::Publish)),
        );
```

Guard the `sample::<engine_gpu::LeniaProbe>` registration with `#[cfg(feature = "gpu-compute")]`. The GPU plugin already inserts `LeniaProbe` (`plugin/mod.rs:31`).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p engine-render command_executor && cargo test -p engine-gpu gpu_lenia_matches_cpu`
Expected: 5 adapter tests pass, and the GPU parity test passes on a Vulkan adapter. On this workstation, run it with `VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json` if no GPU is visible.

- [ ] **Step 5: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy --workspace --all-targets --all-features -- -D warnings
git add lib/engine
git commit -m "feat: add opt-in throttled GPU mass sampling for Lenia channels

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Per-channel colour in the voxel shader

**Files:**
- Create: `lib/engine/engine-gpu/src/plugin/drawing/palette.rs`
- Modify: `lib/engine/engine-gpu/src/plugin/drawing/mod.rs` (`pub(crate) mod palette;`)
- Modify: `lib/engine/engine-gpu/src/plugin/drawing/pipeline.rs` (layout entry 1)
- Modify: `lib/engine/engine-gpu/src/plugin/drawing/voxel_draw_node.rs` (bind palette buffer)
- Modify: `lib/engine/engine-gpu/src/plugin/mod.rs` (render-world resource + extract system)
- Modify: `lib/engine/engine-gpu/assets/voxel_mesh.wgsl`
- Modify: `lib/engine/engine-gpu/src/render/compute/voxel/tests/mod.rs` (size assertion)

**Interfaces:**
- Consumes: Task 1 `ChannelSettings`, `ChannelId`, `Channels`, `ramp`, `Srgb::linear`, `SIGNAL`; existing `GpuGridSlice`, `ExtractedGpuSlices`, `SurfaceParams.world` (atlas offset × 4 world units).
- Produces: `PaletteEntry { rect: [f32; 4], stops: [[f32; 4]; 4] }`, `PaletteUniform { header: [u32; 4], entries: [PaletteEntry; 8] }` (656 bytes), `palette::build(entries: impl IntoIterator<Item = ([f32; 4], Srgb)>) -> PaletteUniform`, `palette::rect(&GpuGridSlice) -> [f32; 4]`, the `VoxelPalette(PaletteUniform)` render resource, and `palette::extract`.

- [ ] **Step 1: Write the failing tests**

In `palette.rs`, start with tests:

```rust
//! Per-channel voxel ramps: up to eight world rectangles, matched in the fragment shader.

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::automata::channel::{SIGNAL, SWATCHES, Srgb, ramp};

    #[test]
    fn layout_is_656_bytes_with_sixteen_byte_entries() {
        assert_eq!(std::mem::size_of::<PaletteEntry>(), 80);
        assert_eq!(std::mem::size_of::<PaletteUniform>(), 656);
    }

    #[test]
    fn build_stores_linear_ramp_stops_and_caps_at_eight() {
        let entries = (0..10).map(|i| ([i as f32, 0.0, i as f32 + 1.0, 1.0], SWATCHES[i % 5]));
        let uniform = build(entries);
        assert_eq!(uniform.header[0], 8);
        let expected = ramp(SWATCHES[0]).map(|s: Srgb| s.linear());
        for (stop, want) in uniform.entries[0].stops.iter().zip(expected) {
            assert_eq!(&stop[..3], &want[..]);
            assert_eq!(stop[3], 1.0);
        }
        assert_eq!(build([]).header[0], 0, "empty palette falls back to shader default");
        let _ = SIGNAL;
    }

    #[test]
    fn rect_matches_surface_world_mapping() {
        let slice = engine_core::automata::GpuGridSlice {
            layer: 2, offset: bevy::math::UVec2::new(64, 0), size: bevy::math::UVec2::splat(64), depth: 8, rule: "lenia".into(), rule_bits: 0,
        };
        assert_eq!(rect(&slice), [256.0, 0.0, 512.0, 256.0]);
    }
}
```

Update `voxel/tests/mod.rs` `active_shaders_validate` by appending `assert_eq!(std::mem::size_of::<crate::plugin::drawing::palette::PaletteUniform>(), 656);`. If the visibility is `pub(crate)`, this works from inside the crate.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p engine-gpu palette`
Expected: `cannot find type PaletteEntry`.

- [ ] **Step 3: Implement the palette types and extraction**

Prepend to `palette.rs`:

```rust
use bevy::{prelude::*, render::Extract};
use bytemuck::{Pod, Zeroable};
use engine_core::automata::{
    GpuGridSlice,
    channel::{ChannelId, ChannelSettings, Channels, Srgb, ramp},
};

/// Maximum distinct channel ramps per frame (the channel capacity).
pub(crate) const MAX_ENTRIES: usize = 8;

/// One instance's world-space XY rectangle and four linear-RGB ramp stops.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub(crate) struct PaletteEntry {
    pub rect: [f32; 4],
    pub stops: [[f32; 4]; 4],
}

/// Uniform block; `header.x` is the entry count, the rest is padding.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct PaletteUniform {
    pub header: [u32; 4],
    pub entries: [PaletteEntry; MAX_ENTRIES],
}

/// Render-world palette for the current frame.
#[derive(Resource, Clone, Copy)]
pub(crate) struct VoxelPalette(pub PaletteUniform);

impl Default for VoxelPalette {
    fn default() -> Self {
        Self(build([]))
    }
}

/// Pack up to eight `(rect, colour)` pairs; earlier entries win where rectangles overlap.
pub(crate) fn build(entries: impl IntoIterator<Item = ([f32; 4], Srgb)>) -> PaletteUniform {
    let mut uniform = PaletteUniform { header: [0; 4], entries: [PaletteEntry::default(); MAX_ENTRIES] };
    for (slot, (rect, colour)) in uniform.entries.iter_mut().zip(entries) {
        slot.rect = rect;
        slot.stops = ramp(colour).map(|stop| {
            let [r, g, b] = stop.linear();
            [r, g, b, 1.0]
        });
        uniform.header[0] += 1;
    }
    uniform
}

/// World XY rectangle covered by a slice, matching `VoxelSurfaceNode`'s 4-unit voxels.
pub(crate) fn rect(slice: &GpuGridSlice) -> [f32; 4] {
    let (x, y) = (slice.offset.x as f32 * 4.0, slice.offset.y as f32 * 4.0);
    [x, y, x + slice.size.x as f32 * 4.0, y + slice.size.y as f32 * 4.0]
}

/// Copy channel colours into the render world, selected channel first.
pub(crate) fn extract(
    owners: Extract<Query<(&ChannelId, &ChannelSettings, &GpuGridSlice)>>,
    channels: Extract<Option<Res<Channels>>>,
    mut palette: ResMut<VoxelPalette>,
) {
    let selected = channels.as_ref().and_then(|c| c.selected);
    let mut entries: Vec<_> = owners.iter().collect();
    entries.sort_by_key(|(id, ..)| (Some(**id) != selected, **id));
    palette.0 = build(entries.into_iter().map(|(_, settings, slice)| (rect(slice), settings.colour)));
}
```

In `plugin/mod.rs`, in the render-app block, add `.init_resource::<drawing::palette::VoxelPalette>()`, and add `drawing::palette::extract` to the `ExtractSchedule` tuple.

- [ ] **Step 4: Bind the palette**

In `pipeline.rs`, add a second layout entry:

```rust
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(
                            std::mem::size_of::<super::palette::PaletteUniform>() as u64,
                        ),
                    },
                    count: None,
                },
```

In `voxel_draw_node.rs`, before `create_bind_group`:

```rust
        let palette = world.resource::<super::palette::VoxelPalette>();
        let palette_buffer = world.resource::<RenderDevice>().create_buffer_with_data(&BufferInitDescriptor {
            label: Some("voxel.palette"),
            contents: bytemuck::bytes_of(&palette.0),
            usage: BufferUsages::UNIFORM,
        });
```

Then add `BindGroupEntry { binding: 1, resource: palette_buffer.as_entire_binding() }` to the entries array.

- [ ] **Step 5: Shade with the ramp**

Replace the `fragment` function of `voxel_mesh.wgsl`, and add the palette declarations above it:

```wgsl
struct PaletteEntry { rect: vec4<f32>, stops: array<vec4<f32>, 4> };
struct Palette { header: vec4<u32>, entries: array<PaletteEntry, 8> };
@group(0) @binding(1) var<uniform> palette: Palette;

// Amber ramp used when no channel rectangle contains the fragment.
fn default_stop(i: u32) -> vec3<f32> {
    switch i {
        case 0u: { return vec3<f32>(0.012, 0.011, 0.008); }
        case 1u: { return vec3<f32>(0.21, 0.13, 0.04); }
        case 2u: { return vec3<f32>(1.0, 0.462, 0.064); }
        default: { return vec3<f32>(1.0, 0.75, 0.46); }
    }
}

fn ramp_at(world: vec2<f32>, density: f32) -> vec3<f32> {
    let t = clamp(density, 0.0, 1.0) * 3.0;
    let i = min(u32(floor(t)), 2u);
    let f = t - f32(i);
    for (var k = 0u; k < palette.header.x; k = k + 1u) {
        let r = palette.entries[k].rect;
        if (world.x >= r.x && world.x <= r.z && world.y >= r.y && world.y <= r.w) {
            return mix(palette.entries[k].stops[i].xyz, palette.entries[k].stops[i + 1u].xyz, f);
        }
    }
    return mix(default_stop(i), default_stop(i + 1u), f);
}

@fragment
fn fragment(in: VSOut) -> @location(0) vec4<f32> {
    let light = 0.35 + 0.65 * max(dot(normalize(in.normal), normalize(vec3<f32>(0.4, 0.8, 0.6))), 0.0);
    let band = 0.8 + 0.2 * sin(in.world_pos.y * 0.12);
    // Quantized densities are low near the surface; lift them so the ramp's colour stop dominates.
    let shade = ramp_at(in.world_pos.xy, 0.35 + 0.65 * in.density);
    return vec4<f32>(shade * light * band, 1.0);
}
```

The default stops are `ramp(SIGNAL)` in linear light. Compute the exact values with a throwaway test print (`println!("{:?}", ramp(SIGNAL).map(Srgb::linear))`) and paste them in, replacing the approximations above.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p engine-gpu`
Expected: the palette tests and `active_shaders_validate` pass (naga validates the new uniform), and existing GPU surface tests still pass. If any GPU test builds `VoxelPipeline`'s bind group by hand, add the binding 1 entry there too, using `build([])`.

- [ ] **Step 7: Visual check**

Run: `make lenia`, then press Resume.
Expected: voxels render in amber. Save a screenshot for the PR: `SOUL_SCREENSHOT=/tmp/soul-lenia-amber.png make preview` produces the static fixture, also amber, since the default ramp applies.

- [ ] **Step 8: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy --workspace --all-targets --all-features -- -D warnings
git add lib/engine/engine-gpu
git commit -m "feat: shade voxels with per-channel Oklab ramps

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: Specimen metadata

**Files:**
- Create: `assets/specimens/specimens.toml`
- Create: `lib/models/src/workbench/specimens/mod.rs`, `lib/models/src/workbench/specimens/tests.rs`
- Modify: `lib/models/src/workbench/mod.rs` (`pub mod specimens;`)
- Modify: `lib/models/src/workbench/plugin.rs` (`app.insert_resource(Specimens::embedded())`)
- Modify: `lib/models/Cargo.toml` (`toml = { workspace = true }`)

**Interfaces:**
- Consumes: `Kind::ALL`, `Kind::id`, Task 5 `metric_for`.
- Produces:
  - `Group::{Cellular, Machines, Grammars, Agents}`, with `Group::ALL` and `Group::label() -> &'static str` (lower-case: "cellular" …).
  - `Specimen { id, title, group, subtitle, caption, explanation, metric, citation, doi }`, with `Specimen::doi_url() -> String`.
  - `Specimens(pub Vec<Specimen>)`, with `parse(&str) -> Result<Self, String>`, `embedded()`, `get(&str) -> Option<&Specimen>` (maps `lenia:*` to `lenia`) and `figure(&str) -> Option<usize>` (1-based).
  - `valid_doi(&str) -> bool`.

- [ ] **Step 1: Write the data file**

`assets/specimens/specimens.toml`. Every DOI was resolved against doi.org (HTTP 302), and its title, authors, venue, volume, pages and year were checked against Crossref on 2026-10-19:

```toml
# Specimen copy shown by the lab: caption under the stage, explanation and
# citation in the inspector footnote. Keep claims within the family contracts.

[[specimen]]
id          = "wolfram:rule30"
title       = "Rule 30"
group       = "cellular"
subtitle    = "elementary, chaotic"
caption     = "Space-time diagram grown from one live cell; time runs downward."
explanation = "Each cell's next state depends on itself and its two neighbours through the update table 00011110."
metric      = "live_cells"
citation    = "S. Wolfram, \"Statistical mechanics of cellular automata\", Reviews of Modern Physics 55(3), 601–644 (1983)."
doi         = "10.1103/RevModPhys.55.601"

[[specimen]]
id          = "wolfram:rule110"
title       = "Rule 110"
group       = "cellular"
subtitle    = "elementary, universal"
caption     = "Space-time diagram grown from one live cell; time runs downward."
explanation = "Update table 01101110. Cook proved that this elementary rule can emulate any computation."
metric      = "live_cells"
citation    = "M. Cook, \"Universality in Elementary Cellular Automata\", Complex Systems 15(1), 1–40 (2004)."
doi         = "10.25088/ComplexSystems.15.1.1"

[[specimen]]
id          = "life:conway"
title       = "Conway's Game of Life"
group       = "cellular"
subtitle    = "B3/S23"
caption     = "A glider crossing a bounded 64 × 64 field with dead edges."
explanation = "A dead cell with exactly three live neighbours is born; a live cell with two or three survives."
metric      = "population"
citation    = "M. Gardner, \"Mathematical Games\", Scientific American 223(4), 120–123 (1970)."
doi         = "10.1038/scientificamerican1070-120"

[[specimen]]
id          = "particle:hpp"
title       = "HPP lattice gas"
group       = "cellular"
subtitle    = "lattice gas, collisions"
caption     = "Two opposing particle walls meet on a 64 × 64 lattice."
explanation = "Particles hop along four lattice directions; head-on pairs scatter at right angles, conserving particle number and momentum."
metric      = "particles"
citation    = "J. Hardy, Y. Pomeau, O. de Pazzis, \"Time evolution of a two-dimensional model system. I.\", Journal of Mathematical Physics 14(12), 1746–1759 (1973)."
doi         = "10.1063/1.1666248"

[[specimen]]
id          = "lenia"
title       = "Lenia"
group       = "cellular"
subtitle    = "continuous, quantized planes"
caption     = "Density evolving on independent XY planes of a 64 × 64 × 8 volume."
explanation = "Density in 0–255 evolves under a finite 3 × 3 Gaussian kernel with zero boundaries; each XY plane is independent. A bounded variant, not isotropic 3-D Lenia."
metric      = "mass"
citation    = "B. W.-C. Chan, \"Lenia: Biology of Artificial Life\", Complex Systems 28(3), 251–286 (2019)."
doi         = "10.25088/ComplexSystems.28.3.251"

[[specimen]]
id          = "regular:nfa"
title       = "Finite automaton"
group       = "machines"
subtitle    = "a*b"
caption     = "Input tape read one symbol per step."
explanation = "Tracks every reachable state at once and accepts any number of a's followed by a single b."
metric      = "steps"
citation    = "M. O. Rabin, D. Scott, \"Finite Automata and Their Decision Problems\", IBM Journal of Research and Development 3(2), 114–125 (1959)."
doi         = "10.1147/rd.32.0114"

[[specimen]]
id          = "context-free:anbn"
title       = "Pushdown automaton"
group       = "machines"
subtitle    = "aⁿbⁿ"
caption     = "Input tape with the stack it builds."
explanation = "Pushes on each a and pops on each b; accepts when the stack empties exactly at the end of the input."
metric      = "steps"
citation    = "N. Chomsky, \"Three models for the description of language\", IRE Transactions on Information Theory 2(3), 113–124 (1956)."
doi         = "10.1109/TIT.1956.1056813"

[[specimen]]
id          = "context-free:parentheses"
title       = "Balanced parentheses"
group       = "machines"
subtitle    = "pushdown"
caption     = "Input tape with the stack it builds."
explanation = "Pushes on each opening parenthesis and pops on each closing one; accepts when every parenthesis is matched."
metric      = "steps"
citation    = "N. Chomsky, \"Three models for the description of language\", IRE Transactions on Information Theory 2(3), 113–124 (1956)."
doi         = "10.1109/TIT.1956.1056813"

[[specimen]]
id          = "context-sensitive:anbncn"
title       = "Linear-bounded automaton"
group       = "machines"
subtitle    = "aⁿbⁿcⁿ"
caption     = "The machine's tape never grows beyond its input."
explanation = "Marks one a, b and c per pass within the original tape; accepts when every symbol is matched."
metric      = "steps"
citation    = "S.-Y. Kuroda, \"Classes of languages and linear-bounded automata\", Information and Control 7(2), 207–223 (1964)."
doi         = "10.1016/S0019-9958(64)90120-2"

[[specimen]]
id          = "turing:replace-a"
title       = "Turing machine"
group       = "machines"
subtitle    = "a → X"
caption     = "Tape and head after each transition."
explanation = "Scans right rewriting each a as X and accepts at the first blank. A small example, not a universal machine."
metric      = "steps"
citation    = "A. M. Turing, \"On Computable Numbers, with an Application to the Entscheidungsproblem\", Proceedings of the London Mathematical Society s2-42(1), 230–265 (1937)."
doi         = "10.1112/plms/s2-42.1.230"

[[specimen]]
id          = "context-free:koch"
title       = "Koch curve"
group       = "grammars"
subtitle    = "L-system F → F+F--F+F"
caption     = "Turtle path of the rewritten word."
explanation = "Every F is rewritten in parallel each step; expansion stops at a 4096-symbol bound."
metric      = "length"
citation    = "A. Lindenmayer, \"Mathematical models for cellular interactions in development I\", Journal of Theoretical Biology 18(3), 280–299 (1968)."
doi         = "10.1016/0022-5193(68)90079-9"

[[specimen]]
id          = "swarm:boids"
title       = "Boids"
group       = "agents"
subtitle    = "flocking"
caption     = "24 boids in a 64 × 64 arena that wraps at its edges."
explanation = "Each boid steers by separation, alignment and cohesion with neighbours inside its radius."
metric      = "polarisation"
citation    = "C. W. Reynolds, \"Flocks, herds and schools: A distributed behavioral model\", ACM SIGGRAPH Computer Graphics 21(4), 25–34 (1987)."
doi         = "10.1145/37402.37406"

[[specimen]]
id          = "swarm:ants"
title       = "Ant foraging"
group       = "agents"
subtitle    = "trail laying"
caption     = "32 ants, two food sites and evaporating pheromone trails."
explanation = "Ants wander from the nest, return food along their trail and reinforce it; unused trails evaporate."
metric      = "food"
citation    = "J.-L. Deneubourg, S. Aron, S. Goss, J. M. Pasteels, \"The self-organizing exploratory pattern of the argentine ant\", Journal of Insect Behavior 3(2), 159–168 (1990)."
doi         = "10.1007/BF01417909"
```

- [ ] **Step 2: Write the failing tests**

`specimens/tests.rs`:

```rust
//! The embedded specimen file covers every family exactly once with valid, honest metadata.
use super::*;
use crate::workbench::{catalog::Kind, channels::metrics::metric_for};

#[test]
fn every_family_and_lenia_has_exactly_one_entry() {
    let specimens = Specimens::embedded();
    let mut expected: Vec<String> = Kind::ALL.iter().map(|k| k.id().to_string()).collect();
    expected.push("lenia".into());
    let mut actual: Vec<String> = specimens.0.iter().map(|s| s.id.clone()).collect();
    expected.sort();
    actual.sort();
    assert_eq!(actual, expected);
}

#[test]
fn fields_are_present_and_dois_valid() {
    for s in Specimens::embedded().0 {
        for (name, value) in [("title", &s.title), ("subtitle", &s.subtitle), ("caption", &s.caption), ("explanation", &s.explanation), ("citation", &s.citation)] {
            assert!(!value.trim().is_empty(), "{} missing {name}", s.id);
        }
        assert!(valid_doi(&s.doi), "{} has malformed DOI {}", s.id, s.doi);
        assert_eq!(s.doi_url(), format!("https://doi.org/{}", s.doi));
    }
}

#[test]
fn metric_names_match_the_metric_sources() {
    let specimens = Specimens::embedded();
    for kind in Kind::ALL {
        assert_eq!(specimens.get(kind.id()).unwrap().metric, metric_for(kind).name, "{kind:?}");
    }
    assert_eq!(specimens.get("lenia").unwrap().metric, "mass");
    assert_eq!(specimens.get("lenia:orbium").unwrap().id, "lenia");
}

#[test]
fn copy_keeps_contract_disclaimers() {
    let specimens = Specimens::embedded();
    assert!(specimens.get("lenia").unwrap().explanation.contains("not isotropic 3-D Lenia"));
    assert!(specimens.get("turing:replace-a").unwrap().explanation.contains("not a universal machine"));
}

#[test]
fn doi_syntax_and_unknown_groups() {
    assert!(valid_doi("10.1145/37402.37406"));
    assert!(!valid_doi("10.12/x"), "registrant needs 4–9 digits");
    assert!(!valid_doi("doi:10.1145/1"));
    assert!(!valid_doi("10.1145/"));
    let bad = "[[specimen]]\nid='x'\ntitle='x'\ngroup='planets'\nsubtitle='x'\ncaption='x'\nexplanation='x'\nmetric='x'\ncitation='x'\ndoi='10.1145/1'\n";
    assert!(Specimens::parse(bad).is_err());
}

#[test]
fn figure_numbers_follow_file_order() {
    let specimens = Specimens::embedded();
    assert_eq!(specimens.figure("wolfram:rule30"), Some(1));
    assert_eq!(specimens.figure("missing"), None);
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p models specimens`
Expected: `cannot find type Specimens`.

- [ ] **Step 4: Implement**

`specimens/mod.rs`:

```rust
//! Authored specimen copy (caption, explanation, citation) embedded from `assets/specimens`.
use bevy::prelude::Resource;
use serde::Deserialize;
#[cfg(test)]
mod tests;

const SOURCE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/specimens/specimens.toml"));

/// Library section a specimen belongs to.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Group {
    /// Grids and continuous fields.
    Cellular,
    /// Symbolic recognisers and Turing machines.
    Machines,
    /// Rewriting systems.
    Grammars,
    /// Multi-agent arenas.
    Agents,
}

impl Group {
    /// Library order.
    pub const ALL: [Self; 4] = [Self::Cellular, Self::Machines, Self::Grammars, Self::Agents];

    /// Margin label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Cellular => "cellular",
            Self::Machines => "machines",
            Self::Grammars => "grammars",
            Self::Agents => "agents",
        }
    }
}

/// One library entry.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Specimen {
    /// Catalog id passed to `AutomataCommand::SeedPattern`.
    pub id: String,
    /// Serif display title.
    pub title: String,
    /// Library section.
    pub group: Group,
    /// Italic library subtitle.
    pub subtitle: String,
    /// Stage figure caption.
    pub caption: String,
    /// One-line footnote explanation.
    pub explanation: String,
    /// Headline metric name (matches the backend's `MetricId::name`).
    pub metric: String,
    /// Full reference.
    pub citation: String,
    /// DOI without the resolver prefix.
    pub doi: String,
}

impl Specimen {
    /// Resolver URL for the DOI.
    pub fn doi_url(&self) -> String {
        format!("https://doi.org/{}", self.doi)
    }
}

#[derive(Deserialize)]
struct File {
    specimen: Vec<Specimen>,
}

/// Every specimen in file order.
#[derive(Resource, Clone, Debug)]
pub struct Specimens(pub Vec<Specimen>);

impl Specimens {
    /// Parse and validate TOML specimen data.
    pub fn parse(source: &str) -> Result<Self, String> {
        let file: File = toml::from_str(source).map_err(|e| e.to_string())?;
        if let Some(bad) = file.specimen.iter().find(|s| !valid_doi(&s.doi)) {
            return Err(format!("{}: malformed DOI {}", bad.id, bad.doi));
        }
        Ok(Self(file.specimen))
    }

    /// The compiled-in file; validity is guaranteed by this module's tests.
    pub fn embedded() -> Self {
        Self::parse(SOURCE).expect("assets/specimens/specimens.toml is validated by tests")
    }

    /// Entry for a catalog id; Lenia seed variants share the `lenia` entry.
    pub fn get(&self, id: &str) -> Option<&Specimen> {
        let id = if id.starts_with("lenia") { "lenia" } else { id };
        self.0.iter().find(|s| s.id == id)
    }

    /// 1-based figure number in file order.
    pub fn figure(&self, id: &str) -> Option<usize> {
        let id = self.get(id)?.id.as_str();
        self.0.iter().position(|s| s.id == id).map(|i| i + 1)
    }
}

/// `10.<4–9 digit registrant>/<non-empty suffix without whitespace>`.
pub fn valid_doi(doi: &str) -> bool {
    let Some((registrant, suffix)) = doi.strip_prefix("10.").and_then(|rest| rest.split_once('/')) else {
        return false;
    };
    (4..=9).contains(&registrant.len())
        && registrant.chars().all(|c| c.is_ascii_digit())
        && !suffix.is_empty()
        && !suffix.chars().any(char::is_whitespace)
}
```

In `WorkbenchPlugin::build`, add `.insert_resource(super::specimens::Specimens::embedded())`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p models specimens`
Expected: 6 tests pass.

- [ ] **Step 6: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy -p models --all-targets -- -D warnings
git add assets/specimens lib/models
git commit -m "feat: embed verified specimen captions explanations and citations

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---
### Task 10: Theme tokens, Libertinus fonts and borderless visuals

**Files:**
- Create: `assets/fonts/libertinus/{LibertinusSerif-Regular,LibertinusSerif-Italic,LibertinusSerif-Semibold,LibertinusSans-Regular,LibertinusSans-Bold,LibertinusMono-Regular}.otf`, `assets/fonts/libertinus/OFL.txt`
- Create: `lib/ui/src/theme/{mod.rs,tokens.rs,fonts.rs,visuals.rs}`
- Move: `lib/ui/src/styles/fade.rs` → `lib/ui/src/theme/fade/mod.rs`
- Delete: `lib/ui/src/styles/` (`mod.rs`, `plugin.rs`)
- Modify: `lib/ui/src/lib.rs` (`pub mod theme;` replaces `pub mod styles;`), `lib/ui/src/plugin.rs` (`ThemePlugin` replaces `StylesPlugin`), and every `styles::fullscreen_bg()` call site (`main_menu`, `scenarios_menu/{new,load}`, `options_menu`, `pause_menu`)

**Interfaces:**
- Produces:
  - `theme::tokens::{BG, PANEL, RAISED, TRACK, INK, INK_HI, INK_DIM, SIGNAL, SIGNAL_BG, OK, FAULT, INFO}: Color32`, `theme::tokens::color(Srgb) -> Color32`, `theme::tokens::contrast(Color32, Color32) -> f32`, and `theme::tokens::size::{LABEL, BODY, TITLE, READOUT, HUD}: f32`.
  - `theme::fonts::{SERIF, SERIF_ITALIC, SERIF_SEMIBOLD, SANS_BOLD}: &str` family names, `theme::fonts::family(&str) -> FontFamily` and `theme::fonts::definitions() -> FontDefinitions`.
  - `theme::visuals::{apply(&mut Style), install(&Context)}` and `theme::{fullscreen_bg, panel_bg, BOTTOM_PAD}`.
  - `theme::ThemePlugin`.

- [ ] **Step 1: Bundle the fonts**

```bash
S=$(mktemp -d) && curl -sL -o "$S/L.zip" https://github.com/alerque/libertinus/releases/download/v7.051/Libertinus-7.051.zip
echo "4d9be29b5cb380c35af8ba967abcc752ad1e07be1f738a9789c33e0dd7478c92  $S/L.zip" | sha256sum -c -
mkdir -p assets/fonts/libertinus
for f in LibertinusSerif-Regular LibertinusSerif-Italic LibertinusSerif-Semibold LibertinusSans-Regular LibertinusSans-Bold LibertinusMono-Regular; do
  unzip -p "$S/L.zip" "Libertinus-7.051/static/OTF/$f.otf" > "assets/fonts/libertinus/$f.otf"
done
unzip -p "$S/L.zip" Libertinus-7.051/OFL.txt > assets/fonts/libertinus/OFL.txt
ls -la assets/fonts/libertinus
```

Expected: `OK` from sha256sum, then six `.otf` files of 59–338 KB plus `OFL.txt`.

- [ ] **Step 2: Write the failing tests**

`theme/tokens.rs` (tests only first):

```rust
//! Instrument palette and type scale; the only colour constants the UI uses.

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
```

`theme/fonts.rs` (tests only first):

```rust
//! Libertinus Serif / Sans / Mono, embedded and registered as named egui families.

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
            assert!(chain.len() > 1, "{name} keeps egui fallbacks for missing symbols");
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
                assert!(fonts.has_glyphs(&FontId::new(12.0, FontFamily::Proportional), "❚↺"), "fallback");
            });
        });
    }
}
```

`theme/visuals.rs` (tests only first):

```rust
//! Global egui style: borderless surfaces, Instrument colours, Libertinus type scale.

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_egui::egui::Style;

    #[test]
    fn no_widget_or_window_draws_a_hairline() {
        let mut style = Style::default();
        apply(&mut style);
        let w = &style.visuals.widgets;
        for state in [&w.noninteractive, &w.inactive, &w.hovered, &w.active, &w.open] {
            assert_eq!(state.bg_stroke.width, 0.0);
        }
        assert_eq!(style.visuals.window_stroke.width, 0.0);
        assert_eq!(style.visuals.panel_fill, PANEL);
        assert_eq!(style.visuals.override_text_color, Some(INK));
    }

    #[test]
    fn type_scale_uses_spec_sizes() {
        let mut style = Style::default();
        apply(&mut style);
        assert_eq!(style.text_styles[&TextStyle::Body].size, size::BODY);
        assert_eq!(style.text_styles[&TextStyle::Heading].family, family(SERIF));
        assert_eq!(style.text_styles[&TextStyle::Small].size, size::LABEL);
    }
}
```

Create `theme/mod.rs`:

```rust
//! Instrument visual language: tokens, Libertinus fonts, borderless egui style, widgets.
pub mod fade;
pub mod fonts;
pub mod tokens;
pub mod visuals;
pub use visuals::{BOTTOM_PAD, fullscreen_bg, panel_bg};
```

Move fade: `mkdir -p lib/ui/src/theme/fade && git mv lib/ui/src/styles/fade.rs lib/ui/src/theme/fade/mod.rs && git rm -r lib/ui/src/styles`.

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p ui theme`
Expected: unresolved names such as `BG`, `definitions` and `apply`.

- [ ] **Step 4: Implement tokens**

Prepend to `tokens.rs`:

```rust
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
```

- [ ] **Step 5: Implement fonts**

Prepend to `fonts.rs`:

```rust
use bevy_egui::egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

macro_rules! font {
    ($file:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fonts/libertinus/", $file))
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
    ("libertinus-serif-italic", font!("LibertinusSerif-Italic.otf")),
    ("libertinus-serif-semibold", font!("LibertinusSerif-Semibold.otf")),
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
        fonts.font_data.insert(key.into(), Arc::new(FontData::from_static(bytes)));
    }
    let fallback = fonts.families[&FontFamily::Proportional].clone();
    let mono_fallback = fonts.families[&FontFamily::Monospace].clone();
    let chain = |first: &str, rest: &[String]| {
        std::iter::once(first.to_string()).chain(rest.iter().cloned()).collect::<Vec<_>>()
    };
    fonts.families.insert(FontFamily::Proportional, chain("libertinus-sans", &fallback));
    fonts.families.insert(FontFamily::Monospace, chain("libertinus-mono", &mono_fallback));
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
```

If egui 0.31's `font_data` map stores `FontData` rather than `Arc<FontData>`, drop the `Arc::new`. Check with `grep -n "pub font_data" ~/.cargo/registry/src/*/epaint-0.31*/src/text/fonts.rs`.

- [ ] **Step 6: Implement visuals and the plugin**

Prepend to `visuals.rs`:

```rust
use super::{fonts::{SERIF, definitions, family}, tokens::*};
use bevy_egui::egui::{self, CornerRadius, FontFamily, FontId, Shadow, Stroke, Style, TextStyle, Visuals, vec2};

/// Legacy menu bottom padding.
pub const BOTTOM_PAD: f32 = 20.0;

/// Full-screen menu background.
pub fn fullscreen_bg() -> egui::Frame {
    egui::Frame::new().fill(BG)
}

/// Side panel background for legacy menus.
pub fn panel_bg() -> egui::Frame {
    egui::Frame::new().fill(PANEL)
}

/// Apply the Instrument style to `style`.
pub fn apply(style: &mut Style) {
    style.text_styles = [
        (TextStyle::Small, FontId::new(size::LABEL, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(size::BODY, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(size::BODY, FontFamily::Proportional)),
        (TextStyle::Heading, FontId::new(size::TITLE, family(SERIF))),
        (TextStyle::Monospace, FontId::new(size::BODY, FontFamily::Monospace)),
    ]
    .into();
    let mut v = Visuals::dark();
    v.panel_fill = PANEL;
    v.window_fill = PANEL;
    v.extreme_bg_color = BG;
    v.faint_bg_color = RAISED;
    v.window_stroke = Stroke::NONE;
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.override_text_color = Some(INK);
    v.hyperlink_color = SIGNAL;
    v.selection.bg_fill = SIGNAL_BG;
    v.selection.stroke = Stroke::new(1.0, SIGNAL);
    let w = &mut v.widgets;
    for state in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
        state.bg_stroke = Stroke::NONE;
        state.corner_radius = CornerRadius::same(4);
    }
    w.noninteractive.fg_stroke = Stroke::new(1.0, INK);
    (w.inactive.weak_bg_fill, w.inactive.bg_fill) = (RAISED, TRACK);
    (w.hovered.weak_bg_fill, w.hovered.bg_fill) = (TRACK, TRACK);
    (w.active.weak_bg_fill, w.active.bg_fill) = (SIGNAL_BG, SIGNAL);
    w.active.fg_stroke = Stroke::new(1.0, SIGNAL);
    w.hovered.fg_stroke = Stroke::new(1.0, INK_HI);
    style.visuals = v;
    style.spacing.item_spacing = vec2(8.0, 6.0);
    style.spacing.button_padding = vec2(10.0, 4.0);
}

/// Install fonts and style on a context once.
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(definitions());
    ctx.style_mut(apply);
}
```

Append to `theme/mod.rs`:

```rust
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

/// Installs the theme into the primary egui context on its first pass, before any UI draws.
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(fade::FadePlugin)
            .add_systems(EguiPrimaryContextPass, install_once.in_set(ThemeSet));
    }
}

/// Ordering label so the lab draws after the theme is installed.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ThemeSet;

fn install_once(mut contexts: EguiContexts, mut done: Local<bool>) {
    if *done {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut() {
        visuals::install(ctx);
        *done = true;
    }
}
```

In `lib/ui/src/plugin.rs`, replace `styles::plugin::StylesPlugin` with `theme::ThemePlugin`. Then run `grep -rln "styles::" lib/ui/src | xargs sed -i 's/crate::styles/crate::theme/g; s/\bstyles::/theme::/g'`, and fix any `use crate::{styles, …}` imports by hand.

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test -p ui theme && cargo build`
Expected: 6 theme tests pass, and the app builds.

- [ ] **Step 8: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy --workspace --all-targets --all-features -- -D warnings
git add assets/fonts lib/ui
git commit -m "feat: install Instrument tokens, Libertinus fonts and borderless egui style

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 11: S4 Marginalia widgets

**Files:**
- Create: `lib/ui/src/theme/widgets/{mod.rs,margin.rs,readout.rs,controls.rs}`
- Modify: `lib/ui/src/theme/mod.rs` (`pub mod widgets;`)

The `theme/` directory then holds 2 child directories (`fade`, `widgets`) and 4 files, within the limits.

**Interfaces:**
- Consumes: Task 10 tokens and fonts; Task 1 `Status`.
- Produces (all `pub` in `theme::widgets`):
  - `serif(text) -> RichText`, `serif_italic(text) -> RichText`, `mono(text) -> RichText`;
  - `margin_section(ui, label: &str, active: bool, body: impl FnOnce(&mut Ui))`, with `MARGIN_WIDTH: f32 = 56.0`;
  - `readout(ui, value: &str, unit: &str, big: bool)` and `tick_digits(u64) -> String`;
  - `footnote(ui, number: u32, explanation: &str, citation: &str, doi: &str) -> Option<String>`, which returns the DOI URL when its link is clicked;
  - `note(ui, text: &str, warn: bool)`;
  - `pill(ui, label: &str, primary: bool, enabled: bool) -> Response` and `pill_fill(primary, enabled) -> Color32`;
  - `lamp(ui, Status)` and `lamp_colour(Status) -> Color32`;
  - `slider(ui, label: &str, value: &mut f32, range: RangeInclusive<f32>, log: bool) -> bool` (returns changed);
  - `choice<T: PartialEq + Copy>(ui, current: &mut T, options: &[(T, &str)]) -> bool`.

- [ ] **Step 1: Write the failing tests**

`widgets/mod.rs`:

```rust
//! S4 Marginalia vocabulary: margin sections, readouts, footnotes, pills, lamps, sliders, choices.
pub mod controls;
pub mod margin;
pub mod readout;
pub use controls::{choice, lamp, lamp_colour, pill, pill_fill, slider};
pub use margin::{MARGIN_WIDTH, margin_section, note};
pub use readout::{footnote, mono, readout, serif, serif_italic, tick_digits};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::tokens::*;
    use bevy_egui::egui::{CentralPanel, Context, RawInput};
    use engine_core::automata::channel::Status;

    #[test]
    fn colour_rules() {
        assert_eq!(lamp_colour(Status::Running), OK);
        assert_eq!(lamp_colour(Status::Paused), SIGNAL);
        assert_eq!(lamp_colour(Status::Halted), FAULT);
        assert_eq!(pill_fill(true, true), SIGNAL);
        assert_eq!(pill_fill(false, true), RAISED);
        assert_eq!(pill_fill(true, false), RAISED);
    }

    #[test]
    fn ticks_are_zero_padded_to_six_digits() {
        assert_eq!(tick_digits(184), "000184");
        assert_eq!(tick_digits(12_345_678), "12345678");
    }

    #[test]
    fn widgets_render_headless_without_panicking() {
        let ctx = Context::default();
        crate::theme::visuals::install(&ctx);
        let output = ctx.run(RawInput::default(), |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                margin_section(ui, "readout", true, |ui| readout(ui, "000184", "tick", true));
                let mut v = 0.27;
                slider(ui, "μ", &mut v, 0.0..=1.0, false);
                let mut seed = 0;
                choice(ui, &mut seed, &[(0, "blob"), (1, "ring")]);
                lamp(ui, Status::Running);
                pill(ui, "Pause", true, true);
                assert_eq!(footnote(ui, 1, "Born with three.", "Gardner (1970).", "10.1038/x"), None);
            });
        });
        assert!(!output.shapes.is_empty());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p ui widgets`
Expected: unresolved imports (`margin_section`, …).

- [ ] **Step 3: Implement**

`widgets/readout.rs`:

```rust
//! Typographic primitives: serif/mono text, readouts and citation footnotes.
use crate::theme::{fonts::{SERIF, SERIF_ITALIC, family}, tokens::*};
use bevy_egui::egui::{FontFamily, RichText, Ui};

/// Upright Libertinus Serif.
pub fn serif(text: impl Into<String>) -> RichText {
    RichText::new(text).family(family(SERIF))
}

/// True-italic Libertinus Serif in the dim ink.
pub fn serif_italic(text: impl Into<String>) -> RichText {
    RichText::new(text).family(family(SERIF_ITALIC)).color(INK_DIM)
}

/// Libertinus Mono in the signal colour.
pub fn mono(text: impl Into<String>) -> RichText {
    RichText::new(text).family(FontFamily::Monospace).color(SIGNAL)
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
pub fn footnote(ui: &mut Ui, number: u32, explanation: &str, citation: &str, doi: &str) -> Option<String> {
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
```

`widgets/margin.rs`:

```rust
//! Tufte-style margin sections: italic label column, content hanging beside it.
use super::readout::serif_italic;
use crate::theme::tokens::*;
use bevy_egui::egui::{Align, Layout, Ui, vec2};

/// Width of the right-aligned margin label column.
pub const MARGIN_WIDTH: f32 = 56.0;

/// Draw `label` in the margin (amber when `active`) and `body` beside it.
pub fn margin_section(ui: &mut Ui, label: &str, active: bool, body: impl FnOnce(&mut Ui)) {
    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(vec2(MARGIN_WIDTH, 14.0), Layout::right_to_left(Align::Min), |ui| {
            ui.label(serif_italic(label).color(if active { SIGNAL } else { INK_DIM }));
        });
        ui.vertical(body);
    });
    ui.add_space(10.0);
}

/// Small italic note; `warn` uses the signal colour.
pub fn note(ui: &mut Ui, text: &str, warn: bool) {
    ui.label(serif_italic(text).size(size::BODY - 1.0).color(if warn { SIGNAL } else { INK_DIM }));
}
```

`widgets/controls.rs`:

```rust
//! Pills, status lamps, thin sliders and serif radio choices.
use super::readout::{mono, serif};
use crate::theme::{fonts::{SANS_BOLD, family}, tokens::*};
use bevy_egui::egui::{self, Color32, CornerRadius, Response, RichText, Sense, Ui, vec2};
use engine_core::automata::channel::Status;
use std::ops::RangeInclusive;

/// Pill fill: amber only for an enabled primary action.
pub fn pill_fill(primary: bool, enabled: bool) -> Color32 {
    if primary && enabled { SIGNAL } else { RAISED }
}

/// Rounded, borderless action button.
pub fn pill(ui: &mut Ui, label: &str, primary: bool, enabled: bool) -> Response {
    let text = if primary && enabled {
        RichText::new(label).family(family(SANS_BOLD)).color(BG)
    } else {
        RichText::new(label).color(if enabled { INK } else { TRACK })
    };
    let button = egui::Button::new(text).fill(pill_fill(primary, enabled)).corner_radius(CornerRadius::same(10));
    ui.add_enabled(enabled, button)
}

/// Lamp colour per lifecycle state.
pub fn lamp_colour(status: Status) -> Color32 {
    match status {
        Status::Running => OK,
        Status::Paused => SIGNAL,
        Status::Halted => FAULT,
    }
}

/// 7 px status lamp; running lamps carry a soft glow.
pub fn lamp(ui: &mut Ui, status: Status) {
    let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
    let colour = lamp_colour(status);
    if status == Status::Running {
        ui.painter().circle_filled(rect.center(), 5.0, colour.gamma_multiply(0.25));
    }
    ui.painter().circle_filled(rect.center(), 3.5, colour);
}

/// Labelled thin slider with a mono value; returns whether the value changed.
pub fn slider(ui: &mut Ui, label: &str, value: &mut f32, range: RangeInclusive<f32>, log: bool) -> bool {
    ui.horizontal(|ui| {
        ui.label(super::readout::serif_italic(label));
        let changed = ui
            .add(
                egui::Slider::new(value, range)
                    .show_value(false)
                    .trailing_fill(true)
                    .logarithmic(log)
                    .handle_shape(egui::style::HandleShape::Circle),
            )
            .changed();
        ui.label(mono(format!("{value:.3}")));
        changed
    })
    .inner
}

/// Serif radio row (`●` selected, `○` not); returns whether the choice changed.
pub fn choice<T: PartialEq + Copy>(ui: &mut Ui, current: &mut T, options: &[(T, &str)]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        for (value, label) in options {
            let on = *current == *value;
            let text = serif(format!("{} {label}", if on { "●" } else { "○" })).color(if on { SIGNAL } else { INK_DIM });
            if ui.add(egui::Label::new(text).sense(Sense::click())).clicked() && !on {
                *current = *value;
                changed = true;
            }
        }
    });
    changed
}
```

If `Button::corner_radius` does not exist in egui 0.31, use `.rounding(10.0)`. Check with `grep -n "pub fn corner_radius\|pub fn rounding" ~/.cargo/registry/src/*/egui-0.31*/src/widgets/button.rs`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p ui widgets`
Expected: 3 tests pass.

- [ ] **Step 5: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy -p ui --all-targets -- -D warnings
git add lib/ui/src/theme
git commit -m "feat: add S4 marginalia widgets for sections readouts footnotes and controls

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 12: Stage rectangle, stage cameras and input gating

**Files:**
- Create: `lib/engine/engine-common/src/controls/camera/manager/stage.rs`
- Modify: `lib/engine/engine-common/src/controls/camera/manager/mod.rs` (`pub mod stage;`; add `StageCamera` to the WorldCamera spawn bundle)
- Modify: `lib/engine/engine-common/src/controls/camera/manager/plugin.rs` (register `apply_stage`, `frame_selected`; init `StageRect`)
- Modify: `lib/engine/engine-common/src/controls/camera/input/plugin.rs` (gate pointer systems with `stage_pointer_allowed` and keyboard systems with `stage_keys_allowed`)
- Modify: `src/app/preview/scene.rs` (`StageCamera` on the preview camera; remove it from `WorldCamera` in `isolate_preview_camera`)

**Interfaces:**
- Consumes: Task 1 `Channels`, `ChannelId`, `Backend`; existing `GpuGridSlice`, `WorldCamera`.
- Produces:
  - `StageRect { managed: bool, min: Vec2, size: Vec2, scale: f32, hovered: bool, typing: bool, world_visible: bool }`, a `Resource + Default`;
  - `StageCamera`, a marker `Component`;
  - `physical_viewport(&StageRect, window_physical: UVec2) -> Option<Viewport>`;
  - the systems `apply_stage` and `frame_selected`;
  - the run conditions `stage_pointer_allowed(Option<Res<StageRect>>) -> bool` and `stage_keys_allowed(Option<Res<StageRect>>) -> bool`.

- [ ] **Step 1: Write the failing tests**

`stage.rs`, tests first:

```rust
//! The lab's stage rectangle drives every stage camera's viewport, activity and input.

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(min: Vec2, size: Vec2, scale: f32) -> StageRect {
        StageRect { managed: true, min, size, scale, world_visible: true, ..default() }
    }

    #[test]
    fn logical_rect_maps_to_physical_pixels() {
        let v = physical_viewport(&rect(Vec2::new(32.0, 28.0), Vec2::new(988.0, 652.0), 2.0), UVec2::new(2560, 1440)).unwrap();
        assert_eq!(v.physical_position, UVec2::new(64, 56));
        assert_eq!(v.physical_size, UVec2::new(1976, 1304));
    }

    #[test]
    fn viewport_is_clamped_inside_the_window() {
        let v = physical_viewport(&rect(Vec2::new(1200.0, 0.0), Vec2::new(500.0, 900.0), 1.0), UVec2::new(1280, 720)).unwrap();
        assert_eq!(v.physical_size, UVec2::new(80, 720));
    }

    #[test]
    fn degenerate_stage_has_no_viewport() {
        assert!(physical_viewport(&rect(Vec2::ZERO, Vec2::new(0.4, 300.0), 1.0), UVec2::new(1280, 720)).is_none());
        assert!(physical_viewport(&rect(Vec2::new(1400.0, 0.0), Vec2::splat(100.0), 1.0), UVec2::new(1280, 720)).is_none());
        assert!(physical_viewport(&rect(Vec2::ZERO, Vec2::splat(100.0), 1.0), UVec2::ZERO).is_none());
    }

    #[test]
    fn input_gates_default_open_without_a_lab() {
        assert!(stage_pointer_allowed(None));
        assert!(stage_keys_allowed(None));
    }

    #[test]
    fn frame_selected_centres_on_the_slice_and_ignores_missing_slice() {
        use engine_core::automata::{GpuGridSlice, channel::{Backend, ChannelId, ChannelRow, ChannelSettings, Channels, Rate, Status}};
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).init_resource::<Channels>().add_systems(Update, frame_selected);
        let camera = app.world_mut().spawn((StageCamera, Transform::from_xyz(0.0, 0.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y))).id();
        let id = ChannelId(3);
        app.world_mut().spawn((id, GpuGridSlice { layer: 0, offset: UVec2::new(64, 0), size: UVec2::splat(64), depth: 8, rule: "lenia".into(), rule_bits: 0 }));
        let row = ChannelRow { id, kind: "lenia".into(), backend: Backend::Gpu, status: Status::Paused, tick: 0, epoch: 0, settings: ChannelSettings::new(Rate::Hz60) };
        let mut channels = app.world_mut().resource_mut::<Channels>();
        channels.rows.push(row);
        channels.selected = Some(id);
        app.update();
        let t = app.world().get::<Transform>(camera).unwrap().translation;
        assert!((t.x - 384.0).abs() < 1e-3 && (t.y - 128.0).abs() < 1e-3, "{t:?}");
        let mut channels = app.world_mut().resource_mut::<Channels>();
        channels.selected = Some(ChannelId(99));
        app.update();
        assert_eq!(app.world().get::<Transform>(camera).unwrap().translation, t, "missing slice leaves camera alone");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p engine-common stage`
Expected: unresolved `StageRect`, `physical_viewport` and related names.

- [ ] **Step 3: Implement**

Prepend to `stage.rs`:

```rust
use bevy::{prelude::*, render::camera::Viewport, window::PrimaryWindow};
use engine_core::automata::{GpuGridSlice, channel::{ChannelId, Channels}};

/// Stage area in logical egui points, published by the lab shell every frame.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct StageRect {
    /// False until a lab shell publishes; cameras are left untouched while false.
    pub managed: bool,
    /// Top-left corner in logical points.
    pub min: Vec2,
    /// Size in logical points.
    pub size: Vec2,
    /// Physical pixels per logical point (`egui::Context::pixels_per_point`).
    pub scale: f32,
    /// Pointer is over the stage and not over a panel, drawer or popup.
    pub hovered: bool,
    /// A text field has keyboard focus.
    pub typing: bool,
    /// The selected channel renders in the 3-D world (GPU); false deactivates stage cameras.
    pub world_visible: bool,
}

/// Marks 3-D cameras that render into the stage.
#[derive(Component, Default, Debug)]
pub struct StageCamera;

/// Distance kept from a framed instance by perspective stage cameras.
const FRAME_DISTANCE: f32 = 260.0;

/// Physical viewport for the stage, clamped into the window; `None` when degenerate.
pub fn physical_viewport(rect: &StageRect, window: UVec2) -> Option<Viewport> {
    let min = (rect.min * rect.scale).round().max(Vec2::ZERO);
    let max = ((rect.min + rect.size) * rect.scale).round().min(window.as_vec2());
    let size = (max - min).max(Vec2::ZERO);
    if size.x < 1.0 || size.y < 1.0 {
        return None;
    }
    Some(Viewport { physical_position: min.as_uvec2(), physical_size: size.as_uvec2(), ..default() })
}

/// Fit stage cameras to the stage; deactivate them while a CPU channel is shown.
pub fn apply_stage(
    rect: Res<StageRect>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut cameras: Query<&mut Camera, With<StageCamera>>,
) {
    let Some(window) = window else { return };
    if !rect.managed {
        return;
    }
    let viewport = physical_viewport(&rect, window.physical_size());
    for mut camera in &mut cameras {
        camera.is_active = rect.world_visible && viewport.is_some();
        camera.viewport = viewport.clone();
    }
}

/// When the selection changes to a GPU channel, look at its volume's centre.
pub fn frame_selected(
    channels: Res<Channels>,
    slices: Query<(&ChannelId, &GpuGridSlice)>,
    mut cameras: Query<&mut Transform, With<StageCamera>>,
    mut framed: Local<Option<ChannelId>>,
) {
    let selected = channels.selected;
    if selected == *framed {
        return;
    }
    let Some((_, slice)) = slices.iter().find(|(id, _)| Some(**id) == selected) else {
        return;
    };
    *framed = selected;
    let size = Vec3::new(slice.size.x as f32, slice.size.y as f32, slice.depth.max(1) as f32) * 4.0;
    let centre = Vec3::new(slice.offset.x as f32 * 4.0, slice.offset.y as f32 * 4.0, 0.0) + size * 0.5;
    for mut transform in &mut cameras {
        let forward = transform.forward();
        transform.translation = centre - forward * FRAME_DISTANCE;
    }
}

/// Pointer-driven camera input only while the pointer is over the stage.
pub fn stage_pointer_allowed(rect: Option<Res<StageRect>>) -> bool {
    rect.is_none_or(|r| !r.managed || r.hovered)
}

/// Keyboard camera input only while no text field is focused.
pub fn stage_keys_allowed(rect: Option<Res<StageRect>>) -> bool {
    rect.is_none_or(|r| !r.managed || !r.typing)
}
```

The test expects `translation.x == 384`. That is the centre's x, because the camera looks straight down −Z, so `forward = −Z` and only `z` changes.

Register in `manager/plugin.rs`: `.init_resource::<stage::StageRect>().add_systems(Update, (stage::frame_selected, stage::apply_stage).chain().in_set(engine_core::automata::channel::ChannelSet::Settle))`. Add `StageCamera` to the `WorldCamera` bundle in `spawn_cameras`.

In `input/plugin.rs`, append `.run_if(stage_pointer_allowed)` to the mouse-driven systems (orbit, zoom, drag-pan) and `.run_if(stage_keys_allowed)` to the keyboard-driven ones (WASD/arrow pan, Z/X zoom). Open the file to see which system reads which input; each `gather_*` system name says it.

In `src/app/preview/scene.rs`, add `engine_common::controls::camera::manager::stage::StageCamera` to the preview `Camera3d` bundle. Change `isolate_preview_camera` to:

```rust
pub(super) fn isolate_preview_camera(
    mut commands: Commands,
    mut cameras: Query<(Entity, &mut Camera), With<WorldCamera>>,
) {
    for (entity, mut camera) in &mut cameras {
        camera.is_active = false;
        commands.entity(entity).remove::<StageCamera>();
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p engine-common stage && cargo build`
Expected: 5 tests pass, and the app builds.

- [ ] **Step 5: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy --workspace --all-targets --all-features -- -D warnings
git add lib/engine/engine-common src/app/preview/scene.rs
git commit -m "feat: render stage cameras into the published stage rectangle

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 13: Lab state, shortcuts, chrome and library drawer

**Files:**
- Create: `lib/ui/src/lab/{mod.rs,state.rs,shortcuts.rs,library.rs}`
- Create: `lib/ui/src/lab/chrome/{mod.rs,rail.rs,tabs.rs,transport.rs}`
- Modify: `lib/ui/src/lib.rs` (`pub mod lab;`)

The lab is not registered in `OutputPlugin` until Task 16, so the old UI keeps working until then.

**Interfaces:**
- Consumes: Tasks 1, 2, 9, 10, 11 and 12. Also `Catalog`, `RuleRegistry`, `Workbench` (read), `LeniaSimulation` (read) and `FreeCam`.
- Produces:
  - `lab::state::{LabState, LabMode}`. `LabState` holds `mode`, `drawer_open`, `settings_open`, `inspector_open`, `inspector_width`, `filter: String`, `idle_secs`, `inputs: HashMap<ChannelId, String>`, `views: HashMap<ChannelId, StageView>` (the `StageView` type is added in Task 15) and `toast: Option<(String, f32)>`, plus the methods `clamp_width(f32) -> f32`, `note_pointer(moved: bool, dt: f32)` and `transport_alpha() -> f32`.
  - `lab::Effects { commands: Vec<ChannelCommand>, automata: Vec<AutomataCommand>, open_url: Option<String>, mark_centre: bool }`.
  - `lab::shortcuts::{Shortcut, map(egui::Key) -> Option<Shortcut>, collect(keys: &[egui::Key], typing: bool) -> Vec<Shortcut>, apply(Shortcut, &mut LabState, &Channels, freecam: bool, effects: &mut Effects) -> bool}`. `apply` returns whether the lab consumed the key.
  - `lab::library::{entries(&Specimens, available: &HashSet<String>, filter: &str) -> Vec<(Group, Vec<&Specimen>)>, show(ctx, …)}`.
  - `lab::chrome::{rail::show, tabs::show, transport::show, transport::sparkline(&Trace, Rect) -> Vec<Pos2>}`.
  - `lab::LabData`: a `SystemParam` bundling the read-only resources.
  - `lab::swallow_escape` (a `PreUpdate` system) and `lab::LabPlugin` (registered in Task 16).

- [ ] **Step 1: Write the failing tests**

`lab/shortcuts.rs`, tests first:

```rust
//! Lab keyboard map; camera bindings (arrows, WASD, Z/X, freecam Space) take precedence.

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_egui::egui::Key;
    use engine_core::automata::channel::{Backend, ChannelId, ChannelRow, ChannelSettings, Rate, Status};

    fn channels(status: Status) -> Channels {
        let row = |id| ChannelRow { id: ChannelId(id), kind: "life:conway".into(), backend: Backend::Cpu, status, tick: 0, epoch: 0, settings: ChannelSettings::new(Rate::Hz10) };
        Channels { rows: vec![row(0), row(4)], selected: Some(ChannelId(0)) }
    }

    #[test]
    fn keys_map_to_shortcuts() {
        assert_eq!(map(Key::Space), Some(Shortcut::RunPause));
        assert_eq!(map(Key::Period), Some(Shortcut::Step));
        assert_eq!(map(Key::Num2), Some(Shortcut::Select(1)));
        assert_eq!(map(Key::ArrowRight), None, "arrows belong to the camera");
        assert_eq!(collect(&[Key::R, Key::P], true), [], "typing suppresses shortcuts");
        assert_eq!(collect(&[Key::R], false), [Shortcut::Reset]);
    }

    #[test]
    fn run_pause_step_reset_target_the_selected_channel() {
        let mut lab = LabState::default();
        let mut fx = Effects::default();
        assert!(apply(Shortcut::RunPause, &mut lab, &channels(Status::Paused), false, &mut fx));
        assert!(matches!(fx.commands[0].action, ChannelAction::Pause(false)));
        assert!(apply(Shortcut::Step, &mut lab, &channels(Status::Paused), false, &mut fx));
        assert!(!apply(Shortcut::Step, &mut lab, &channels(Status::Running), false, &mut fx), "no step while running");
        assert!(!apply(Shortcut::RunPause, &mut lab, &channels(Status::Halted), false, &mut fx));
        assert!(!apply(Shortcut::RunPause, &mut lab, &channels(Status::Paused), true, &mut fx), "freecam owns Space");
        assert!(apply(Shortcut::Select(1), &mut lab, &channels(Status::Paused), false, &mut fx));
        assert_eq!(fx.commands.last().unwrap().id, ChannelId(4));
        assert!(!apply(Shortcut::Select(5), &mut lab, &channels(Status::Paused), false, &mut fx));
    }

    #[test]
    fn escape_is_swallowed_only_when_lab_consumes_it() {
        let mut lab = LabState { drawer_open: true, ..default() };
        let mut fx = Effects::default();
        let c = channels(Status::Paused);
        assert!(lab.consumes_escape());
        assert!(apply(Shortcut::Back, &mut lab, &c, false, &mut fx));
        assert!(!lab.drawer_open);
        lab.mode = LabMode::Presenter;
        assert!(apply(Shortcut::Back, &mut lab, &c, false, &mut fx));
        assert_eq!(lab.mode, LabMode::Workbench);
        assert!(!lab.consumes_escape(), "plain Escape falls through to the pause menu");
        assert!(!apply(Shortcut::Back, &mut lab, &c, false, &mut fx));
    }

    #[test]
    fn toggles() {
        let mut lab = LabState::default();
        let mut fx = Effects::default();
        let c = channels(Status::Paused);
        apply(Shortcut::Presenter, &mut lab, &c, false, &mut fx);
        assert_eq!(lab.mode, LabMode::Presenter);
        apply(Shortcut::Inspector, &mut lab, &c, false, &mut fx);
        assert!(!lab.inspector_open);
        apply(Shortcut::Library, &mut lab, &c, false, &mut fx);
        assert!(lab.drawer_open);
    }
}
```

`lab/state.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspector_width_is_clamped() {
        assert_eq!(LabState::clamp_width(100.0), 220.0);
        assert_eq!(LabState::clamp_width(300.0), 300.0);
        assert_eq!(LabState::clamp_width(900.0), 420.0);
        assert_eq!(LabState::default().inspector_width, 260.0);
    }

    #[test]
    fn presenter_transport_fades_after_two_idle_seconds() {
        let mut lab = LabState::default();
        lab.note_pointer(false, 1.9);
        assert_eq!(lab.transport_alpha(), 1.0);
        lab.note_pointer(false, 0.4);
        assert_eq!(lab.transport_alpha(), 0.0);
        lab.note_pointer(true, 0.016);
        assert_eq!(lab.transport_alpha(), 1.0);
    }
}
```

`lab/library.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use models::workbench::specimens::Specimens;

    fn available() -> HashSet<String> {
        ["life:conway", "lenia", "regular:nfa", "swarm:boids"].map(String::from).into()
    }

    #[test]
    fn groups_follow_library_order_and_hide_unavailable_specimens() {
        let specimens = Specimens::embedded();
        let groups = entries(&specimens, &available(), "");
        let names: Vec<_> = groups.iter().map(|(g, _)| g.label()).collect();
        assert_eq!(names, ["cellular", "machines", "agents"], "empty groups are omitted");
        assert_eq!(groups[0].1.len(), 2);
    }

    #[test]
    fn filter_matches_title_subtitle_and_group_case_insensitively() {
        let specimens = Specimens::embedded();
        let only = |f| entries(&specimens, &available(), f).into_iter().flat_map(|(_, s)| s).map(|s| s.id.clone()).collect::<Vec<_>>();
        assert_eq!(only("GAME"), ["life:conway"]);
        assert_eq!(only("a*b"), ["regular:nfa"]);
        assert_eq!(only("agents"), ["swarm:boids"]);
        assert!(only("zzz").is_empty());
    }
}
```

`lab/chrome/transport.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::automata::channel::metrics::MetricSample;

    #[test]
    fn sparkline_spans_the_rect_with_max_at_the_top() {
        let mut trace = Trace::new(10);
        for (t, v) in [(0, 0.0), (1, 5.0), (2, 10.0)] {
            trace.push(MetricSample { tick: t, epoch: 0, value: v });
        }
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 20.0));
        let points = sparkline(&trace, rect);
        assert_eq!(points.first().unwrap(), &pos2(0.0, 20.0));
        assert_eq!(points.last().unwrap(), &pos2(100.0, 0.0));
    }

    #[test]
    fn flat_or_short_traces() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 20.0));
        let mut trace = Trace::new(10);
        trace.push(MetricSample { tick: 0, epoch: 0, value: 3.0 });
        assert!(sparkline(&trace, rect).is_empty(), "one sample draws nothing");
        trace.push(MetricSample { tick: 1, epoch: 0, value: 3.0 });
        assert!(sparkline(&trace, rect).iter().all(|p| p.y == 10.0), "flat trace sits mid-height");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p ui lab`
Expected: unresolved names.

- [ ] **Step 3: Implement state and shortcuts**

`lab/state.rs`, above the tests:

```rust
//! Per-viewer lab UI state: mode, panels, drafts and stage views.
use super::stage::view::StageView;
use bevy::prelude::Resource;
use engine_core::automata::channel::ChannelId;
use std::collections::HashMap;

/// Workbench (all panels) or presenter (stage and HUD only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabMode {
    /// Rail, tabs, inspector, transport and stage.
    #[default]
    Workbench,
    /// Full-window stage with HUD readouts.
    Presenter,
}

/// UI-only state; nothing here affects simulation.
#[derive(Resource, Debug)]
pub struct LabState {
    pub mode: LabMode,
    pub drawer_open: bool,
    pub settings_open: bool,
    pub inspector_open: bool,
    pub inspector_width: f32,
    pub filter: String,
    pub idle_secs: f32,
    /// Unsent symbolic input drafts.
    pub inputs: HashMap<ChannelId, String>,
    /// Zoom and pan per CPU channel.
    pub views: HashMap<ChannelId, StageView>,
    /// Message and remaining seconds.
    pub toast: Option<(String, f32)>,
}

impl Default for LabState {
    fn default() -> Self {
        Self {
            mode: LabMode::Workbench,
            drawer_open: true, // the library opens on first launch
            settings_open: false,
            inspector_open: true,
            inspector_width: Self::DEFAULT_WIDTH,
            filter: String::new(),
            idle_secs: 0.0,
            inputs: HashMap::new(),
            views: HashMap::new(),
            toast: None,
        }
    }
}

impl LabState {
    /// Default inspector width in logical points.
    pub const DEFAULT_WIDTH: f32 = 260.0;
    /// Seconds without pointer motion before the presenter transport fades.
    pub const IDLE_FADE_SECS: f32 = 2.0;

    /// Inspector width within 220–420 points.
    pub fn clamp_width(width: f32) -> f32 {
        width.clamp(220.0, 420.0)
    }

    /// Track pointer idleness for the presenter transport.
    pub fn note_pointer(&mut self, moved: bool, dt: f32) {
        self.idle_secs = if moved { 0.0 } else { self.idle_secs + dt };
    }

    /// 1 while recently active, fading to 0 over 0.3 s after the idle threshold.
    pub fn transport_alpha(&self) -> f32 {
        (1.0 - (self.idle_secs - Self::IDLE_FADE_SECS) / 0.3).clamp(0.0, 1.0)
    }

    /// Whether Escape belongs to the lab (closing the drawer or leaving presenter).
    pub fn consumes_escape(&self) -> bool {
        self.drawer_open || self.mode == LabMode::Presenter
    }
}
```

`lab/shortcuts.rs`, above the tests:

```rust
use super::{Effects, state::{LabMode, LabState}};
use bevy_egui::egui::Key;
use engine_core::automata::channel::{ChannelAction, ChannelCommand, Channels, Status};

/// Lab actions reachable from the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shortcut {
    RunPause,
    Step,
    Reset,
    Select(usize),
    Presenter,
    Inspector,
    Library,
    Back,
}

/// Key → shortcut. Arrow keys, WASD and Z/X stay with the camera.
pub fn map(key: Key) -> Option<Shortcut> {
    const DIGITS: [Key; 8] = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8];
    Some(match key {
        Key::Space => Shortcut::RunPause,
        Key::Period => Shortcut::Step,
        Key::R => Shortcut::Reset,
        Key::P => Shortcut::Presenter,
        Key::N => Shortcut::Inspector,
        Key::L => Shortcut::Library,
        Key::Escape => Shortcut::Back,
        other => Shortcut::Select(DIGITS.iter().position(|d| *d == other)?),
    })
}

/// Shortcuts for this frame's pressed keys; none while a text field has focus.
pub fn collect(keys: &[Key], typing: bool) -> Vec<Shortcut> {
    if typing { Vec::new() } else { keys.iter().filter_map(|k| map(*k)).collect() }
}

/// Apply one shortcut; returns whether the lab consumed it.
pub fn apply(shortcut: Shortcut, lab: &mut LabState, channels: &Channels, freecam: bool, fx: &mut Effects) -> bool {
    let selected = channels.selected_row();
    let mut send = |action| match selected {
        Some(row) => {
            fx.commands.push(ChannelCommand { id: row.id, action });
            true
        }
        None => false,
    };
    match shortcut {
        Shortcut::RunPause if freecam => false,
        Shortcut::RunPause => match selected.map(|r| r.status) {
            Some(Status::Running) => send(ChannelAction::Pause(true)),
            Some(Status::Paused) => send(ChannelAction::Pause(false)),
            _ => false,
        },
        Shortcut::Step if selected.is_some_and(|r| r.status == Status::Paused) => send(ChannelAction::Step),
        Shortcut::Step => false,
        Shortcut::Reset => send(ChannelAction::Reset),
        Shortcut::Select(index) => match channels.rows.get(index) {
            Some(row) => {
                fx.commands.push(ChannelCommand { id: row.id, action: ChannelAction::Select });
                true
            }
            None => false,
        },
        Shortcut::Presenter => {
            lab.mode = if lab.mode == LabMode::Presenter { LabMode::Workbench } else { LabMode::Presenter };
            true
        }
        Shortcut::Inspector => {
            lab.inspector_open = !lab.inspector_open;
            true
        }
        Shortcut::Library => {
            lab.drawer_open = !lab.drawer_open;
            true
        }
        Shortcut::Back if lab.drawer_open => {
            lab.drawer_open = false;
            true
        }
        Shortcut::Back if lab.mode == LabMode::Presenter => {
            lab.mode = LabMode::Workbench;
            true
        }
        Shortcut::Back => false,
    }
}
```

- [ ] **Step 4: Implement the lab module root**

`lab/mod.rs`:

```rust
//! The automata lab shell: rail, library drawer, channel tabs, stage, inspector, transport, presenter.
//!
//! Systems here only read snapshots and write `ChannelCommand`/`AutomataCommand`;
//! `Workbench` and `LeniaSimulation` are read to paint models and show parameters.
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use engine_common::controls::camera::{manager::stage::StageRect, motion::freecam::FreeCam};
use engine_core::{
    automata::{channel::{ChannelCommand, ChannelId, Channels, metrics::Metrics}, lenia::LeniaSimulation, GpuGridSlice},
    events::{AutomataCommand, DebugSeedSquare},
    prelude::{AppState, RuleRegistry},
};
use models::workbench::{catalog::Catalog, plugin::Workbench, specimens::Specimens};
use std::collections::HashSet;
pub mod chrome;
pub mod inspector;
pub mod library;
pub mod shortcuts;
pub mod stage;
pub mod state;

/// Everything the UI wants to change this frame, flushed once at the end.
#[derive(Default, Debug)]
pub struct Effects {
    pub commands: Vec<ChannelCommand>,
    pub automata: Vec<AutomataCommand>,
    pub open_url: Option<String>,
    pub mark_centre: bool,
}

/// Read-only inputs to every lab panel.
#[derive(SystemParam)]
pub struct LabData<'w, 's> {
    pub channels: Res<'w, Channels>,
    pub metrics: Res<'w, Metrics>,
    pub specimens: Res<'w, Specimens>,
    pub catalog: Res<'w, Catalog>,
    pub rules: Res<'w, RuleRegistry>,
    pub bench: Res<'w, Workbench>,
    pub lenia: Query<'w, 's, (&'static ChannelId, &'static LeniaSimulation)>,
    pub slices: Query<'w, 's, (&'static ChannelId, &'static GpuGridSlice)>,
    pub freecam: Query<'w, 's, (), With<FreeCam>>,
}

impl LabData<'_, '_> {
    /// Ids the library may spawn: registered CPU kinds plus registered GPU rules.
    pub fn available(&self) -> HashSet<String> {
        let mut ids: HashSet<String> = self.catalog.0.iter().map(|k| k.id().to_string()).collect();
        ids.extend(self.rules.ids().cloned());
        ids
    }
}

/// Registers the lab; added by `OutputPlugin` (Task 16).
pub struct LabPlugin;

impl Plugin for LabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<state::LabState>()
            .add_systems(PreUpdate, swallow_escape.after(bevy::input::InputSystem))
            .add_systems(
                EguiPrimaryContextPass,
                draw.after(crate::theme::ThemeSet).run_if(in_state(AppState::InGame)),
            );
    }
}

/// Keep Escape from also opening the pause menu when the lab uses it.
pub fn swallow_escape(lab: Res<state::LabState>, mut keys: ResMut<ButtonInput<KeyCode>>) {
    if lab.consumes_escape() {
        keys.clear_just_pressed(KeyCode::Escape);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut contexts: EguiContexts,
    mut lab: ResMut<state::LabState>,
    data: LabData,
    mut stage_rect: ResMut<StageRect>,
    time: Res<Time>,
    mut commands: EventWriter<ChannelCommand>,
    mut automata: EventWriter<AutomataCommand>,
    mut debug: EventWriter<DebugSeedSquare>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let mut fx = Effects::default();
    let (keys, typing) = ctx.input(|i| {
        let keys: Vec<egui::Key> = i.events.iter().filter_map(|e| match e {
            egui::Event::Key { key, pressed: true, repeat: false, .. } => Some(*key),
            _ => None,
        }).collect();
        (keys, ctx.wants_keyboard_input())
    });
    let freecam = !data.freecam.is_empty();
    for shortcut in shortcuts::collect(&keys, typing) {
        shortcuts::apply(shortcut, &mut lab, &data.channels, freecam, &mut fx);
    }
    let dt = time.delta_secs();
    lab.note_pointer(ctx.input(|i| i.pointer.delta() != egui::Vec2::ZERO), dt);
    match lab.mode {
        state::LabMode::Workbench => {
            chrome::rail::show(ctx, &mut lab, &data, &mut fx);
            chrome::tabs::show(ctx, &mut lab, &data, &mut fx);
            chrome::transport::show(ctx, &data, &mut fx);
            inspector::show(ctx, &mut lab, &data, &mut fx);
            *stage_rect = stage::show(ctx, &mut lab, &data, &mut fx, typing);
            library::show(ctx, &mut lab, &data, &mut fx);
        }
        state::LabMode::Presenter => *stage_rect = stage::presenter::show(ctx, &mut lab, &data, &mut fx, typing),
    }
    stage::overlay::toast(ctx, &mut lab, dt);
    if let Some(url) = fx.open_url.take() {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
    if fx.mark_centre
        && let Some((_, slice)) = data.slices.iter().find(|(id, _)| Some(**id) == data.channels.selected)
    {
        debug.write(DebugSeedSquare { slice: slice.clone(), value: 255 });
    }
    commands.write_batch(fx.commands);
    automata.write_batch(fx.automata);
}
```

`RuleRegistry::ids()` is used by today's spawner (`rules.ids().par_bridge()`), so it returns an iterator of `&String`.

`ctx.open_url` failures surface asynchronously. bevy_egui's `open_url` support logs an error if the browser cannot be launched. Toast on failure is best-effort: set `lab.toast = Some(("Could not open the DOI link".into(), 4.0))` only when `webbrowser`'s result is observable. With bevy_egui it is not, so the toast path is exercised by `overlay::toast` alone and the log entry is the failure record.

- [ ] **Step 5: Implement the chrome and the library**

`lab/chrome/mod.rs`:

```rust
//! Fixed chrome around the stage: icon rail, channel tabs and the transport bar.
pub mod rail;
pub mod tabs;
pub mod transport;
```

`lab/chrome/rail.rs`:

```rust
//! 32 px icon rail: library (painted book spines) and settings (⁂).
use crate::{lab::{Effects, LabData, state::{LabMode, LabState}}, theme::{fonts::{SERIF, family}, tokens::*, widgets::{note, serif}}};
use bevy_egui::egui::{self, Frame, Margin, RichText, Sense, Stroke, vec2};
use engine_core::automata::channel::Backend;

/// Draw the rail and, when open, the settings popover.
pub fn show(ctx: &egui::Context, lab: &mut LabState, data: &LabData, fx: &mut Effects) {
    egui::SidePanel::left("lab.rail")
        .exact_width(32.0)
        .resizable(false)
        .frame(Frame::new().fill(PANEL).inner_margin(Margin::symmetric(0, 10)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let (rect, library) = ui.allocate_exact_size(vec2(32.0, 26.0), Sense::click());
                let colour = if lab.drawer_open { SIGNAL } else { INK_DIM };
                for (dx, h) in [(-5.0, 12.0), (0.0, 14.0), (5.0, 11.0)] {
                    let x = rect.center().x + dx;
                    ui.painter().line_segment(
                        [egui::pos2(x, rect.bottom() - 6.0), egui::pos2(x, rect.bottom() - 6.0 - h)],
                        Stroke::new(2.5, colour),
                    );
                }
                if library.on_hover_text("Library (L)").clicked() {
                    lab.drawer_open = !lab.drawer_open;
                }
                let gear = RichText::new("⁂").family(family(SERIF)).size(16.0)
                    .color(if lab.settings_open { SIGNAL } else { INK_DIM });
                if ui.add(egui::Label::new(gear).sense(Sense::click())).on_hover_text("Settings").clicked() {
                    lab.settings_open = !lab.settings_open;
                }
            });
        });
    if !lab.settings_open {
        return;
    }
    egui::Area::new("lab.settings".into())
        .fixed_pos(egui::pos2(36.0, 44.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            Frame::new().fill(PANEL).inner_margin(12).corner_radius(6).show(ui, |ui| {
                ui.label(serif("Settings").size(size::TITLE));
                if ui.button("Presenter mode (P)").clicked() {
                    lab.mode = LabMode::Presenter;
                    lab.settings_open = false;
                }
                let gpu = data.channels.selected_row().is_some_and(|r| r.backend == Backend::Gpu);
                if ui.add_enabled(gpu, egui::Button::new("Mark centre of selected GPU specimen")).clicked() {
                    fx.mark_centre = true;
                }
                note(ui, "Debug overlay: F3", false);
            });
        });
}
```

`lab/chrome/tabs.rs`:

```rust
//! Channel tabs: serif specimen names, status lamps, close and add.
use crate::{lab::{Effects, LabData, state::LabState}, theme::{tokens::*, widgets::{lamp, serif}}};
use bevy_egui::egui::{self, Frame, Margin, Sense, Stroke};
use engine_core::automata::channel::{ChannelAction, ChannelCommand, ChannelRow};
use models::workbench::specimens::Specimens;

/// Tab label: the specimen title, falling back to the catalog id.
pub fn title(row: &ChannelRow, specimens: &Specimens) -> String {
    specimens.get(&row.kind).map_or_else(|| row.kind.clone(), |s| s.title.clone())
}

/// Draw the top tab bar.
pub fn show(ctx: &egui::Context, lab: &mut LabState, data: &LabData, fx: &mut Effects) {
    egui::TopBottomPanel::top("lab.tabs")
        .exact_height(28.0)
        .frame(Frame::new().fill(PANEL).inner_margin(Margin::symmetric(12, 0)))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 16.0;
                for row in &data.channels.rows {
                    let selected = data.channels.selected == Some(row.id);
                    let response = ui.horizontal(|ui| {
                        lamp(ui, row.status);
                        let name = ui.add(egui::Label::new(serif(title(row, &data.specimens)).size(size::TITLE - 2.0)
                            .color(if selected { INK_HI } else { INK_DIM })).sense(Sense::click()));
                        if name.clicked() {
                            fx.commands.push(ChannelCommand { id: row.id, action: ChannelAction::Select });
                        }
                        let close = ui.add(egui::Label::new(egui::RichText::new("×").color(TRACK)).sense(Sense::click()));
                        if close.on_hover_text("Close").clicked() {
                            fx.commands.push(ChannelCommand { id: row.id, action: ChannelAction::Remove });
                        }
                    });
                    if selected {
                        let r = response.response.rect;
                        ui.painter().line_segment([r.left_bottom(), r.right_bottom()], Stroke::new(2.0, SIGNAL));
                    }
                }
                let add = ui.add(egui::Label::new(serif("+").color(SIGNAL).size(size::TITLE)).sense(Sense::click()));
                if add.on_hover_text("Open a specimen (L)").clicked() {
                    lab.drawer_open = true;
                }
            });
        });
}
```

`lab/chrome/transport.rs`, above its tests:

```rust
//! Transport bar: run/pause, step, reset, metric trace and steps/s selector.
use crate::{lab::{Effects, LabData}, theme::{tokens::*, widgets::{mono, pill, serif_italic}}};
use bevy_egui::egui::{self, Frame, Margin, Pos2, Rect, Sense, Stroke, pos2, vec2};
use engine_core::automata::channel::{ChannelAction, ChannelCommand, Rate, Status, metrics::Trace};

/// Sparkline vertices: oldest at the left, the trace maximum at the top.
pub fn sparkline(trace: &Trace, rect: Rect) -> Vec<Pos2> {
    let Some((lo, hi)) = trace.range() else { return Vec::new() };
    let n = trace.len();
    if n < 2 {
        return Vec::new();
    }
    trace
        .samples()
        .enumerate()
        .map(|(i, s)| {
            let x = rect.left() + rect.width() * i as f32 / (n - 1) as f32;
            let t = if hi > lo { ((s.value - lo) / (hi - lo)) as f32 } else { 0.5 };
            pos2(x, rect.bottom() - t * rect.height())
        })
        .collect()
}

/// Draw the bottom transport bar for the selected channel.
pub fn show(ctx: &egui::Context, data: &LabData, fx: &mut Effects) {
    egui::TopBottomPanel::bottom("lab.transport")
        .exact_height(40.0)
        .frame(Frame::new().fill(PANEL).inner_margin(Margin::symmetric(12, 0)))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let row = data.channels.selected_row();
                let status = row.map(|r| r.status);
                let mut send = |action| if let Some(r) = row { fx.commands.push(ChannelCommand { id: r.id, action }) };
                let running = status == Some(Status::Running);
                let runnable = matches!(status, Some(Status::Running | Status::Paused));
                if pill(ui, if running { "Pause" } else { "Run" }, true, runnable).clicked() {
                    send(ChannelAction::Pause(running));
                }
                if pill(ui, "Step", false, status == Some(Status::Paused)).clicked() {
                    send(ChannelAction::Step);
                }
                if pill(ui, "Reset", false, row.is_some()).clicked() {
                    send(ChannelAction::Reset);
                }
                let width = (ui.available_width() - 150.0).max(40.0);
                let (rect, _) = ui.allocate_exact_size(vec2(width, 26.0), Sense::hover());
                if let Some(metric) = row.and_then(|r| data.metrics.get(r.id)) {
                    ui.painter().text(rect.left_top(), egui::Align2::LEFT_TOP, metric.id.label,
                        egui::FontId::new(size::LABEL, crate::theme::fonts::family(crate::theme::fonts::SERIF_ITALIC)), INK_DIM);
                    let points = sparkline(&metric.trace, rect.shrink2(vec2(0.0, 4.0)));
                    ui.painter().add(egui::Shape::line(points, Stroke::new(1.2, SIGNAL)));
                }
                ui.label(serif_italic("steps/s"));
                for rate in Rate::ALL {
                    let on = row.is_some_and(|r| r.settings.rate == rate);
                    let text = mono(rate.hz().to_string()).color(if on { SIGNAL } else { INK_DIM });
                    if ui.add(egui::Label::new(text).sense(Sense::click())).clicked() {
                        send(ChannelAction::SetRate(rate));
                    }
                }
            });
        });
}
```

`lab/library.rs`, above its tests:

```rust
//! Specimen library drawer: grouped, filterable, opens a channel on click.
use super::{Effects, LabData, state::LabState};
use crate::theme::{tokens::*, widgets::{margin_section, note, pill, serif, serif_italic}};
use bevy_egui::egui::{self, Frame, Sense};
use engine_core::{automata::channel::ChannelIds, events::AutomataCommand};
use models::workbench::{plugin::FULL_NOTE, specimens::{Group, Specimen, Specimens}};
use std::collections::HashSet;

/// Library groups in display order, omitting empty groups; filter is case-insensitive.
pub fn entries<'a>(specimens: &'a Specimens, available: &HashSet<String>, filter: &str) -> Vec<(Group, Vec<&'a Specimen>)> {
    let needle = filter.trim().to_lowercase();
    Group::ALL
        .into_iter()
        .map(|group| {
            let items = specimens.0.iter().filter(|s| {
                s.group == group
                    && available.contains(&s.id)
                    && (needle.is_empty()
                        || [s.title.as_str(), s.subtitle.as_str(), group.label()].iter().any(|t| t.to_lowercase().contains(&needle)))
            });
            (group, items.collect::<Vec<_>>())
        })
        .filter(|(_, items)| !items.is_empty())
        .collect()
}

/// Draw the dimming layer and the drawer while it is open.
pub fn show(ctx: &egui::Context, lab: &mut LabState, data: &LabData, fx: &mut Effects) {
    if !lab.drawer_open {
        return;
    }
    let screen = ctx.screen_rect();
    let dim = egui::Rect::from_min_max(egui::pos2(32.0, 0.0), screen.max);
    egui::Area::new("lab.dim".into()).fixed_pos(dim.min).order(egui::Order::Middle).show(ctx, |ui| {
        let response = ui.allocate_rect(egui::Rect::from_min_size(egui::Pos2::ZERO + dim.min.to_vec2(), dim.size()), Sense::click());
        ui.painter().rect_filled(dim, 0.0, BG.gamma_multiply(0.55));
        if response.clicked() {
            lab.drawer_open = false;
        }
    });
    egui::Area::new("lab.drawer".into()).fixed_pos(egui::pos2(32.0, 0.0)).order(egui::Order::Foreground).show(ctx, |ui| {
        Frame::new().fill(PANEL).inner_margin(egui::Margin { left: 4, right: 14, top: 14, bottom: 14 }).show(ui, |ui| {
            ui.set_width(300.0);
            ui.set_min_height(screen.height() - 28.0);
            ui.add(egui::TextEdit::singleline(&mut lab.filter).hint_text(serif_italic("filter specimens…")).frame(false));
            ui.add_space(10.0);
            let full = data.channels.is_full();
            for (group, items) in entries(&data.specimens, &data.available(), &lab.filter) {
                margin_section(ui, group.label(), false, |ui| {
                    for specimen in items {
                        let gpu = specimen.id == "lenia";
                        let response = ui.horizontal(|ui| {
                            ui.add(egui::Label::new(serif(&specimen.title).color(INK)).sense(Sense::click()))
                                | ui.add(egui::Label::new(serif_italic(&specimen.subtitle)).sense(Sense::click()))
                                | if gpu { ui.label(crate::theme::widgets::mono("GPU").size(size::LABEL).color(INFO)) } else { ui.label("") }
                        }).inner;
                        if response.clicked() && !full {
                            fx.automata.push(AutomataCommand::SeedPattern { id: specimen.id.clone() });
                            lab.drawer_open = false;
                        }
                    }
                });
            }
            margin_section(ui, "channels", false, |ui| {
                ui.horizontal(|ui| {
                    ui.label(crate::theme::widgets::mono(format!("{}", data.channels.rows.len())));
                    ui.label(serif_italic(format!("of {} open", ChannelIds::CAPACITY)));
                    if pill(ui, "clear all", false, !data.channels.rows.is_empty()).clicked() {
                        fx.automata.push(AutomataCommand::Clear);
                    }
                });
                if full {
                    note(ui, FULL_NOTE, true);
                }
            });
        });
    });
}
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p ui lab::shortcuts lab::state lab::library lab::chrome`
Expected: all pass. `lab::stage` and `lab::inspector` are declared in `mod.rs`, but their files arrive in Tasks 14–15. To compile now, create `lab/inspector/mod.rs` with `pub fn show(_: &bevy_egui::egui::Context, _: &mut super::state::LabState, _: &super::LabData, _: &mut super::Effects) {}`. Also create `lab/stage/{mod.rs, view.rs, overlay.rs, presenter.rs}` stubs: `view.rs` defines `#[derive(Clone, Copy, Debug, Default, PartialEq)] pub struct StageView { pub zoom: f32, pub pan: bevy_egui::egui::Vec2 }`; `overlay.rs` defines `pub fn toast(_: &bevy_egui::egui::Context, _: &mut crate::lab::state::LabState, _: f32) {}`; and `mod.rs` and `presenter.rs` each define a `show` that returns `StageRect::default()`. Tasks 14–15 replace these stubs.

- [ ] **Step 7: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy -p ui --all-targets -- -D warnings
git add lib/ui/src/lab lib/ui/src/lib.rs
git commit -m "feat: add lab state shortcuts rail tabs transport and library drawer

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---
### Task 14: Inspector

**Files:**
- Replace stub: `lib/ui/src/lab/inspector/mod.rs`
- Create: `lib/ui/src/lab/inspector/{sections.rs,machine.rs,lenia.rs}`

**Interfaces:**
- Consumes: Tasks 1, 2, 9, 11 and 13. Reads `Session` and `Model`/`Symbolic` from `Workbench`, and `LeniaSimulation.params` (`mu`, `sigma`, `dt`) and `.seed` (`LeniaSeed::{Blob, Ring}`).
- Produces:
  - `inspector::show(ctx, &mut LabState, &LabData, &mut Effects)`.
  - In `sections.rs`: `format_metric(value: f64, id: &MetricId) -> String`, `arena_label(&Model, Kind) -> String` and `input_note(len: usize) -> (String, bool)`.
  - In `machine.rs`: `summary(&Symbolic) -> Vec<(&'static str, String)>` and `turing_table(&TuringMachine) -> Vec<String>`.

- [ ] **Step 1: Write the failing tests**

`inspector/machine.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use models::workbench::{catalog::Kind, symbolic::machine::Symbolic};

    #[test]
    fn summaries_name_state_and_result() {
        let nfa = Symbolic::new(Kind::Nfa, "aaab");
        let rows = summary(&nfa);
        assert_eq!(rows[0], ("result", "running".to_string()));
        assert!(rows.iter().any(|(k, _)| *k == "states"));
        let mut pda = Symbolic::new(Kind::Pushdown, "aabb");
        pda.step();
        assert!(summary(&pda).iter().any(|(k, v)| *k == "stack" && v == "a"));
    }

    #[test]
    fn turing_table_lists_every_transition_sorted() {
        let Symbolic::Turing(tm) = Symbolic::new(Kind::Turing, "aa") else { unreachable!() };
        let table = turing_table(&tm);
        assert_eq!(table, ["q0 ␣ → ␣ L q1", "q0 a → X R q0"]);
    }
}
```

`inspector/sections.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use models::workbench::{catalog::Kind, session::Session};

    #[test]
    fn metric_values_format_by_kind() {
        let order = MetricId { name: "polarisation", label: "order", unit: "0–1" };
        let pop = MetricId { name: "population", label: "alive", unit: "cells" };
        assert_eq!(format_metric(0.912_3, &order), "0.91");
        assert_eq!(format_metric(212.0, &pop), "212");
    }

    #[test]
    fn arenas_describe_size_and_edges() {
        assert_eq!(arena_label(&Session::new(Kind::Life).model, Kind::Life), "64 × 64 · dead edges · CPU");
        assert_eq!(arena_label(&Session::new(Kind::Rule30).model, Kind::Rule30), "64 cells · 64-row history · CPU");
        assert_eq!(arena_label(&Session::new(Kind::Boids).model, Kind::Boids), "64 × 64 · wraps · CPU");
    }

    #[test]
    fn input_note_warns_beyond_the_bound() {
        assert_eq!(input_note(6), ("applies on reset · 6/256".to_string(), false));
        assert_eq!(input_note(300).1, true);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p ui inspector`
Expected: unresolved `summary`, `format_metric` and related names.

- [ ] **Step 3: Implement**

`inspector/machine.rs`:

```rust
//! Symbolic machine readouts: result, states, stack, tape position, Turing table.
use models::{automata::classical::turing::universal_tm::TuringMachine, workbench::symbolic::machine::Symbolic};

/// Key/value rows for the inspector's machine section.
pub fn summary(machine: &Symbolic) -> Vec<(&'static str, String)> {
    let result = match machine.result() {
        Some(true) => "accepted",
        Some(false) => "rejected",
        None if machine.halted() => "bounded (4096 symbols)",
        None => "running",
    };
    let mut rows = vec![("result", result.to_string())];
    match machine {
        Symbolic::Finite(m) => {
            let mut states: Vec<_> = m.states.iter().copied().collect();
            states.sort_unstable();
            rows.push(("states", format!("{states:?}")));
            rows.push(("read", format!("{}/{}", m.cursor, m.input.len())));
        }
        Symbolic::Pushdown(m) => {
            rows.push(("stack", m.stack.iter().collect()));
            rows.push(("read", format!("{}/{}", m.cursor, m.input.len())));
        }
        Symbolic::Bounded(m) => rows.push(("head", format!("{}/{}", m.head, m.tape.len()))),
        Symbolic::Turing(m) => rows.push(("state", format!("q{}", m.state))),
        Symbolic::Koch { word, .. } => rows.push(("symbols", word.chars().count().to_string())),
    }
    rows
}

/// `q<state> <read> → <write> <L|R> q<next>` rows, blank shown as `␣`, sorted.
pub fn turing_table(machine: &TuringMachine) -> Vec<String> {
    let glyph = |c: char| if c == ' ' { '␣' } else { c };
    let mut rows: Vec<String> = machine
        .transitions
        .iter()
        .map(|((state, read), t)| {
            let dir = if matches!(t.direction, models::automata::classical::turing::universal_tm::Direction::Left) { 'L' } else { 'R' };
            format!("q{state} {} → {} {dir} q{}", glyph(*read), glyph(t.write), t.new_state)
        })
        .collect();
    rows.sort();
    rows
}
```

The `transitions`, `direction` and `Direction` names come from `universal_tm.rs` (seen at lines 85–110). If `transitions` is private, add a `pub fn transitions(&self) -> &HashMap<(usize, char), TMTransition>` accessor there.

`inspector/sections.rs`:

```rust
//! Pure formatting helpers for inspector sections.
use engine_core::automata::channel::metrics::MetricId;
use models::workbench::{catalog::Kind, session::Model};

/// Order parameters keep two decimals; counts are integers.
pub fn format_metric(value: f64, id: &MetricId) -> String {
    if id.name == "polarisation" || id.name == "mass" { format!("{value:.2}") } else { format!("{value:.0}") }
}

/// Size, boundary and backend for the `grid` margin section.
pub fn arena_label(model: &Model, kind: Kind) -> String {
    match model {
        Model::Grid(_) if matches!(kind, Kind::Rule30 | Kind::Rule110) => "64 cells · 64-row history · CPU".into(),
        Model::Grid(_) => "64 × 64 · dead edges · CPU".into(),
        Model::Boids(_) => "64 × 64 · wraps · CPU".into(),
        Model::Ants(_) => "64 × 64 · clamped · CPU".into(),
        Model::Symbolic(_) => "tape · CPU".into(),
    }
}

/// Input hint and whether it is a warning (over the 256-symbol bound).
pub fn input_note(len: usize) -> (String, bool) {
    if len > 256 {
        (format!("{len}/256: too long, Reset is refused"), true)
    } else {
        (format!("applies on reset · {len}/256"), false)
    }
}
```

`inspector/lenia.rs`:

```rust
//! Lenia growth sliders, seed choice and the opt-in GPU mass toggle.
use crate::{lab::Effects, theme::widgets::{choice, margin_section, note, slider}};
use bevy_egui::egui::{self, Ui};
use engine_core::automata::{
    channel::{ChannelAction, ChannelCommand, ChannelId, metrics::{ChannelMetric, MetricAvailability}},
    lenia::{LeniaAction, LeniaSimulation},
};
use simulation_kernel::core::lenia::{LeniaParams, LeniaSeed};

/// Growth, seed and sampling sections for one Lenia channel.
pub fn show(ui: &mut Ui, id: ChannelId, sim: &LeniaSimulation, metric: Option<&ChannelMetric>, fx: &mut Effects) {
    let send = |fx: &mut Effects, action| fx.commands.push(ChannelCommand { id, action });
    margin_section(ui, "growth", false, |ui| {
        let mut params = sim.params;
        slider(ui, "μ", &mut params.mu, 0.0..=1.0, false);
        slider(ui, "σ", &mut params.sigma, 0.001..=1.0, true);
        slider(ui, "Δt", &mut params.dt, 0.0..=1.0, false);
        if ui.link(crate::theme::widgets::serif_italic("restore defaults")).clicked() {
            params = LeniaParams::default();
        }
        if params != sim.params {
            send(fx, ChannelAction::Lenia(LeniaAction::Parameters(params)));
        }
    });
    margin_section(ui, "seed", false, |ui| {
        let mut seed = sim.seed;
        if choice(ui, &mut seed, &[(LeniaSeed::Blob, "blob"), (LeniaSeed::Ring, "ring")]) {
            send(fx, ChannelAction::Lenia(LeniaAction::Seed(seed)));
        }
    });
    margin_section(ui, "sample", false, |ui| {
        let mut on = !matches!(metric.map(|m| &m.availability), None | Some(MetricAvailability::OptIn));
        if ui.add(egui::Checkbox::new(&mut on, crate::theme::widgets::serif_italic("mass (GPU readback)"))).changed() {
            send(fx, ChannelAction::SampleMetric(on));
        }
        if let Some(MetricAvailability::Unavailable(reason)) = metric.map(|m| &m.availability) {
            note(ui, reason, true);
        }
    });
}
```

`inspector/mod.rs`:

```rust
//! Right-hand margin inspector: readout, parameters, input, colour, arena, footnote.
use super::{Effects, LabData, state::LabState};
use crate::theme::{tokens::*, widgets::*};
use bevy_egui::egui::{self, Frame, Margin, Sense};
use engine_core::automata::channel::{Backend, ChannelAction, ChannelCommand, SWATCHES, Srgb};
use models::workbench::session::Model;
mod lenia;
pub mod machine;
pub mod sections;

/// Draw the inspector panel unless collapsed (N).
pub fn show(ctx: &egui::Context, lab: &mut LabState, data: &LabData, fx: &mut Effects) {
    if !lab.inspector_open {
        return;
    }
    let panel = egui::SidePanel::right("lab.inspector")
        .resizable(true)
        .default_width(lab.inspector_width)
        .width_range(220.0..=420.0)
        .frame(Frame::new().fill(PANEL).inner_margin(Margin { left: 2, right: 12, top: 14, bottom: 14 }))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| body(ui, lab, data, fx));
        });
    lab.inspector_width = LabState::clamp_width(panel.response.rect.width());
}

fn body(ui: &mut egui::Ui, lab: &mut LabState, data: &LabData, fx: &mut Effects) {
    let Some(row) = data.channels.selected_row() else {
        note(ui, "No specimen selected.", false);
        return;
    };
    let id = row.id;
    let metric = data.metrics.get(id);
    margin_section(ui, "readout", true, |ui| {
        readout(ui, &tick_digits(row.tick), "tick", true);
        if let Some(m) = metric.filter(|m| m.latest.is_some()) {
            readout(ui, &sections::format_metric(m.latest.unwrap().value, &m.id), m.id.label, true);
        }
    });
    let session = data.bench.sessions.iter().find(|s| s.channel == id);
    if let Some(session) = session {
        if let Model::Symbolic(machine) = &session.model {
            margin_section(ui, "machine", false, |ui| {
                for (key, value) in machine::summary(machine) {
                    ui.horizontal(|ui| {
                        ui.label(serif_italic(key));
                        ui.label(mono(value));
                    });
                }
                if let models::workbench::symbolic::machine::Symbolic::Turing(tm) = machine {
                    for line in machine::turing_table(tm) {
                        ui.label(mono(line).color(INK));
                    }
                }
            });
            margin_section(ui, "input", false, |ui| {
                let draft = lab.inputs.entry(id).or_insert_with(|| session.input.clone());
                if ui.add(egui::TextEdit::singleline(draft).char_limit(1024)).changed() {
                    fx.commands.push(ChannelCommand { id, action: ChannelAction::SetInput(draft.clone()) });
                }
                let (text, warn) = sections::input_note(draft.chars().count());
                note(ui, &text, warn);
            });
        }
        margin_section(ui, "grid", false, |ui| {
            ui.label(mono(sections::arena_label(&session.model, session.kind)).color(INK_DIM));
        });
    }
    if row.backend == Backend::Gpu
        && let Some((_, sim)) = data.lenia.iter().find(|(lid, _)| **lid == id)
    {
        lenia::show(ui, id, sim, metric, fx);
        margin_section(ui, "grid", false, |ui| {
            ui.label(mono("64 × 64 × 8 · zero edges · GPU").color(INFO));
        });
    }
    margin_section(ui, "colour", false, |ui| {
        ui.horizontal(|ui| {
            for swatch in SWATCHES {
                let (rect, response) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), Sense::click());
                if row.settings.colour == swatch {
                    ui.painter().circle_filled(rect.center(), 7.0, INK_HI);
                    ui.painter().circle_filled(rect.center(), 5.5, BG);
                }
                ui.painter().circle_filled(rect.center(), 5.0, color(swatch));
                if response.clicked() {
                    fx.commands.push(ChannelCommand { id, action: ChannelAction::SetColour(swatch) });
                }
            }
            let mut rgb = row.settings.colour.0;
            if egui::color_picker::color_edit_button_srgb(ui, &mut rgb).changed() {
                fx.commands.push(ChannelCommand { id, action: ChannelAction::SetColour(Srgb(rgb)) });
            }
        });
    });
    if let Some(specimen) = data.specimens.get(&row.kind) {
        margin_section(ui, "", false, |ui| {
            if let Some(url) = footnote(ui, 1, &specimen.explanation, &specimen.citation, &specimen.doi) {
                fx.open_url = Some(url);
            }
        });
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p ui inspector`
Expected: 5 tests pass. If `turing_table`'s expected strings differ, check the real transitions in `example_tm_replace_a` (blank → write blank, move Left, accept state 1; `a` → write `X`, move Right, stay in 0). Fix the expectation only if the code's formatting, not its semantics, differs.

- [ ] **Step 5: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy -p ui --all-targets -- -D warnings
git add lib/ui/src/lab/inspector lib/models
git commit -m "feat: add margin inspector with readouts machine state input colour and citations

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 15: Stage, painters, overlay and presenter

**Files:**
- Replace stubs: `lib/ui/src/lab/stage/{mod.rs,view.rs,overlay.rs,presenter.rs}`
- Create: `lib/ui/src/lab/stage/painters/{mod.rs,grid.rs,symbolic.rs,agents.rs}`

**Interfaces:**
- Consumes: Task 12 `StageRect`; Task 1 `ramp`, `Srgb`; Task 10 `color`; models `GridSession` (`grid.size`, `history`, `bytes()`), `Flock.boids`, `Foraging.colony{ants, food_sources, home, pheromone}`, `Symbolic`.
- Produces:
  - `stage::show(ctx, &mut LabState, &LabData, &mut Effects, typing: bool) -> StageRect` and `stage::presenter::show` with the same signature.
  - `stage::view::{StageView, Fit, fit(content: Vec2, rect: Rect, view: StageView) -> Fit, Fit::to_screen(Vec2) -> Pos2, Fit::to_cell(Pos2) -> Option<(i32, i32)>, StageView::zoom_by(f32)}`.
  - `stage::painters::{ramp_colour([Srgb; 4], t: f32) -> Color32, paint(painter, &Fit, &Model, Kind, Srgb), content_size(&Model) -> Vec2}`.
  - `stage::painters::symbolic::tape_window(len, head, max) -> Range<usize>`.
  - `stage::overlay::{chrome(ctx/ui…), empty(ui), toast(ctx, &mut LabState, dt)}`.

- [ ] **Step 1: Write the failing tests**

`stage/view.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_egui::egui::{pos2, vec2};

    #[test]
    fn content_is_centred_and_fitted() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 400.0));
        let f = fit(vec2(64.0, 64.0), rect, StageView::default());
        assert!((f.scale - 400.0 * 0.92 / 64.0).abs() < 1e-4);
        assert_eq!(f.to_screen(vec2(32.0, 32.0)), pos2(400.0, 200.0));
    }

    #[test]
    fn cursor_maps_back_to_cells() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 640.0));
        let f = fit(vec2(64.0, 64.0), rect, StageView::default());
        assert_eq!(f.to_cell(f.to_screen(vec2(10.5, 3.5))), Some((10, 3)));
        assert_eq!(f.to_cell(pos2(-50.0, 0.0)), None);
    }

    #[test]
    fn zoom_is_clamped() {
        let mut v = StageView::default();
        v.zoom_by(100.0);
        assert_eq!(v.zoom, 16.0);
        v.zoom_by(0.0001);
        assert_eq!(v.zoom, 0.25);
    }
}
```

`stage/painters/mod.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::automata::channel::{SIGNAL, ramp};

    #[test]
    fn ramp_colour_hits_stops_at_thirds() {
        let stops = ramp(SIGNAL);
        assert_eq!(ramp_colour(stops, 0.0), color(stops[0]));
        assert_eq!(ramp_colour(stops, 2.0 / 3.0), color(stops[2]));
        assert_eq!(ramp_colour(stops, 1.0), color(stops[3]));
        assert_eq!(ramp_colour(stops, 7.0), color(stops[3]), "clamped");
    }
}
```

`stage/painters/symbolic.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tape_window_keeps_the_head_visible() {
        assert_eq!(tape_window(5, 2, 20), 0..6, "short tapes show all cells plus a trailing blank");
        assert_eq!(tape_window(100, 90, 20), 80..100);
        assert_eq!(tape_window(100, 3, 20), 0..20);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p ui stage`
Expected: unresolved `fit`, `ramp_colour`, `tape_window` and related names.

- [ ] **Step 3: Implement the view and painters**

`stage/view.rs`:

```rust
//! Per-channel zoom/pan and the content → screen fit used by CPU painters.
use bevy_egui::egui::{Pos2, Rect, Vec2, vec2};

/// Viewer-controlled zoom and pan for one CPU channel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageView {
    pub zoom: f32,
    pub pan: Vec2,
}

impl Default for StageView {
    fn default() -> Self {
        Self { zoom: 1.0, pan: Vec2::ZERO }
    }
}

impl StageView {
    /// Multiply zoom, clamped to 0.25–16×.
    pub fn zoom_by(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(0.25, 16.0);
    }
}

/// Uniform scale and origin mapping content units to screen points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub scale: f32,
    pub origin: Pos2,
    pub content: Vec2,
}

impl Fit {
    /// Content coordinate → screen point.
    pub fn to_screen(&self, p: Vec2) -> Pos2 {
        self.origin + p * self.scale
    }
    /// Screen point → integer cell, if inside the content.
    pub fn to_cell(&self, p: Pos2) -> Option<(i32, i32)> {
        let c = (p - self.origin) / self.scale;
        (c.x >= 0.0 && c.y >= 0.0 && c.x < self.content.x && c.y < self.content.y).then(|| (c.x as i32, c.y as i32))
    }
}

/// Fit `content` into 92 % of `rect`, then apply zoom about the centre and pan.
pub fn fit(content: Vec2, rect: Rect, view: StageView) -> Fit {
    let content = content.max(vec2(1.0, 1.0));
    let scale = (rect.width() / content.x).min(rect.height() / content.y) * 0.92 * view.zoom;
    let origin = rect.center() - content * scale * 0.5 + view.pan;
    Fit { scale, origin, content }
}
```

`stage/painters/mod.rs`:

```rust
//! Model painters for CPU channels; every colour derives from the channel colour.
use super::view::Fit;
use crate::theme::tokens::color;
use bevy_egui::egui::{Color32, Painter, Vec2, vec2};
use engine_core::automata::channel::Srgb;
use models::workbench::{catalog::Kind, session::Model, symbolic::machine::Symbolic};
pub mod agents;
pub mod grid;
pub mod symbolic;

/// Faint marker for empty cells when cells are large enough to show it.
pub const EMPTY_DOT: Color32 = Color32::from_rgb(0x2c, 0x2b, 0x27);

/// Piecewise-linear sample of a four-stop ramp at `t` ∈ [0, 1].
pub fn ramp_colour(stops: [Srgb; 4], t: f32) -> Color32 {
    let x = t.clamp(0.0, 1.0) * 3.0;
    let i = (x.floor() as usize).min(2);
    let f = x - i as f32;
    let (a, b) = (stops[i].0, stops[i + 1].0);
    let lerp = |k: usize| (f32::from(a[k]) + (f32::from(b[k]) - f32::from(a[k])) * f).round() as u8;
    if f == 0.0 { color(stops[i]) } else if (f - 1.0).abs() < f32::EPSILON { color(stops[i + 1]) } else { Color32::from_rgb(lerp(0), lerp(1), lerp(2)) }
}

/// Content size in model units for fitting.
pub fn content_size(model: &Model) -> Vec2 {
    match model {
        Model::Grid(g) if g.grid.size.y == 1 => vec2(g.grid.size.x as f32, 64.0),
        Model::Grid(g) => vec2(g.grid.size.x as f32, g.grid.size.y as f32),
        Model::Symbolic(Symbolic::Koch { word, .. }) => symbolic::koch_extent(word),
        Model::Symbolic(_) => vec2(20.0, 6.0),
        Model::Boids(_) | Model::Ants(_) => vec2(64.0, 64.0),
    }
}

/// Paint `model` into `painter` with `fit` in the channel colour.
pub fn paint(painter: &Painter, fit: &Fit, model: &Model, kind: Kind, colour: Srgb) {
    match model {
        Model::Grid(g) => grid::paint(painter, fit, g, kind, colour),
        Model::Symbolic(m) => symbolic::paint(painter, fit, m, colour),
        Model::Boids(f) => agents::boids(painter, fit, f, colour),
        Model::Ants(a) => agents::ants(painter, fit, a, colour),
    }
}
```

Replace the `ramp_colour` body with the simpler exact form if Clippy dislikes the float comparisons: `if f <= 0.0 { color(stops[i]) } else { … }`. The test pins only stops and clamping.

`stage/painters/grid.rs`:

```rust
//! Elementary space-time history, Life cells and HPP density.
use super::{EMPTY_DOT, Fit, ramp_colour};
use crate::theme::tokens::color;
use bevy_egui::egui::{Painter, Rect, Vec2, vec2};
use engine_core::automata::channel::{Srgb, ramp};
use models::workbench::{catalog::Kind, grid::GridSession};

/// Paint one grid session.
pub fn paint(painter: &Painter, fit: &Fit, grid: &GridSession, kind: Kind, colour: Srgb) {
    let width = grid.grid.size.x as usize;
    let cell = |x: usize, y: usize| Rect::from_min_size(fit.to_screen(vec2(x as f32, y as f32)), Vec2::splat(fit.scale)).shrink(fit.scale * 0.08);
    let dots = fit.scale >= 6.0;
    let stops = ramp(colour);
    let rows: Vec<Vec<u8>> = if grid.grid.size.y == 1 { grid.history.iter().cloned().collect() } else { grid.bytes().chunks(width).map(<[u8]>::to_vec).collect() };
    for (y, row) in rows.iter().enumerate() {
        for (x, value) in row.iter().enumerate() {
            match *value {
                0 if dots => {
                    painter.circle_filled(cell(x, y).center(), fit.scale * 0.06, EMPTY_DOT);
                }
                0 => {}
                v if kind == Kind::Hpp => {
                    painter.rect_filled(cell(x, y), fit.scale * 0.12, ramp_colour(stops, 0.35 + 0.65 * v.count_ones() as f32 / 4.0));
                }
                _ => {
                    painter.rect_filled(cell(x, y), fit.scale * 0.12, color(colour));
                }
            }
        }
    }
}
```

`stage/painters/symbolic.rs`:

```rust
//! Tapes and stacks as large cells with the head marked; Koch as a turtle path.
use super::Fit;
use crate::theme::tokens::*;
use bevy_egui::egui::{Align2, FontId, Painter, Rect, Stroke, Vec2, vec2};
use engine_core::automata::channel::Srgb;
use models::workbench::symbolic::machine::Symbolic;
use std::ops::Range;

/// Visible tape cells: the whole tape plus a trailing blank, or a `max`-wide window ending near the head.
pub fn tape_window(len: usize, head: usize, max: usize) -> Range<usize> {
    let total = len + 1;
    if total <= max {
        return 0..total;
    }
    let start = head.saturating_sub(max / 2).min(len.saturating_sub(max));
    start..(start + max).min(len)
}

fn turtle(word: &str) -> Vec<Vec2> {
    let mut points = vec![Vec2::ZERO];
    let mut angle = 0.0_f32;
    for symbol in word.chars() {
        match symbol {
            'F' => points.push(*points.last().unwrap() + Vec2::angled(angle)),
            '+' => angle += std::f32::consts::FRAC_PI_3,
            '-' => angle -= std::f32::consts::FRAC_PI_3,
            _ => {}
        }
    }
    points
}

/// Bounding size of the Koch turtle path.
pub fn koch_extent(word: &str) -> Vec2 {
    let p = turtle(word);
    let (min, max) = p.iter().fold((Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)), |(a, b), q| (a.min(*q), b.max(*q)));
    (max - min).max(vec2(1.0, 1.0))
}

/// Paint a symbolic machine.
pub fn paint(painter: &Painter, fit: &Fit, machine: &Symbolic, colour: Srgb) {
    let (tape, head): (Vec<char>, usize) = match machine {
        Symbolic::Finite(m) => (m.input.clone(), m.cursor),
        Symbolic::Pushdown(m) => (m.input.clone(), m.cursor),
        Symbolic::Bounded(m) => (m.tape.clone(), m.head),
        Symbolic::Turing(m) => (m.tape.clone(), m.head),
        Symbolic::Koch { word, .. } => return koch(painter, fit, word, colour),
    };
    let window = tape_window(tape.len(), head, 20);
    let start = window.start;
    for (slot, index) in window.enumerate() {
        let min = fit.to_screen(vec2(slot as f32, 2.0));
        let rect = Rect::from_min_size(min, vec2(fit.scale * 0.9, fit.scale * 1.2));
        let at_head = index == head;
        painter.rect_filled(rect, 3.0, if at_head { SIGNAL_BG } else { RAISED });
        if at_head {
            painter.rect_stroke(rect, 3.0, Stroke::new(2.0, color(colour)), bevy_egui::egui::StrokeKind::Outside);
        }
        let glyph = tape.get(index).map_or('␣', |c| if *c == ' ' { '␣' } else { *c });
        painter.text(rect.center(), Align2::CENTER_CENTER, glyph, FontId::monospace(fit.scale * 0.6), if glyph.is_alphanumeric() { color(colour) } else { INK });
    }
    if let Symbolic::Pushdown(m) = machine {
        for (depth, symbol) in m.stack.iter().rev().enumerate().take(4) {
            let p = fit.to_screen(vec2((head - start) as f32 + 0.45, 3.6 + depth as f32 * 0.6));
            painter.text(p, Align2::CENTER_TOP, symbol, FontId::monospace(fit.scale * 0.45), INK_DIM);
        }
    }
}

fn koch(painter: &Painter, fit: &Fit, word: &str, colour: Srgb) {
    let points = turtle(word);
    let min = points.iter().fold(Vec2::splat(f32::MAX), |a, b| a.min(*b));
    let screen: Vec<_> = points.iter().map(|p| fit.to_screen(*p - min)).collect();
    painter.add(bevy_egui::egui::Shape::line(screen, Stroke::new(1.5, color(colour))));
}
```

If `rect_stroke` in egui 0.31 takes three arguments (no `StrokeKind`), drop the fourth.

`stage/painters/agents.rs`:

```rust
//! Boids as heading triangles; ants with food, nest and pheromone ramp.
use super::{Fit, ramp_colour};
use crate::theme::tokens::*;
use bevy_egui::egui::{Painter, Rect, Shape, Stroke, Vec2, vec2};
use engine_core::automata::channel::{Srgb, ramp};
use models::workbench::agents::{Flock, Foraging};

/// Paint a flock.
pub fn boids(painter: &Painter, fit: &Fit, flock: &Flock, colour: Srgb) {
    for boid in &flock.boids {
        let heading = boid.velocity.normalize_or_zero();
        let (h, side) = (vec2(heading.x, heading.y), vec2(-heading.y, heading.x));
        let p = vec2(boid.position.x, boid.position.y);
        let points = [p + h * 1.2, p - h * 0.8 + side * 0.65, p - h * 0.4, p - h * 0.8 - side * 0.65].map(|q| fit.to_screen(q));
        painter.add(Shape::convex_polygon(points.to_vec(), color(colour), Stroke::NONE));
    }
}

/// Paint a foraging arena.
pub fn ants(painter: &Painter, fit: &Fit, arena: &Foraging, colour: Srgb) {
    let stops = ramp(colour);
    for (p, strength) in &arena.colony.pheromone {
        let rect = Rect::from_min_size(fit.to_screen(vec2(p.x as f32, p.y as f32)), Vec2::splat(fit.scale));
        painter.rect_filled(rect, 0.0, ramp_colour(stops, (strength / 6.0).clamp(0.05, 0.6)));
    }
    for food in &arena.colony.food_sources {
        painter.circle_filled(fit.to_screen(vec2(food.x as f32 + 0.5, food.y as f32 + 0.5)), fit.scale * 0.9, OK);
    }
    let home = arena.colony.home;
    painter.circle_stroke(fit.to_screen(vec2(home.x as f32 + 0.5, home.y as f32 + 0.5)), fit.scale, Stroke::new(2.0, INK_HI));
    for ant in &arena.colony.ants {
        let c = if ant.carrying_food { color(stops[3]) } else { color(colour) };
        painter.circle_filled(fit.to_screen(vec2(ant.pos.x as f32 + 0.5, ant.pos.y as f32 + 0.5)), fit.scale * 0.35, c);
    }
}
```

- [ ] **Step 4: Implement the stage, overlay and presenter**

`stage/overlay.rs`:

```rust
//! Stage chrome: top-right instrument overlay, figure caption, empty state, toast.
use crate::{lab::{LabData, state::LabState}, theme::{fonts::{SERIF_ITALIC, SERIF_SEMIBOLD, family}, tokens::*}};
use bevy_egui::egui::{self, Align2, FontId, Rect};

/// Draw the overlay and caption for the selected channel.
pub fn chrome(painter: &egui::Painter, rect: Rect, data: &LabData, cursor: Option<(i32, i32)>, zoom: f32) {
    let Some(row) = data.channels.selected_row() else { return };
    let arena = data.bench.sessions.iter().find(|s| s.channel == row.id)
        .map(|s| crate::lab::inspector::sections::arena_label(&s.model, s.kind))
        .unwrap_or_else(|| "64 × 64 × 8 · zero edges · GPU".into());
    let cursor = cursor.map_or("—".into(), |(x, y)| format!("({x}, {y})"));
    let lines = format!("{arena}\nzoom {:.0}% · {cursor}", zoom * 100.0);
    painter.text(rect.right_top() + egui::vec2(-12.0, 10.0), Align2::RIGHT_TOP, lines, FontId::monospace(size::LABEL + 1.0), INK_DIM);
    if let Some(specimen) = data.specimens.get(&row.kind) {
        let n = data.specimens.figure(&row.kind).unwrap_or(0);
        let at = rect.left_bottom() + egui::vec2(14.0, -12.0);
        let fig = painter.text(at, Align2::LEFT_BOTTOM, format!("Fig. {n}. "), FontId::new(size::BODY, family(SERIF_SEMIBOLD)), INK_HI);
        painter.text(egui::pos2(fig.right(), at.y), Align2::LEFT_BOTTOM, &specimen.caption, FontId::new(size::BODY, family(SERIF_ITALIC)), INK_DIM);
    }
}

/// Centred prompt when no channels are open.
pub fn empty(painter: &egui::Painter, rect: Rect) {
    painter.text(rect.center(), Align2::CENTER_CENTER, "Open a specimen from the library (L)", FontId::new(size::TITLE, family(SERIF_ITALIC)), INK_DIM);
}

/// Bottom-centre serif toast that expires after its duration.
pub fn toast(ctx: &egui::Context, lab: &mut LabState, dt: f32) {
    let Some((message, remaining)) = lab.toast.as_mut() else { return };
    *remaining -= dt;
    if *remaining <= 0.0 {
        lab.toast = None;
        return;
    }
    let message = message.clone();
    egui::Area::new("lab.toast".into()).anchor(Align2::CENTER_BOTTOM, [0.0, -56.0]).order(egui::Order::Tooltip).show(ctx, |ui| {
        egui::Frame::new().fill(RAISED).corner_radius(6).inner_margin(10).show(ui, |ui| {
            ui.label(crate::theme::widgets::serif_italic(message).color(INK));
        });
    });
}
```

`stage/mod.rs`:

```rust
//! The stage: CPU painters or the 3-D world through `StageRect`, plus overlay chrome.
use super::{Effects, LabData, state::LabState};
use crate::theme::tokens::BG;
use bevy_egui::egui::{self, Frame, Sense};
use engine_common::controls::camera::manager::stage::StageRect;
use engine_core::automata::channel::Backend;
pub mod overlay;
pub mod painters;
pub mod presenter;
pub mod view;

/// Draw the stage as the central panel and return the rectangle for stage cameras.
pub fn show(ctx: &egui::Context, lab: &mut LabState, data: &LabData, _fx: &mut Effects, typing: bool) -> StageRect {
    egui::CentralPanel::default().frame(Frame::NONE).show(ctx, |ui| paint(ui, lab, data, typing, true)).inner
}

/// Shared by workbench and presenter; `chrome` toggles overlay and caption.
pub(crate) fn paint(ui: &mut egui::Ui, lab: &mut LabState, data: &LabData, typing: bool, chrome: bool) -> StageRect {
    let rect = ui.max_rect();
    let response = ui.interact(rect, ui.id().with("stage"), Sense::click_and_drag());
    let row = data.channels.selected_row();
    let gpu = row.is_some_and(|r| r.backend == Backend::Gpu);
    let mut cursor = None;
    let mut zoom = 1.0;
    if !gpu {
        ui.painter().rect_filled(rect, 0.0, BG);
    }
    match row.and_then(|r| data.bench.sessions.iter().find(|s| s.channel == r.id)) {
        Some(session) => {
            let view = lab.views.entry(session.channel).or_default();
            if response.hovered() {
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll != 0.0 {
                    view.zoom_by((scroll / 200.0).exp());
                }
            }
            if response.dragged() {
                view.pan += response.drag_delta();
            }
            zoom = view.zoom;
            let fit = view::fit(painters::content_size(&session.model), rect, *view);
            painters::paint(&ui.painter_at(rect), &fit, &session.model, session.kind, session.settings.colour);
            cursor = response.hover_pos().and_then(|p| fit.to_cell(p));
        }
        None if row.is_none() => overlay::empty(ui.painter(), rect),
        None => {}
    }
    if chrome {
        overlay::chrome(ui.painter(), rect, data, cursor, zoom);
    }
    StageRect {
        managed: true,
        min: bevy::math::Vec2::new(rect.min.x, rect.min.y),
        size: bevy::math::Vec2::new(rect.width(), rect.height()),
        scale: ui.ctx().pixels_per_point(),
        hovered: response.hovered(),
        typing,
        world_visible: gpu,
    }
}
```

`stage/presenter.rs`:

```rust
//! Presenter mode: full-window stage, HUD readouts, serif title, fading transport.
use super::paint;
use crate::{lab::{Effects, LabData, state::LabState}, theme::{tokens::*, widgets::{mono, pill, serif, serif_italic, tick_digits}}};
use bevy_egui::egui::{self, Align2, Frame};
use engine_common::controls::camera::manager::stage::StageRect;
use engine_core::automata::channel::{ChannelAction, ChannelCommand, Status};

/// Draw presenter mode and return the full-window stage rectangle.
pub fn show(ctx: &egui::Context, lab: &mut LabState, data: &LabData, fx: &mut Effects, typing: bool) -> StageRect {
    let rect = egui::CentralPanel::default().frame(Frame::NONE).show(ctx, |ui| paint(ui, lab, data, typing, false)).inner;
    let Some(row) = data.channels.selected_row() else { return rect };
    egui::Area::new("lab.hud".into()).anchor(Align2::RIGHT_TOP, [-20.0, 16.0]).show(ctx, |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
            ui.label(mono(tick_digits(row.tick)).size(size::HUD));
            ui.label(serif_italic("tick"));
            if let Some(m) = data.metrics.get(row.id).and_then(|m| m.latest.map(|s| (m.id, s))) {
                ui.label(mono(crate::lab::inspector::sections::format_metric(m.1.value, &m.0)).size(26.0));
                ui.label(serif_italic(m.0.label));
            }
        });
    });
    if let Some(specimen) = data.specimens.get(&row.kind) {
        egui::Area::new("lab.title".into()).anchor(Align2::LEFT_BOTTOM, [22.0, -18.0]).show(ctx, |ui| {
            let n = data.specimens.figure(&row.kind).unwrap_or(0);
            ui.label(serif_italic(format!("Specimen {n} · {}", specimen.group.label())).size(size::BODY + 1.0));
            ui.label(serif(&specimen.title).size(24.0).color(INK_HI));
            ui.label(serif_italic(&specimen.caption));
        });
    }
    let alpha = lab.transport_alpha();
    if alpha > 0.0 {
        egui::Area::new("lab.mini".into()).anchor(Align2::RIGHT_BOTTOM, [-20.0, -18.0]).show(ctx, |ui| {
            ui.multiply_opacity(alpha * 0.8);
            ui.horizontal(|ui| {
                let running = row.status == Status::Running;
                let mut send = |action| fx.commands.push(ChannelCommand { id: row.id, action });
                if pill(ui, if running { "Pause" } else { "Run" }, true, row.status != Status::Halted).clicked() {
                    send(ChannelAction::Pause(running));
                }
                if pill(ui, "Step", false, row.status == Status::Paused).clicked() {
                    send(ChannelAction::Step);
                }
                if pill(ui, "Reset", false, true).clicked() {
                    send(ChannelAction::Reset);
                }
            });
        });
    }
    rect
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p ui stage`
Expected: 5 tests pass.

- [ ] **Step 6: Audit, lint, commit**

```bash
python3 tools/structure/audit.py && cargo fmt --all && cargo clippy -p ui --all-targets -- -D warnings
git add lib/ui/src/lab/stage
git commit -m "feat: paint CPU specimens on the stage with overlay caption and presenter mode

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 16: Switch to the lab and remove the legacy UI

**Files:**
- Modify: `lib/ui/src/plugin.rs` (`OutputPlugin` adds `ThemePlugin`, `lab::LabPlugin`, the trimmed `ComponentsPlugin`)
- Move: `lib/ui/src/presentation/panels/world/pause_menu.rs` → `lib/ui/src/components/menus/meta/pause_menu/mod.rs`, with its plugin registration moved into `components/menus/meta/plugin.rs`
- Delete: `lib/ui/src/presentation/` (camera overlay, minimap, panels), `lib/ui/src/components/banners/`, `lib/ui/src/components/menus/automata/`
- Modify: `lib/ui/src/lib.rs` (drop `presentation` and the `overlays`/`panels` re-exports), `lib/ui/src/components/{mod.rs,plugin.rs}`, `lib/ui/src/components/menus/{mod.rs,plugin.rs}`
- Modify: `src/app/preview/scene.rs` (`camera()` no longer spawns the `Text` label; the `label` argument is removed)
- Modify: `src/app/preview/automata/mod.rs`, `src/app/preview/lenia/mod.rs` (drop label strings)

**Interfaces:**
- Consumes: everything above.
- Produces: the running lab. `MinimapSelection` and `ZoomOverlay` no longer exist, so grep for and delete their remaining users.

- [ ] **Step 1: Write the failing test**

Add to `lib/ui/src/lab/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn legacy_ui_modules_are_gone() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        for gone in ["presentation", "styles", "components/banners", "components/menus/automata"] {
            assert!(!std::path::Path::new(root).join(gone).exists(), "{gone} should be removed");
        }
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p ui legacy_ui_modules_are_gone`
Expected: FAIL with `presentation should be removed`.

- [ ] **Step 3: Remove and rewire**

```bash
mkdir -p lib/ui/src/components/menus/meta/pause_menu
git mv lib/ui/src/presentation/panels/world/pause_menu.rs lib/ui/src/components/menus/meta/pause_menu/mod.rs
git rm -r lib/ui/src/presentation lib/ui/src/components/banners lib/ui/src/components/menus/automata
grep -rn "presentation::\|overlays::\|panels::\|banners\|automata::plugin::AutomataMenusPlugin\|MinimapSelection\|BannersPlugin\|OverlaysPlugin" lib src --include='*.rs'
```

Fix each hit. In `components/menus/plugin.rs`, add only `MetaMenusPlugin`. In `components/plugin.rs`, add `(DebugComponentsPlugin, MenusPlugin)`. In `meta/plugin.rs`, register the pause menu system: it was registered in `presentation/panels/world/plugin.rs`, so copy that `add_systems` line into a new `pause_menu/plugin.rs` with a `PauseMenuUiPlugin` and add it to the tuple.

`lib/ui/src/plugin.rs`:

```rust
//! Output-layer root: theme, lab shell, menus and debug overlays.
use crate::{components::plugin::ComponentsPlugin, lab::LabPlugin, theme::ThemePlugin};
use bevy::prelude::*;

/// Adds every output feature.
pub struct OutputPlugin;

impl Plugin for OutputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ThemePlugin, ComponentsPlugin, LabPlugin));
    }
}
```

`lib/ui/src/lib.rs`:

```rust
//! egui output layer: Instrument theme, automata lab shell, meta menus and debug overlays.
pub mod components;
pub mod lab;
pub mod plugin;
pub mod theme;
```

In `src/app/preview/scene.rs`, delete the `commands.spawn((Text::new(label), …))` block and the `label` parameter from `camera()`, and update its three callers. Leave the voxel-demo fixture's `info!` log in place.

- [ ] **Step 4: Run tests and the app**

Run: `cargo test -p ui && make automata`
Expected: the tests pass. The window opens at (80, 80) on monitor 1 with the rail, an open library drawer, empty tabs, the empty-stage prompt and the transport. Click Rule 30: the drawer closes, a tab appears, and the stage shows the amber space-time history. Press Space to run, `.` to step (while paused), P for presenter, Esc to return, and N to collapse the inspector. Save a screenshot of each state with `SOUL_SCREENSHOT` where supported, or with a window capture, for the PR.

- [ ] **Step 5: Audit, check, commit**

```bash
python3 tools/structure/audit.py && make check
git add -A lib/ui src/app
git commit -m "feat: replace floating automata windows with the lab shell

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 17: Qualification, evidence and documentation

**Files:**
- Modify: `src/app/preview/automata/flow/mod.rs` (channel-command pass per family)
- Modify: `src/app/preview/automata/capture.rs` and `src/app/preview/lenia/verification/capture.rs` (stage pixel check → `image-checks.json` rows)
- Create: `src/app/preview/automata/flow/stage_check.rs` (pure pixel counting; shared by both captures through `pub(in crate::app::preview)`)
- Modify: `src/app/preview/lenia/verification/flow/actions.rs` (channel-command pass after the existing lifecycle)
- Modify: `README.md`, `docs/kanban/v0-0-2.md`, `docs/verification/automata/README.md`, `docs/verification/lenia/README.md`
- Replace: `docs/verification/{automata,lenia}/{rtx3090,mesa}/*` (recaptured evidence), `docs/verification/*/image-checks.json`

**Interfaces:**
- Consumes: `Channels`, `ChannelCommand`, `StageRect`, `SIGNAL`.
- Produces:
  - `stage_check::count_signal(rgba: &[u8], width: u32, stage: (u32, u32, u32, u32), target: [u8; 3], tolerance: u8) -> u64`.
  - `stage_check::panels_clear(rgba: &[u8], width: u32, stage: …) -> bool`, which reports whether the 2 px band just inside the stage edge contains no panel-fill pixels.

- [ ] **Step 1: Write the failing tests**

`stage_check.rs`:

```rust
//! Screenshot assertions: specimen pixels inside the stage, and no panel over the stage.

#[cfg(test)]
mod tests {
    use super::*;

    fn image(w: u32, h: u32, fill: [u8; 3]) -> Vec<u8> {
        (0..w * h).flat_map(|_| [fill[0], fill[1], fill[2], 255]).collect()
    }

    #[test]
    fn counts_only_matching_pixels_inside_the_stage() {
        let mut rgba = image(10, 10, [13, 15, 16]);
        for (x, y) in [(2, 2), (3, 2), (9, 9)] {
            let i = ((y * 10 + x) * 4) as usize;
            rgba[i..i + 3].copy_from_slice(&[255, 181, 71]);
        }
        assert_eq!(count_signal(&rgba, 10, (0, 0, 5, 5), [255, 181, 71], 24), 2);
        assert_eq!(count_signal(&rgba, 10, (0, 0, 10, 10), [255, 181, 71], 24), 3);
    }

    #[test]
    fn detects_panel_fill_inside_the_stage_edge() {
        let rgba = image(10, 10, [13, 15, 16]);
        assert!(panels_clear(&rgba, 10, (1, 1, 8, 8)));
        let panel = image(10, 10, [18, 20, 21]);
        assert!(!panels_clear(&panel, 10, (1, 1, 8, 8)));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --bin swarm_orchestrator_for_autonomous_learners stage_check`
Expected: unresolved `count_signal`.

- [ ] **Step 3: Implement the checks**

Prepend to `stage_check.rs`:

```rust
/// Pixels within `tolerance` per channel of `target`, inside stage `(x, y, w, h)` of an RGBA8 image.
pub(in crate::app::preview) fn count_signal(rgba: &[u8], width: u32, stage: (u32, u32, u32, u32), target: [u8; 3], tolerance: u8) -> u64 {
    let (x0, y0, w, h) = stage;
    let mut count = 0;
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let i = ((y * width + x) * 4) as usize;
            let Some(px) = rgba.get(i..i + 3) else { continue };
            if px.iter().zip(target).all(|(a, b)| a.abs_diff(b) <= tolerance) {
                count += 1;
            }
        }
    }
    count
}

/// True when the 2 px band inside the stage edge has no panel-coloured (#121415) pixels.
pub(in crate::app::preview) fn panels_clear(rgba: &[u8], width: u32, stage: (u32, u32, u32, u32)) -> bool {
    let (x0, y0, w, h) = stage;
    let panel = [0x12, 0x14, 0x15];
    let band = |x: u32, y: u32| x < x0 + 2 || y < y0 + 2 || x >= x0 + w - 2 || y >= y0 + h - 2;
    (y0..y0 + h).all(|y| {
        (x0..x0 + w).filter(|x| band(*x, y)).all(|x| {
            let i = ((y * width + x) * 4) as usize;
            rgba.get(i..i + 3).is_none_or(|px| px != panel)
        })
    })
}
```

Wire it into both captures. In `capture::request`, before spawning the `Screenshot`, copy `world.resource::<StageRect>()` and convert it to physical pixels with `physical_viewport(&rect, window.physical_size())`. In the observer, `trigger.event().0` is the captured `Image`: convert it with `image.clone().try_into_dynamic().unwrap().to_rgba8()` (`bevy::image` exposes `try_into_dynamic`) and call `count_signal(…, [0xff, 0xb5, 0x47], 24)` and `panels_clear`. Append `{"image": "<name>.png", "width": W, "height": H, "stage": [x,y,w,h], "signal_pixels": N, "panels_clear": true}` to `<output>/image-checks.json`. In `finished`, assert `signal_pixels > 0` for populated captures, `== 0` for post-clear captures (the Lenia flow captures after clear), and `panels_clear`.

For **Lenia**, voxel shading multiplies the ramp by lighting, so exact amber pixels are rare. Count with tolerance 24 against `[0xff, 0xb5, 0x47]` **or** with the hue test `r > g && g > b && r > 80`. Implement it as a second predicate `count_warm(rgba, width, stage) -> u64` (same loop, predicate `px[0] > px[1] && px[1] > px[2] && px[0] > 80`) and use `count_warm` in the Lenia capture. Add a unit test: a pixel `[200, 120, 40]` counts, `[40, 120, 200]` does not.

- [ ] **Step 4: Add the channel-command passes**

In `automata/flow/mod.rs` phase 0, after the existing direct `session` assertions, exercise the same lifecycle through commands. Store the channel id in `Verification` (add a `channel: Option<ChannelId>` field) and run it over the next three frames as phases `0a`/`0b`, or as an extra `phase == 4` loop before capture. The checks:

```rust
            // Channel-command pass: identical lifecycle through the public contract.
            let id = world.resource::<Channels>().selected.expect("spawned channel is selected");
            world.send_event(ChannelCommand { id, action: ChannelAction::Reset });
            world.send_event(ChannelCommand { id, action: ChannelAction::Step });
            check.channel = Some(id);
            check.phase = 4;
        } else if check.phase == 4 {
            let row = world.resource::<Channels>().get(check.channel.unwrap()).cloned().expect("row");
            assert_eq!((row.tick, row.status), (1, Status::Paused), "channel step advanced once");
            // restore the deterministic capture state used by the evidence
            let mut bench = world.resource_mut::<Workbench>();
            let session = bench.sessions.first_mut().unwrap();
            session.reset().unwrap();
            for _ in 0..check.ticks { session.single_step(); }
            check.phase = 1;
            check.hold = check.frames;
```

Here `check.ticks` is the family's capture tick count, computed in phase 0 before this pass.

In the Lenia `actions.rs`, after the existing respawn verification, add the same pass: `Step` while paused (`target_tick + 1`), `SetRate(Rate::Hz10)` (row rate updates), `SetColour(SWATCHES[3])` (row colour updates), then `Remove`. After two frames, assert the row is gone and `ChannelIds::open() == 0`. Log `LENIA_CHANNEL_VERIFIED`.

- [ ] **Step 5: Run qualification on both drivers**

```bash
make verify-automata
make verify-lenia
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json SOUL_AUTOMATA_VERIFY=/tmp/soul-automata-mesa make verify-automata
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json SOUL_LENIA_VERIFY=/tmp/soul-lenia-mesa make verify-lenia
```

Expected: each run ends with `SUCCESS` in its output directory, `AUTOMATA_RUNTIME_VERIFIED: all 12 families passed`, the Lenia success line, and `LENIA_CHANNEL_VERIFIED`. Every `image-checks.json` row reports `panels_clear: true`.

- [ ] **Step 6: Replace the evidence and the docs**

```bash
cp /tmp/soul-automata-verification/*.png /tmp/soul-automata-verification/{sessions.csv,image-checks.json,runtime.log} docs/verification/automata/rtx3090/ 2>/dev/null
cp /tmp/soul-automata-mesa/*.png docs/verification/automata/mesa/
cp /tmp/soul-lenia-verification/*.png docs/verification/lenia/rtx3090/
cp /tmp/soul-lenia-mesa/*.png docs/verification/lenia/mesa/
```

Each Make target writes its log where its flow already does, so copy `runtime.log` from the same output directory.

`README.md`:
- replace the "Use Pause/Resume, Single step, Reset…" paragraph and the Automata-workbench paragraph with a **Lab** section covering:
  - the layout: rail, library drawer, channel tabs, stage, inspector, transport;
  - the shortcuts: Space run/pause, `.` step, R reset, 1–8 select, P presenter, N inspector, L library, Esc close/exit;
  - steps/s, per-channel colour, and opt-in Lenia mass sampling;
  - that every specimen cites its source with a DOI;
- keep the honesty paragraphs about Lenia and the Turing example;
- add before/after screenshots: `docs/verification/automata/rtx3090/life-conway.png` (new), and move the old image to `docs/verification/automata/before-lab.png` from git history with `git show HEAD~N:docs/verification/automata/rtx3090/life-conway.png > docs/verification/automata/before-lab.png`.

`docs/kanban/v0-0-2.md`: add a "Lab shell + visual language (sub-projects 1+2)" card under Done, linking the spec and plan, and add Backlog cards for sub-projects 3, 4+5 and 6.

Verification READMEs: replace "nonzero cyan model pixels" with "nonzero signal/warm pixels inside the stage rectangle, with no panel overlapping the stage", and list the new channel-command pass.

- [ ] **Step 7: Final check and commit**

```bash
python3 tools/structure/audit.py && make check
git add -A src docs README.md
git commit -m "test: qualify the lab through channel commands and stage-bounded image checks

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
