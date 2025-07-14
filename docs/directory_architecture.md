```zsh
src/
├── main.rs                   (top‑level bootstrap)
├── app.rs                    (global App builder helpers)
│
├── dev_utils/                (already present; QoL tooling)
│   ├── tools/
│   │   └── quit.rs
│   ├── logging/
│   └── monitoring/
│
├── ca_engine/                ← Generic, dimension‑agnostic engine layer
│   ├── mod.rs
│   ├── core.rs               (traits & common ECS components)
│   ├── grid.rs               (dense & sparse grid back‑ends)
│   ├── stepper.rs            (parallel stepping helpers)
│   ├── render2d.rs           (default sprite & shader pass)
│   └── render3d.rs           (optional voxel view)
│
├── automata/                 ← Each “type” lives in its own sub‑tree
│   ├── mod.rs
│   ├── type1_elementary/     (1‑dim, Wolfram class, 2‑state)
│   │   ├── mod.rs
│   │   ├── rules.rs
│   │   └── plugin.rs
│   ├── type2_surface/        (2‑dim, Dean / Conway family, N‑state)
│   │   ├── mod.rs
│   │   ├── rules.rs
│   │   └── plugin.rs
│   └── type3_volume/         (3‑dim, Lenia / morphogenetic)
│       ├── mod.rs
│       ├── rules.rs
│       └── plugin.rs
│
├── ui/                       ← Visual‑design instrumentation
│   ├── mod.rs
│   ├── side_panel.rs         (egui sliders, dropdowns)
│   └── file_io.rs            (load/save .rle, .life, or custom)
│
└── assets/                   (optional textures, colour LUTs, icon)
```