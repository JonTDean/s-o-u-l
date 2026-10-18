# S.O.U.L. — Swarm Orchestrator for Autonomous Learners

A Bevy workbench for cellular automata and volumetric simulation experiments.
The current renderer uses portable GPU surface extraction and indirect drawing.
Lenia evolution and the optional ash/Vulkan mesh-shader backend remain unfinished.

## Run

Use the stable Rust toolchain, Python 3.11+, and native Bevy build dependencies.

```sh
cargo run
```

The normal application starts at the main menu. To inspect a deterministic
16 × 16 × 16 voxel volume without navigating the scenario menus:

```sh
make preview
```

The preview is a static rendering fixture. It opens a resizable 1280 × 720 window
at desktop position (80, 80), leaving the second display available. The Make
target uses X11/Xwayland so the compositor honors this placement. It exercises
atlas offsets, exposed-face extraction, world transforms, depth testing, and the
indirect draw path. It does not demonstrate automaton evolution.

To save an image of only the application window after the renderer records its
first ready voxel draw:

```sh
SOUL_SCREENSHOT=/tmp/soul-preview.png make preview
```

The portable backend is the default. The `engine-gpu/mesh_shaders` feature keeps
experimental Vulkan code available for development; it is not registered in the
portable render graph and has not qualified for runtime use.

## Source ownership

| Source root | Responsibility |
| --- | --- |
| `src/app` | Runtime configuration, plugin registration, scheduling, preview |
| `lib/simulation_kernel/src` | Rule interfaces, cell storage, CPU steppers |
| `lib/engine/engine-core/src` | Registries, world metadata, application state |
| `lib/engine/engine-common/src` | Scene flow, camera input and motion |
| `lib/engine/engine-gpu/src` | GPU storage, extraction, compute and draw stages |
| `lib/engine/engine-render/src` | Render orchestration and automaton commands |
| `lib/models/src` | Automaton families, rule registration, analytics |
| `lib/ui/src` | Menus, styling, panels and overlays |
| `lib/tooling/src` | Debugging, logging and monitoring |
| `lib/tests/src` | Cross-crate native behavior tests |
| `tools/structure` | Structural enforcement and its self-tests |

Modules have one responsibility and expose deliberate façades. Bevy plugins own
registration; systems communicate through resources, components and events.
Keep simulation logic separate from presentation and make schedule ordering
explicit where one system depends on another.

## Structural governance

`structural-governance.toml` defines every governed source root, including
Rust tests, WGSL shaders, PowerShell scripts, Python tooling and the checker.
Every governed directory must contain:

- Zero to three direct governed child directories.
- One to four direct source files, including `mod.rs` and export façades.
- No source file above 200 non-comment source lines.

Language-aware tokenization excludes blanks and comment-only lines. Code with
trailing comments counts once; comment markers inside strings remain code.
There are no source-path exceptions. Binary assets, dependency/build output,
repository metadata and data-only files are outside the source boundary.
The checker also reports tracked or new source files outside the declared roots.

Use semantic `snake_case` filenames and modules, `PascalCase` Rust types, and
`SCREAMING_SNAKE_CASE` constants. Group related modules by ownership rather than
adding artificial nesting. Preserve public import paths with meaningful
re-exports when moving implementations. Document public types and functions;
explain invariants, synchronization and unsafe prerequisites in comments.

## Verify and contribute

```sh
python3 -m pip install -r tools/structure/requirements.txt
make check
```

`make check` runs the governance self-tests and complete audit, Rust formatting,
workspace Clippy with all targets/features and warnings denied, and native tests.
The GPU tests require a Vulkan adapter; CI installs Mesa for software execution.
They use the production buffers, pipeline and bindings, and compare surface
counts with an independent CPU reference. They also test adjacency, transforms,
empty regions and bounded overflow.

Pick a task from `docs/kanban/v0-0-2.md`, keep its status current, and include
before/after screenshots or GIFs for rendering changes. Keep structural and behavioral changes
reviewable and preserve unrelated working-tree changes. CI runs `make check`.

## Verified rendering baseline

GPU-003 verifies a static volumetric preview and structural compliance.
The [before image](docs/verification/voxel-before.png) records the initial blank
preview capture; the [after image](docs/verification/voxel-after.png) shows the
ready renderer. Capture waits for the first voxel draw instead of assuming
shader compilation has finished. Owned scene/UI cameras use one sample;
multisample pipeline specialization remains future work.

Structural extraction groups camera motion and translation, engine execution,
GPU rendering/storage/setup/drawing, formal grammars, individuality analytics,
UI presentation, and tooling telemetry. Existing consumer import paths are
re-exported where implementations moved.

On 2026-10-18, `CARGO_BUILD_JOBS=2 CMAKE_BUILD_PARALLEL_LEVEL=2 make check` passed
20 governance self-tests, 9 native tests, and 9 documentation tests, with zero
structural violations across all 14 roots. The GPU checks also passed on Mesa's
software Vulkan backend. CI selects Python 3.13 and Rust 1.94.1 for consistency
with the verified local tools.
