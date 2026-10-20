# Authored worlds, quantitative comparison and continuous Lenia

This work starts from the merged Claude lab shell at `38ea3c1` and preserves its
rail, drawers, stage, inspector, transport and presenter. Implementation lives in
`feature/worlds-comparison-lenia`, in an isolated worktree. Native previews stay
windowed on monitor 1; qualification never moves the physical pointer.

## Using the lab

- For Life, elementary rules and HPP, set width/height (8–256) and dead/periodic
  boundaries in the inspector, then **Apply and reset**. Elementary worlds have
  one evolving row and retain up to 64 history rows. Resizing replaces the seed.
- Pause a cellular channel, then choose **paint** or **erase** in **author**.
  Dragging fills crossed cells. Each stroke is one undo item; **Undo/Redo** keeps
  16 strokes until reset/load. HPP paints direction bits 1–15. Panning uses **pan**;
  zoom remains available. Painting starts a new tick-zero epoch and makes the
  edited field the Reset seed. Rule/preset changes remain explicit inspector edits.
- **Project → Pause all → Save project** saves exact current state and the authored
  reset seed. Loading validates before replacement. Version 2 adds world settings
  and optional continuous Lenia state; version 1 projects migrate to 64-cell dead
  boundaries and the original quantized Lenia model. Capacity is eight total channels.
- **Compare** accepts CPU and GPU channels. Pause both, then capture or step both.
  A capture waits for the requested completed ticks and cancels on reset, parameter
  change, removal, resume or timeout. Labels retain each channel's epoch and tick;
  different absolute ticks are explicitly flagged. Compatible cellular fields show
  MAE, RMS, maximum error, changed-cell count and an XY maximum absolute difference
  projection over every Z plane. HPP uses direction-bit units, not density units.
  Noncellular families retain metric traces and stepping, without a field heatmap.
- Metric plots use actual ticks and retain the latest 200 samples of the current
  epoch. GPU mass sampling is opt-in. **Export metrics and difference CSV** writes
  a new folder with epochs, ticks, channel kinds, units and exact captured fields.
  Comparison observation does not consume project-save or metric readbacks.
- Lenia's **model** inspector selects **quantized XY** or **continuous XYZ**.
  Model/kernel changes reset. Continuous controls expose radius 1–4, Gaussian or
  shell kernels, and dead/periodic boundaries. Growth controls apply live.

## Model contract

The original quantized model remains unchanged: u8 density, independent XY planes,
finite unnormalized 3×3 kernel, zero edges and explicit half-up quantization.

Continuous XYZ v1 stores f32 densities in `[0,1]` in a 64×64×8 volume. Every offset
inside the Euclidean radius contributes a radial Gaussian weight. Gaussian has
normalized-radius mean 0 and width 0.5; shell has mean 0.65 and width 0.2. Weights
are normalized over the full discrete kernel. Dead boundaries supply zero;
periodic boundaries wrap each axis. Growth is
`2 exp(-0.5 ((potential - mu) / sigma)^2) - 1`; clipped Euler integration uses `dt`.
All three axes couple. Only rendering converts to bytes. Checkpoints retain exact
float values and model settings; animation exports evolve an isolated CPU reference.

This is an explicitly defined finite-lattice extension of the convolution/growth
construction in [Chan's Lenia paper](https://arxiv.org/abs/1812.05433). The supplied
blob/ring fixtures are not claims to reproduce published organisms. The independent
CPU oracle uses direct f64 convolution and f32 state; GPU agreement is measured with
a maximum absolute error below `1e-5`, while serialized restoration is exact.

## Reproduction

`make check` runs governance, formatting, strict Clippy, workspace tests and doctests.
`SOUL_WORLDS_VERIFY=/new/output/folder make experiment` runs the native end-to-end
scenario. Repeat with `VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json` for Mesa.
The scenario exercises authored periodic worlds, undo/redo, completed CPU and GPU
comparison, CSV export, exact float checkpoint/load, CPU-oracle continuation,
PNG/GIF export and screenshots through production systems.

Qualification results and artifact links are recorded after the final runs below.
