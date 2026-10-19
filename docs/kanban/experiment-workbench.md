# Experiment workbench delivery

Branch: `feature/experiment-workbench`, isolated from the user's working tree.
Base: `44ae1c3`. Merge to local `main` only after all four gates pass.

1. Validated per-session speed, cellular rules/seeds, flocking and ant parameters.
   Runtime parameters apply without reset; initial conditions require explicit reset.
2. Versioned project files preserve CPU state, RNG, parameters, ticks and GPU Lenia
   snapshots. Loading validates before replacing live state; malformed files leave
   the current experiment intact. Continued execution must match uninterrupted runs.
3. Two-session comparison, synchronized stepping, metrics, PNG and animated GIF
   exports with project/settings metadata. Exports must decode and preserve frames.
4. Sparse frontier births, dead-cell pruning with memory retention, finite-domain
   handling, serial/parallel/dense parity and measured reproducible benchmarks.

All 14 source roots remain governed. Each source checkpoint changes at most 100
nonblank, non-comment source lines; moved modules retain public re-exports.
Verification includes `make check`, native controls and captures on monitor 1,
project/export round trips, sparse parity and benchmarks. Existing family and
Lenia qualifications remain regression gates. Preserve the original checkout's
unrelated `.gitignore` edit and do not push without an explicit request.

## Completed

All four items passed workspace checks and native qualification on RTX 3090 and
Mesa at source revision `1e4d128`. See
[the delivery evidence](../verification/experiments/README.md) for project/export
round trips, GPU continuation, sparse measurements and source governance.
Dense remains the default based on the measured finite-domain results.
