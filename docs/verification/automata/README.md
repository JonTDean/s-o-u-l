# Automata family qualification — 2026-10-19

All twelve non-Lenia examples in the existing family tree are runnable and
qualified. The [contracts](../../kanban/automata-families.md) define their scope.
Source revision: `2cb81c3` (subsequent changes record evidence and documentation).

## Run

```sh
make automata
CARGO_BUILD_JOBS=2 CMAKE_BUILD_PARALLEL_LEVEL=2 make check
SOUL_AUTOMATA_VERIFY=/tmp/families-rtx make verify-automata
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json \
  SOUL_AUTOMATA_VERIFY=/tmp/families-mesa make verify-automata
SOUL_LENIA_VERIFY=/tmp/lenia-regression make verify-lenia
```

The interactive lab opens windowed at 1280×720 with Rule 30 paused at tick 32.
Choose another family in the spawner. The active list and session selector switch
between independent instances. Resume, single-step, reset and remove operate on
the selected instance; Clear all releases both CPU and Lenia instances.

## Qualification

[Workspace checks](checks.log) pass 41 native Rust tests, 9 documentation tests,
20 governance tests, formatting and strict all-target/all-feature Clippy.
The same 14 governed roots contain 287 source files with no exceptions;
the largest file is 144 non-comment source lines.

[Commit cadence](commit-cadence.json) audits implementation checkpoints after
`84cf014`; every checkpoint is below 100 added/deleted nonblank, non-comment
source lines. The final evidence commit changes documentation only.

Native capture runs passed on NVIDIA RTX 3090 (550.163.01, Vulkan) and llvmpipe
(Mesa 25.0.7, LLVM 19.1.7, Vulkan). Each family was spawned through the production
command route, single-stepped while paused, reset, advanced to its fixture state,
held paused for 30 frames, captured, and cleared before the next instance.
The harness rejects stale success markers and waits for each screenshot write.
This exercises the UI's command and session operations; physical mouse clicks
were not injected.

- NVIDIA: [runtime log](rtx3090/runtime.log), [session ledger](rtx3090/sessions.csv), [success](rtx3090/SUCCESS).
- Mesa: [runtime log](mesa/runtime.log), [session ledger](mesa/sessions.csv), [success](mesa/SUCCESS).
- Existing Lenia: [regression log](lenia-regression.log), [readbacks](lenia-readbacks.csv). All lifecycle phases still match the CPU reference.

There were no errors in the final native runs. Mesa reports its expected
software-renderer performance warning. A Rust incremental-metadata compiler
panic during development was resolved by clearing only the models package build
artifacts; a fresh complete check passed. No hosted CI run or push is claimed.

## Behavioral evidence

| Model | Numerical or state oracle | Native screenshot |
| --- | --- | --- |
| Rule 30 | All eight neighborhood outputs; serial/parallel agreement | [NVIDIA](rtx3090/wolfram-rule30.png) / [Mesa](mesa/wolfram-rule30.png) |
| Rule 110 | All eight outputs, including repaired 011/100 entries | [NVIDIA](rtx3090/wolfram-rule110.png) / [Mesa](mesa/wolfram-rule110.png) |
| Conway Life | Blinker period 2 and glider displacement after 4 ticks | [NVIDIA](rtx3090/life-conway.png) / [Mesa](mesa/life-conway.png) |
| HPP | Four direction propagations; all 16 collision combinations and particle counts | [NVIDIA](rtx3090/particle-hpp.png) / [Mesa](mesa/particle-hpp.png) |
| NFA | Exhaustive short words, epsilon cycles and nondeterministic closure | [NVIDIA](rtx3090/regular-nfa.png) / [Mesa](mesa/regular-nfa.png) |
| a^n b^n | Exhaustive words through length 6; ordering and empty-input semantics | [NVIDIA](rtx3090/context-free-anbn.png) / [Mesa](mesa/context-free-anbn.png) |
| Parentheses | Nested, empty, malformed and invalid-symbol inputs | [NVIDIA](rtx3090/context-free-parentheses.png) / [Mesa](mesa/context-free-parentheses.png) |
| Koch | Parallel production, 4^n forward symbols, allocation bound | [NVIDIA](rtx3090/context-free-koch.png) / [Mesa](mesa/context-free-koch.png) |
| LBA | Exhaustive ordered-language cases, Unicode rejection, invariant tape length | [NVIDIA](rtx3090/context-sensitive-anbncn.png) / [Mesa](mesa/context-sensitive-anbncn.png) |
| Turing | Rewrite/accept/reject and both tape-extension boundaries | [NVIDIA](rtx3090/turing-replace-a.png) / [Mesa](mesa/turing-replace-a.png) |
| Boids | Order-independent update, finite positions, speed and arena bounds | [NVIDIA](rtx3090/swarm-boids.png) / [Mesa](mesa/swarm-boids.png) |
| Ants | Reproducible 500-tick replay, food delivery, evaporation and bounded trails | [NVIDIA](rtx3090/swarm-ants.png) / [Mesa](mesa/swarm-ants.png) |

Cross-crate tests additionally verify every family plugin's registration,
production spawn/clear events, pause/resume/single-step/reset, stable terminal
states, capacity rejection and session isolation. All cellular models match
serial and parallel steppers for 80 ticks. Screenshots were visually inspected;
[image checks](image-checks.json) record dimensions and visible canvas content.

## Ownership and boundaries

The new `models/workbench` modules own catalog metadata, bounded sessions and
CPU scheduling; grid, symbolic and agent implementations have separate owners.
Each existing family plugin registers its examples. The UI's
`active_menu/workbench` modules draw controls, grids, tapes and turtle geometry.
`src/app/preview/automata` owns native qualification and application-only captures.
No governed root was added, excluded or exempted.

The new simulations run on the CPU with egui presentation; Lenia retains its
qualified GPU pipeline. Symbolic examples demonstrate their named language or
transition table, not arbitrary regex compilation, arbitrary LL(1) parsing or a
universal Turing machine. Dean's custom multi-energy rule needs its transition
specification; isotropic Lenia and published Orbium remain separate roadmap work.

Elementary rule numbering follows [Wolfram's original description](https://www.wolframscience.com/nks/p676--the-rule-110-cellular-automaton/).
The flock uses the three steering terms described by [Craig Reynolds](https://www.red3d.com/cwr/boids/).
