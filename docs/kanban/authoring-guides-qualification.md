# Authoring, guided experiments, organism and release qualification

Baseline: local main `cdf6f59`. Worktree: `s-o-u-l-authoring`.

Authorized scope:
1. Named reusable specimen library with validated import/export, duplication and exact reopening.
2. Reproducible scenarios, guided experiment steps and presenter notes integrated into the Instrument UI.
3. A pinned published Lenia organism fixture, documented model, independent numerical checks and native display.
4. Representative Criterion benchmarks and Linux/Windows/macOS CI with build, test and runtime coverage explicitly recorded.
5. Replace letter tiles in simple automata with actual voxel entities, retaining symbol meaning in the inspector.

All existing first-party roots in `structural-governance.toml` remain governed.
New code follows its owning library: persistence/scenarios in models, UI workflows
and selected-channel voxel presentation in ui, Lenia semantics in simulation_kernel
and GPU shaders. Benchmark and verification tools remain governed first-party code.
Each source checkpoint is at most 100 changed lines. Preserve the unrelated README
edit in the original checkout. Preserve Claude's lab layout. Native checks use a
window on monitor 1 only. Local main is merged after verification; remote workflow
execution is recorded separately from local execution.

Acceptance: exact persisted starting states and invalid-import rejection; complete
scenario-to-comparison/export journey; numerical organism/reference agreement;
actual voxel entity lifecycle and visible native rendering; workspace/governance
checks; benchmark execution; portable CI definitions and available runner results.
