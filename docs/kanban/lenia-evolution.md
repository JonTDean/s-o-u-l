# Lenia evolution qualification

The next working item is a deterministic CPU reference, followed by GPU parity,
workbench controls, and native runtime qualification. Commits stop at 100 changed
source lines, with smaller cohesive checkpoints where practical.

## Contract

- Density is an integer in 0..255. Zero is dead.
- Each Z plane evolves independently using the existing 3×3 XY Gaussian weights.
  The finite kernel retains its existing weights without renormalization.
- Outside a slice is zero density; adjacent atlas allocations never contribute.
- Growth is `2 * exp(-(u-mu)^2 / (2*sigma^2)) - 1`.
- Update is `round(255 * clamp(density/255 + dt*growth, 0, 1))`.
- Parameters must be finite: mu in 0..1, sigma in 0.001..1, dt in 0..1.
- All cells read the previous tick. GPU state swaps after every tick, including
  multiple ticks in one render frame. Presentation reads the final state.
- Pause retains state; single-step advances exactly once while paused. Reset
  restores the selected deterministic seed and resets its tick count. Removal
  releases simulation state and prevents stale rendering on slot reuse.

This is the repository's quantized, local Lenia variant. It is not a claim that
the historical ring seed reproduces the published Orbium organism.

## Acceptance

- [ ] CPU: independent expected values, neighbors, boundaries, clamps, invalid
  parameters, deterministic multi-tick fixtures, and model/stepper agreement.
- [ ] GPU: production pipeline readback agrees with CPU on NVIDIA and Mesa for
  odd/even tick counts, edge cells, nonzero origins, independent slices, reset,
  parameter changes, and output packing/padding.
- [ ] Workbench: spawn, pause/resume, single-step, reset, parameter controls,
  and clear; deterministic tests exercise the same command handling as the UI.
- [ ] Runtime: visible evolving voxels in a window on monitor 1, screenshots,
  clean logs, complete `make check`, and an evidence-based roadmap update.
