# Authoring, guided experiments, voxel machines and Orbium

Verified on 2026-10-20. Baseline local main: `cdf6f59`.
Native source checkpoint: `c21b634`; product source is unchanged by subsequent
verification documentation. All 15 roots in `structural-governance.toml` retain
0–3 direct children, 1–4 direct source files and at most 200 non-comment SLOC,
without exemptions. [Commit cadence](commit-cadence.json) records source checkpoints.

## Delivered behavior

- A named library stores exactly one paused CPU/GPU channel using the validated
  project schema. Save preserves completed state, settings and tick; reopening
  replaces the current workspace. Duplication and import/export reject invalid
  projects and existing destinations. Portable names exclude reserved filenames.
  `SOUL_LIBRARY` overrides the platform data directory's `soul/specimens` folder.
- Three deterministic paired experiments cover elementary rules, Life boundaries,
  and accepted/rejected pushdown input. Each has four prediction/stepping/comparison/
  export cues and presenter support. Claude's existing lab layout is retained.
- Symbolic machine tapes and stacks are actual Bevy `Mesh3d` voxel entities with
  a white head cap and inspector colour key. The selected channel owns the visible
  entities; camera framing and cleanup follow selection. Comparison and exported
  images use corresponding shaded voxel cells. Koch retains its geometric curve.
- Published Orbium uses Chan's original cells and polynomial periodic XY model,
  radius 13, mu 0.15, sigma 0.014, dt 0.1. This model is distinct from the existing
  quantized XY and Gaussian/shell XYZ variants. [Fixture provenance and license](../../../assets/specimens/README.md)
  pin the upstream notebook. `scripts/orbium_reference.py` independently generates
  the NumPy FFT reference; its 1,000-tick mass/centroid ledger documents survival.
- Criterion covers 24 dense/sparse serial/parallel Life combinations and three
  Lenia reference steppers. [Estimates](benchmark-estimates.json) retain 27 cases,
  confidence intervals, four-thread setting and hardware. The [hosted baseline](hosted-benchmark-estimates.json)
retains all 27 cases from the final-source Ubuntu run. Grid setup is excluded
  from Life step timings; Lenia measures its allocating reference-step API.

## Local acceptance

`make check` passes formatting, strict all-target/all-feature Clippy, 172 native
Rust tests, nine doctests and 20 governance tests. One existing manual benchmark
remains intentionally ignored. `cargo build --workspace --all-features` succeeds.
See [checks](checks.log) and [build](build.log).

The production-command native harness (`make verify-authoring`) passes on both
[NVIDIA RTX 3090](rtx3090/SUCCESS) and [Mesa llvmpipe](mesa/SUCCESS):

1. Load the boundary scenario, synchronize ticks and export the difference CSV.
2. Capture a Life specimen, duplicate/export it and reopen its exact saved state.
3. Load the language experiment, show its guide and verify live voxel entities.
4. Spawn Orbium, save exact completed GPU floats, duplicate/export, reopen and
   advance ten ticks against the independent FFT reference.

[Artifact validation](artifact-checks.json) confirms 256 comparison rows per run,
identical specimen copies, cross-renderer identical Life projects, decoded PNGs
and two-frame GIFs with 100 ms frame times. Native Orbium maximum absolute error
at tick 10 is 0.0000051095 on RTX and 0.0000057949 on Mesa (limit 0.002).
Unit tests also compare CPU and GPU states through tick 100.

Captured windows were 1760×960 at (0,0) on RTX and (81,116) on Mesa, inside monitor 1's 1920×1080
extent. Monitor 2 was reserved. [Library](rtx3090/authoring-library.png),
[voxel machines](rtx3090/symbolic-voxels.png), and
[published Orbium](rtx3090/published-orbium.png) were visually inspected.
[All twelve family regressions](families/SUCCESS) and the
[existing Lenia lifecycle](legacy-lenia/SUCCESS) pass again on RTX.

## Hosted qualification

The platform workflow runs formatting/governance, strict portable compilation,
CPU/persistence tests, Linux Vulkan oracle tests, native builds and screenshot-backed
window boot checks on Ubuntu 24.04, Windows 2022 and macOS 15. It also uploads
Criterion results. macOS exposed an old shaderc CMake minimum incompatibility;
`CMAKE_POLICY_VERSION_MINIMUM=3.5` explicitly opts into compatible CMake 4 policy.
The experimental Vulkan interop module is now compiled only on native Vulkan
targets supported by wgpu; macOS uses the portable Metal renderer.
Runner results and exact commits are recorded in `hosted.json` when complete.
A workflow definition alone does not establish platform runtime qualification.

## Limits and observed retries

This delivery adds named specimens and existing-family parameter editing, not a
new user-authored rule language. The three guides are bounded authored experiments.
Orbium qualification covers the pinned model and seed, not every Lenia organism.
Benchmark numbers are local/hosted baselines, not stable release performance budgets.
The experimental ash mesh-shader backend remains outside portable qualification.

The local workstation had intermittent failures: an old cached bevy_egui archive
failed linking (targeted package clean and rebuild passed), one native process and
one benchmark attempt exited with SIGSEGV, and one Clippy startup reported allocator
corruption. No cause was established. A debugger run, repeated native runs, a focused
benchmark repeat, the complete benchmark rerun and the complete check rerun passed.
These retries are not evidence of a diagnosed fix. Final logs retain successful
runs; normal ALSA default-device startup warnings remain unrelated to the render gate.
