# Experiment workbench qualification

The four delivery items are implemented on `feature/experiment-workbench`, based
on `44ae1c3`, in a separate worktree. The qualified source revision is `1e4d128`; later commits record evidence only.

## Use

`make experiment` opens a 1760×960 window on monitor 1. Monitor 2 is untouched.

- Open **Experiment settings** in the automata workbench. Rule, speed and agent
  behavior edits apply live. Pattern, random seed and population edits require
  **Apply and reset**. Reset pauses and returns the session to tick zero.
- **Project → Pause all → Save project** saves the completed checkpoint. CPU
  clocks, grids/history, tapes, stacks, programs, agents, trails and RNG state are
  retained. Lenia saves actual GPU density bytes and the completed tick.
- **Load project** replaces the current experiment only after version, shape,
  settings and state validation. Writes use same-directory temporary files and
  atomic rename. Project files are limited to 16 MiB, eight CPU and eight GPU
  sessions; version 1 supports the existing 64×64×8 Lenia volume.
- **Compare and export** selects two CPU sessions, displays their measurements,
  and pauses/steps both together. A terminal session prevents the pair step.
  Existing GPU Lenia instances retain their independent native controls.
- Export the selected pair or a saved complete project, including Lenia, into a
  new folder. Each bundle contains `snapshot.png`, `animation.gif`, `project.json`
  and `manifest.json`. UI export uses 30 frames at 100 ms/frame and advances one
  transition per frame on isolated copies. Live sessions are unchanged.

Images use one 256-pixel panel per session. Elementary rules show time history;
Lenia uses maximum-density XY projection. Symbolic tape images show the first
256 cells; the full state remains in the project. Lenia GIF frames evolve the
qualified quantized CPU reference from the completed GPU checkpoint. This is
still independent XY-plane Lenia, not an isotropic 3D model.

## Automated verification

`make check` passes 50 native tests, 9 documentation tests and 20 governance tests,
plus formatting and strict all-target/all-feature Clippy. The ignored performance
benchmark runs separately. See [checks](checks.log), [structure](structure.json)
and [commit cadence](commit-cadence.json).

The same 14 source roots contain 330 governed source files, maximum 144 SLOC,
with no exceptions. Workbench evolution modules moved under `evolution` to keep
simulation ownership separate from experiment controls and project storage;
public imports remain available through deliberate re-exports.

## Sparse backend measurements

Run `SOUL_BENCH_REPORT=/absolute/path/report.json make benchmark-sparse`.
The [report](sparse-benchmark.json) uses release optimization, a fixed seed,
four Rayon workers, five samples and ten transitions per sample on an Intel
Core i9-7980XE. Shapes are 64×64 and 128×128, initial occupancies 1%, 10%, 50%.
All four backends finish with identical occupied-cell counts. Separate parity
tests compare every cell and memory value, including absent births, spontaneous
births, edge clipping and dead cells with retained memory.

Dense storage remains the workbench default: the finite-domain sparse backend
scans the full domain so rules that create cells from empty backgrounds remain
correct. Sparse is not faster in these measurements. Unbounded frontier stepping
requires a quiescent background; its neighbors include previously absent cells.
Payload estimates exclude map capacity, allocator overhead and temporary buffers;
they are not process RSS measurements or universal performance guarantees.

For the 128×128, 10% initial-occupancy case (ten ticks, median of five):

| Backend | Time |
| --- | ---: |
| dense | 13.81 ms |
| dense_parallel | 6.61 ms |
| sparse_bounded | 45.38 ms |
| sparse_bounded_parallel | 13.81 ms |

The final 609 occupied entries represent 34,104 bytes of sparse entry payload
versus 655,360 bytes of dense cell payload, before the excluded overheads.

## Native evidence

All runs passed on NVIDIA RTX 3090 (550.163.01, Vulkan) and Mesa llvmpipe
(25.0.7, LLVM 19.1.7, Vulkan). The experiment scenario edits Life/elementary
settings, steps a comparison pair, captures completed GPU tick 4 through the
production save path, loads the project, verifies exact restored density, and
checks GPU ticks 5 and 6 against the CPU reference. It then captures the window.
The scenario exercises production operations without injecting physical mouse
clicks or moving the pointer on monitor 2.

- Experiment: [RTX screenshot](rtx3090/experiment.png), [RTX log](rtx3090/runtime.log),
  [Mesa screenshot](mesa/experiment.png), [Mesa log](mesa/runtime.log).
- Family regressions: [RTX ledger](families-rtx3090/sessions.csv),
  [Mesa ledger](families-mesa/sessions.csv); all twelve examples pass on both.
- Lenia lifecycle: [RTX readbacks](lenia-rtx3090/readbacks.csv),
  [Mesa readbacks](lenia-mesa/readbacks.csv); every existing phase passes.
- [Export decoding and cross-renderer hashes](image-checks.json): 768×256 PNGs,
  eight GIF frames at 100 ms each, distinct successive frames, identical project
  and animation bytes across renderers. The native scenario uses eight frames;
  the interactive export control uses thirty.

Final runtime logs contain no errors. Mesa reports its expected software-renderer
warning. During development, LLVM crashed rebuilding dependency artifacts after
GIF was enabled on the shared image crate. GIF encoding now uses its own codec
dependency without changing renderer image features; the complete final build,
checks and native qualifications pass.
