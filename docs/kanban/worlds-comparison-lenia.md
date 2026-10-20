# Editable worlds, quantitative comparison and continuous Lenia

Base: `38ea3c1` (Claude's merged lab shell). Implement on a separate worktree;
merge to local main after qualification. No remote push or release publication.

- Preserve the Instrument theme, unified channels, stage/inspector/drawers,
  presenter mode and existing command/metric contracts.
- Editable cellular worlds: dimensions 8..256, dead or periodic edges, paused
  paint/erase strokes, undo/redo, authored reset seeds, project migration and
  exact save/load continuation. Elementary histories and HPP retain their semantics.
- Quantitative comparison: channel selection across CPU/GPU, matched paused
  stepping with completed GPU readbacks, difference fields and error measures,
  epoch-aware metric histories and CSV exports. Mismatched domains are explicit.
- Continuous Lenia: a distinct f32 model with normalized radial kernels, configurable
  radius and shape, zero/periodic boundaries and genuine XYZ coupling. Preserve
  quantized XY Lenia. Verify GPU evolution against an independently implemented
  CPU oracle, including Z transport, reset, isolation and persistence.
- Review the redesigned UI before extending it; qualify additions in its native
  layout on monitor 1, windowed, without taking over monitor 2.

All 14 existing roots remain governed (0..3 child directories, 1..4 source files,
200 noncomment SLOC). Checkpoint at <=100 changed source lines. Completion requires
workspace checks, native RTX/Mesa verification, decoded exports, project migration,
comparison/continuation oracles, updated roadmap and a clean merged tree.

Completed at qualified source `51ef54f`: all three implementation items and the
Claude UI review/integration pass. [Final verification](../verification/worlds/README.md)
records workspace checks, native RTX/Mesa runs, exports, window placement and
commit/structural audits. The final delivery merges to local main; no remote publish.
