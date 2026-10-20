# Lenia qualification — 2026-10-18

All four acceptance gates in the [contract](../../kanban/lenia-evolution.md)
passed. The working item is a quantized Lenia workbench with deterministic CPU
reference, GPU evolution, controls, and verified native lifecycle behavior.
The qualified source revision is `c75bf14`; subsequent documentation records
these results and adds `make lenia` / `make verify-lenia` shortcuts.

## Scope and ownership

Density uses u8 values, the existing unnormalized finite 3×3 Gaussian kernel,
zero boundaries, and independent XY planes at each Z. This is not qualification
of isotropic 3-D Lenia, a published Orbium organism, or the optional ash backend.
The legacy `lenia:orbium` model ID remains compatible and is displayed as Ring.

- `simulation_kernel/core/lenia`: validated parameters, bounded seeds, snapshot CPU reference.
- `models/lenia`: model adapter using the shared kernel and corrected neighbor indices.
- `engine-core/automata/lenia`: per-instance commands, clock, pause, reset epochs.
- `engine-gpu/render/compute/lenia`: pipelines, state swaps, packing and readback.
- `engine-render`: spawn ownership, clear/despawn and atlas reclamation.
- `ui/components/menus/automata/lenia`: workbench controls.
- `src/app/preview/lenia/verification`: native lifecycle and capture qualification.

Meaningful module façades preserve consumers. The same 14 source roots remain
governed with no exceptions: 261 files, maximum 144 non-comment source lines.

## Reproduce

```sh
CARGO_BUILD_JOBS=2 CMAKE_BUILD_PARALLEL_LEVEL=2 make check
SOUL_LENIA_VERIFY=/tmp/lenia-rtx make verify-lenia
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json \
  SOUL_LENIA_VERIFY=/tmp/lenia-mesa make verify-lenia
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json \
  cargo test --workspace --all-features --lib -- --nocapture
```

The Make targets force Vulkan and X11/Xwayland for window placement. Both saved
qualification runs executed the same built native binary with these settings.
The verifier deletes stale success/capture files and writes `SUCCESS` only when
all phases and fresh captures finish. Normal `make lenia` stays open, initially
paused. KWin may reposition a second window; confirm it remains within monitor 1.

## Test evidence

[Final make check log](checks.log): 20 governance tests, 25 native Rust tests,
9 documentation tests, formatting and strict all-target/all-feature Clippy pass.
[Software Vulkan native tests](mesa-tests.log) also pass all 25 native tests.
No hosted CI run is claimed.

CPU tests include an independent f64 oracle, boundaries, invalid parameters,
clamps, deterministic multi-tick fixtures and serial/parallel stepper agreement.
Production GPU tests cover odd/even tick batches, snapshot swapping, nonzero
atlas origins, 257-wide padded rows, separate slices and parameters, reset,
removal/entity reuse, bounds and low-density rendering.

[Commit cadence](commit-cadence.json) audits 45 implementation checkpoints after
`356184e`: maximum 98 added/deleted nonblank, non-comment source lines per commit.
The following evidence/documentation commit adds no governed source lines.

## Native workbench results

| Adapter | Driver | Evidence |
| --- | --- | --- |
| NVIDIA GeForce RTX 3090 | NVIDIA 550.163.01, Vulkan | [Log](rtx3090/runtime.log), [readbacks](rtx3090/readbacks.csv), [success](rtx3090/SUCCESS) |
| llvmpipe, LLVM 19.1.7 | Mesa 25.0.7-2+deb13u1, Vulkan | [Log](mesa/runtime.log), [readbacks](mesa/readbacks.csv), [success](mesa/SUCCESS) |

Both runs have no runtime errors. Mesa emits the expected software-renderer
performance warning. The native harness sends the same commands as the visible
UI; this verifies handlers and rendered controls, not physical mouse clicking.
Every nonempty phase compares the full GPU density array with the CPU reference;
CSV sums/checksums provide compact evidence rather than replacing that comparison.

| Phase | Action and observed state |
| --- | --- |
| 0 | Spawn Blob, epoch 0 / tick 0, 2,548 live cells |
| 1 | Single step while paused, tick 1 |
| 2 | Hold paused for 30 frames; tick 1 and densities unchanged |
| 3 | Set mu=.27, sigma=.08, dt=.02; single step to tick 2 |
| 4 | Reset Blob to epoch 1 / tick 0; parameters retained |
| 5 | Select Ring, epoch 2 / tick 0, 1,656 live cells |
| 6 | Resume then pause; tick 9, 1,856 live cells |
| 7 | Reset Ring, epoch 3 / tick 0 |
| 8 | Clear: zero ECS owners, zero GPU slots, blank model area |
| 9 | Respawn Ring: fresh owner, epoch 0 / tick 0, default parameters |

The resume phase may reach a different tick with different frame timing; the
reference advances to the exact reported tick. Both saved runs reached tick 9
and produced identical CSV records.

## Visible output

All captures are 1280×720 application-window images of the lab. Both adapters'
captures were inspected; [image measurements](image-checks.json) record, inside
the published stage rectangle, nonzero signal/warm (shaded amber) pixels for every
populated capture and zero after clear, with no panel overlapping the stage
(`panels_clear`).

After the respawn phase a **channel-command pass** drives the same instance through
the public `ChannelCommand` contract: Step while paused advances the target tick by
one, SetRate(10) and SetColour update the published row, and Remove despawns the
owner and releases its id (`ChannelIds::open() == 0`). The run logs
`LENIA_CHANNEL_VERIFIED` before success.

| State | RTX 3090 | Mesa |
| --- | --- | --- |
| Initial blob | [Image](rtx3090/lenia-phase-0.png) | [Image](mesa/lenia-phase-0.png) |
| Edited growth, tick 2 | [Image](rtx3090/lenia-phase-3.png) | [Image](mesa/lenia-phase-3.png) |
| Evolved ring, tick 9 | [Image](rtx3090/lenia-phase-6.png) | [Image](mesa/lenia-phase-6.png) |
| Reset ring | [Image](rtx3090/lenia-phase-7.png) | [Image](mesa/lenia-phase-7.png) |
| Clear | [Image](rtx3090/lenia-phase-8.png) | [Image](mesa/lenia-phase-8.png) |
| Respawn | [Image](rtx3090/lenia-phase-9.png) | [Image](mesa/lenia-phase-9.png) |

Final interactive workbench placement: client (81,116), 1280×720, windowed on
HDMI-A-1 (monitor 1, 1920×1080). DP-1 (monitor 2) remains 2560×1440 with HDR enabled.
