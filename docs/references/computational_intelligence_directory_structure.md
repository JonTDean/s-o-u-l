```zsh
computational_intelligence/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── registry.rs               # dynamic rule‑set discovery & Bevy plugin
    │
    ├── classical/                # Chomsky‑aligned automata
    │   ├── type_3_regular/       # {3 :: 1 rule families …}
    │   │   ├── mod.rs
    │   │   ├── wolfram_1d.rs
    │   │   └── regex_nfa.rs
    │   ├── type_2_context_free/  # {2 :: 2 …}
    │   │   ├── mod.rs
    │   │   ├── ll1_pushdown.rs
    │   │   └── l_system.rs
    │   ├── type_1_context_sensitive/
    │   │   ├── mod.rs
    │   │   └── lba_examples.rs
    │   └── type_0_turing/
    │       ├── mod.rs
    │       └── universal_tm.rs
    │
    ├── dynamical/                # continuous & hybrid models
    │   ├── reservoir/
    │   │   ├── mod.rs
    │   │   ├── echo_state.rs
    │   │   └── liquid_state.rs
    │   ├── lenia/
    │   │   ├── mod.rs
    │   │   ├── kernel.rs
    │   │   ├── species/
    │   │   │   ├── mod.rs
    │   │   │   └── orbium.rs
    │   │   └── evolver.rs
    │   └── swarm/
    │       ├── mod.rs
    │       ├── boids.rs
    │       └── ant_colony.rs
    │
    ├── analytics/                # IIT / ITI / metrics
    │   ├── mod.rs
    │   ├── iit_phi.rs
    │   ├── iti_individuality.rs
    │   └── swarm_metrics.rs
    │
    ├── bridges/                  # glue to `engine_core`
    │   ├── mod.rs
    │   ├── cell_adapter.rs
    │   └── world_stepper.rs
    │
    └── prelude.rs                # re‑export common types & plugins
```