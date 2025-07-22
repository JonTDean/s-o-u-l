# S.O.U.L - Swarm Orchestrator for aUtonomous Learners

A **visual design tool**—not a game—for exploring and comparing multiple
families of cellular computational_intelligence (1‑D Elementary, 2‑D Surface, 3‑D Volume, …)
inside the Bevy engine.

| Layer | Purpose | Examples |
|-------|---------|----------|
| **`ca_engine/`** | Dimension‑agnostic core (grid storage, stepper, render bridges) | dense ↔ sparse back‑ends, parallel stepping |
| **`computational_intelligence/`** | Rule‑set plug‑ins grouped by *type* | Conway / Dean (Type 2), Lenia (Type 3) |
| **`ui/`** | Egui‑powered panels & file I/O | sliders, palette picker, load/save .rle |
| **`dev_utils/`** | QoL helpers | quit‑on‑Esc, FPS logging |

<img src="docs/demo.gif" width="640" alt="Dean's Game of Life demo">

---

## Features

* **Nested plugin graph**—drop‑in new rule sets without recompiling the engine.
* **Arbitrary state depth** (energy levels) and neighbourhood radii.
* **1‑D / 2‑D / 3‑D** views: quads, texture atlas, or low‑res voxels.
* **Hot‑reload parameters** while the simulation runs.
* **Export** to PNG sequence or GIF for presentations.

---

## Quick start

```zsh
# 1. Rust 1.79+ with stable tool‑chain
git clone https://github.com/your‑org/deans‑ca‑workbench.git
cd deans‑ca‑workbench
```

# 2. Native build

```zsh
cargo run --release
```

`Press Esc to quit; use the right‑hand egui panel to adjust rules.`

## Directory tree
```sh
src/
├── main.rs
├── app.rs
├── ca_engine/       # grid, stepper, render paths
├── computational_intelligence/        # type1_elementary / type2_surface / type3_volume
├── ui/              # egui panels + file IO
└── dev_utils/       # quit.rs, logging, monitoring
assets/              # textures, palettes, icons
docs/                # demo recordings, architecture sketches
```

## Configuration

| Resource                     | Default | Meaning                                  |
| ---------------------------- | ------- | ---------------------------------------- |
| `SimParams.complexity_level` | `3`     | Max energy (living strength)             |
| `SimParams.neighbour_radius` | `1`     | Manhattan radius; 1 = Moore‑8            |
| `SimParams.memory_depth`     | `4`     | Ticks a cell decays through dying states |

## Roadmap

| Phase  | Focus                                                      |
| ------ | ---------------------------------------------------------- |
| **P0** | Engine skeleton compiles; blank grid renders               |
| **P1** | Elementary (1‑D) rules & tape visualiser                   |
| **P2** | Dean/Conway suite, parameter UI, GIF export                |
| **P3** | 3‑D Lenia prototype, basic voxel renderer                  |
| **P4** | Layer compositing, side‑by‑side diffing, project save/load |

## Contributing

1. Pick a task from the Kanban board (KANBAN.md) in Backlog and move it to In Progress in your PR.
2. Run cargo clippy --all-targets --all-features -- -D warnings.
3. Document public types and functions.
4. Include before/after screenshots or GIFs if you touch rendering.