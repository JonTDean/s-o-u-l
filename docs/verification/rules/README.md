# Authored rules, composition and two-pool energy

Verified 2026-10-20, baseline main `2f87e8e`, source checkpoint `1d791ab`.
Implementation in `feature/rules-composition`; subsequent changes are evidence,
usage documentation and task aliases. [User guide](../../guides/authored-rules.md)
and [contract](../../kanban/rules-composition.md) describe the exact scope.

## Completed work

- Versioned, bounded declarative binary 3×3 rules with ordered masked transitions,
  validation, inspector text editing, presets, and non-replacing JSON interchange.
  All 512 binary neighborhoods can be specified, including directional rules.
- Two rule layers share a prior grid and clock; union, intersection or XOR combine
  outcomes before the next tick. Definitions persist in projects/specimens and
  comparison provenance. Independent channels remain available for comparison.
- A configurable two-pool starting model with explicit integer diffusion, decay,
  recharge, capacities and birth/survival costs. State and settings persist exactly;
  invalid state is rejected, parameter changes require reset, painting resets
  pools, and undo/redo restores them. Inspector exposes parameters and totals.
- The optional separate sprite renderer is deferred in favor of the existing
  voxel presentation. The roadmap decision is complete; no sprite backend was
  implemented. Existing cellular analysis views are retained.

## Verification

`make check`: **184 native tests, 9 doctests, 20 governance tests**, formatting
and strict Clippy pass. The existing manually invoked benchmark remains ignored
in the ordinary test run. `cargo build --workspace --all-features` passes.
See [checks](checks.log) and [build](build.log).

All 15 governed roots comply with <=200 non-comment SLOC/file, 1–4 direct
source files and 0–3 direct child directories. No roots or limits were relaxed.
[Governance](governance.json) and [checkpoint cadence](commit-cadence.json) record
this boundary. Source checkpoints remain within 100 changed source lines.
New ownership modules live under cellular configuration/evolution, inspector
parameters, project tests and native acceptance fixtures; public entry points
are preserved. The original checkout's unrelated README edit is untouched.

Native `SOUL_RULES_VERIFY` journeys pass on RTX 3090/NVIDIA 550.163.01 and
Mesa lavapipe 25.0.7/LLVM 19.1.7, both Vulkan. They drive normal configure,
step, comparison and file requests; check exact four-tick states; export portable
rules, named specimens, PNG and three-frame GIF; restore the project; and verify
one more command-driven tick against its saved continuation. Screenshots were
visually inspected. Both windows were 1760×960 at (+81,+116), entirely on monitor 1.
See [RTX](rtx/SUCCESS), [Mesa](mesa/SUCCESS), window-bounds JSON, native logs,
and [artifact checks](artifact-checks.json). CPU checkpoints and exports are
identical across the two renderers.

## Scope and limits

The new rules simulate on CPU and are binary, radius-one XY programs. Composition
uses coupled feedback, not independent layer histories. Energy is an explicitly
documented editable starting model, not a reproduction of an unspecified prior
Dean rule. Stage/comparison images measure occupancy; both energy fields remain
in checkpoint JSON and their totals appear in the inspector. Rule-only files do
not contain world/energy settings; use projects or specimens to share those.

This source revision was built and exercised locally on Linux. Prior hosted
Windows/macOS qualifications belong to the previous source and are not claimed
as qualification of these new changes. Broader custom-GPU and experimental ash
renderer work remains outside this milestone.

## Main integration

Local main fast-forwarded from `2f87e8e` to `2025c2c`. A fresh build from the main
worktree passed, followed by `--rules-demo` with the native boot gate. Three live
channels published, the screenshot completed, and the application exited cleanly.
[Boot](main-boot/SUCCESS), [capture](main-boot/boot.png), [window bounds](main-boot/window-bounds.json)
and [build log](main-build.log) record the result. The final evidence commit
changes documentation only; implementation remains identical to `1d791ab`.
All 29 pre-integration commits satisfy the checkpoint cadence (maximum 99 SLOC).
