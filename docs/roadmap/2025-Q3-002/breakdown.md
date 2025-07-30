The reference architecture described in your **state‑flow** and **Bevy plugin** diagrams is built around a modular, plug‑in based design: the main `App` wires together core engine logic, UI, dev tools and optional network features.  Bevy’s plug‑in system deliberately encourages separating subsystems into distinct plug‑ins and even grouping them into plug‑in‑groups.  Features such as conditional compilation via `Cargo` features allow you to toggle code at build time.  These ideas underpin the three road‑maps (rendering, automata and GPU compute) and their kanban boards.

## Linking the road‑maps to the architecture

### Active‑Automata render pipeline (10 phases)

Each phase in the rendering road‑map introduces or modifies modules in the `io/output` tree.  The table below maps the phases to the principal modules touched in your directory architecture (short phrases avoid long descriptions):

| Phase | Code‑name            | Key architecture modules                                                                                                                                                    |
| ----- | -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **0** | Boot‑strap           | Adds `io/output/rendering/active/material.rs` & `plugin.rs` as a new renderer plug‑in; toggled via `active-render` feature (cargo features can enable/disable code).        |
| **1** | Dense Path           | Adds `upload.rs` with a dense buffer upload system; unit test lives in the same module.                                                                                     |
| **2** | Sparse Path          | Extends `upload.rs` to support differential updates; touches `world_update` helper; performance instrumentation in `profiling`.                                             |
| **3** | Pan & Zoom           | Introduces `camera_uniforms.rs` to compute camera position and zoom.                                                                                                        |
| **4** | Colour Modes         | Modifies the shader (WGSL) and UI (`ui/render_settings`); adds uniform `mode` and a LUT for greyscale/heat/categorical.                                                     |
| **5** | Plug‑in Co‑existence | Registers `ActiveAutomataRenderPlugin` in `io/output/plugin` and gives it its own `RenderLabel`; ensures systems run before Bevy’s render sets to keep textures up‑to‑date. |
| **6** | Performance Polish   | Optimises `upload.rs`; instrumentation under a `profiling` feature; adds texture‑atlas tiling for large boards.                                                             |
| **7** | GPU‑Compute Hook     | Adds an interface with `engine_gpu` to bind GPU‑produced textures; the renderer remains agnostic of compute location.                                                       |
| **8** | Editor Tools         | Extends `ui/panels/world/render_tools.rs` to add colour pickers, hover inspection and PNG capture; depends on `image` crate behind a `capture` feature.                     |
| **9** | Docs & Examples      | Adds mdBook chapters under `docs/` and an example in `examples/active_renderer.rs`.                                                                                         |

In your scheduling table, these systems belong to the `MainSet::Render` set and should run before Bevy’s queue stage.  The **anomaly** noted in your implementation plan—`SpriteSystem::RenderSprites` no longer exists—aligns with the updated RenderSet enumeration; use `MainSet::Render` or `RenderSet::Queue` ordering instead.

### Extensible Automata plug‑in layer (9 phases)

This road‑map refactors automaton rules into modular plug‑ins under `models::automata`, decoupling them from `engine_core`.  The phases and modules are summarised below:

| Phase | Code‑name            | Key architecture modules                                                                                                                                                         |
| ----- | -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **0** | Decouple             | Deletes `engine_core::systems::spawner`; introduces `AutomataPattern` trait and expands `models::registry` for rule metadata.                                |
| **1** | Regular V1           | Adds `RegularAutomataPlugin` and a `StepperPlugin` for Wolfram rules 30 & 110 in `automata/classical/regular/plugin.rs`; seeds are registered via `register_with_seed`.          |
| **2** | Context‑Free         | Creates `ContextFreeAutomataPlugin` in `automata/classical/contextless`; registers push‑down and L‑system rules; optional `StringTape` component for HUD display.                |
| **3** | Context‑Sensitive    | Introduces `ContextSensitiveAutomataPlugin` and seeds linear‑bounded automata in `automata/classical/contextful`.                                                                |
| **4** | Turing               | Adds `TuringAutomataPlugin` and a simple tape visualiser in `automata/classical/turing`; seeds the Turing machine tape.                                                          |
| **5** | Dynamical V2         | Splits the earlier dynamical plugin into `LifePlugin`, `LeniaPlugin` and `SwarmPlugin` under `automata/dynamical`; decides whether to use CPU or GPU steppers via rule metadata. |
| **6** | Metadata & Queries   | Extends `RuleRegistry` with `RuleMeta` (family, friendly name, GPU flag, default seed); adds `iter_metadata` helper for UI enumeration.                                          |
| **7** | Scenario Integration | Modifies UI (`io/output/ui/panels/main_menu/controller/scenario/new.rs`) to populate rule lists from `iter_metadata` instead of hard‑coded arrays.                               |
| **8** | Docs & Examples      | Adds mdBook content on writing new automata plug‑ins and examples demonstrating CPU & GPU rules.                                                                                 |

Together, these phases restructure the automata layer so that each rule family lives in its own plug‑in and registers itself declaratively; the engine core becomes agnostic of automaton details, aligning with Bevy’s recommendation to organise subsystems into separate plug‑ins.

### GPU compute shaders for automata (6 phases)

This road‑map introduces `engine_gpu` (or `engine_core::gpu`) to move cellular updates to the GPU.  Each phase maps to new or modified modules:

| Phase | Code‑name            | Key architecture modules                                                                                                                                                    |
| ----- | -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **0** | Groundwork           | Creates the `engine_gpu` crate; defines a `GpuAutomataComputePlugin`; adds compile‑time feature flag `gpu-compute` and stub WGSL file in `assets/shaders`.                  |
| **1** | Proof‑of‑Concept     | Allocates ping‑pong textures (resource `GpuGridTextures`) and implements an `AutomataUpdateNode` in the render graph; Conway Life evolves on the GPU.                       |
| **2** | Multi‑Rule Support   | Extends `RuleRegistry` to record whether a rule has a GPU kernel; adds discrete Wolfram rule kernels; uses a pipeline cache to specialise compute pipelines.                |
| **3** | Continuous Automata  | Adds support for 32‑bit float textures and Gaussian convolution for Lenia; kernels live in `models::automata::dynamical::lenia`.                        |
| **4** | Interactivity & Sync | Implements GPU seeding via `RenderQueue::write_texture`; adds a tiny reduction pass to count alive cells for HUD; supports save/load by copying textures to/from CPU grids. |
| **5** | Scaling & Polish     | Introduces tiled dispatch for huge boards, sparse fallback to CPU when density is low, profiler overlay and WASM fallback for browsers.                                     |

This plan preserves the plug‑in hierarchy: GPU features are isolated in their own crate; the renderer simply samples whatever texture is bound, and automata plug‑ins opt‑in to GPU kernels via metadata.

### Global execution order

The cross‑stream execution plan in your `implementation_plan.md` interleaves the three road‑maps to minimise rework.  It begins by decoupling the automata layer (Automata 0), then builds a basic active renderer (Render 0/1) so you can visualise dense Conway from day one.  Only after stable rendering and basic CPU rules are in place does it introduce sparse updates, pan/zoom, colour modes and plug‑in coexistence.  GPU groundwork (GPU 0) is laid once the renderer is stable, followed by a GPU proof of concept and the hook that swaps the renderer’s texture handle to the GPU output.  Multi‑rule GPU support and more exotic automata families follow, with performance polish and editor tools later.  This staged approach ensures a working build at every step and reflects Bevy’s philosophy of composing small, testable plug‑ins.

## Accentuation: emphasising the architecture

By aligning the road‑maps with the reference architecture, a few themes stand out:

* **Plug‑in modularity** – each feature (rendering, automata families, GPU compute) is encapsulated in its own plug‑in.  This means adding or removing functionality is as simple as toggling a `Cargo` feature or environment variable, and the rest of the engine remains untouched.

* **Separation of concerns** – the core simulation (`engine_core::world`, `engine_core::grid`, `engine_core::stepper`) remains pure; rendering lives in `io/output`; rule logic and metadata live in `computational_intelligence`.  The road‑maps add new crates (`engine_gpu`) rather than overloading existing ones.

* **Ordered system execution** – the render road‑map explicitly places its systems in `MainSet::Render` to run before Bevy’s `Queue` render stage.  This ensures that the texture has been updated (either by CPU upload or GPU compute) before sprites sample it.

* **Progressive enhancement** – initial phases produce a simple but functional renderer and rule set.  Later phases add pan/zoom, colour modes, editor tools and GPU acceleration, but only after earlier foundations are solid.  This mirrors the recommended practice of composing multiple plug‑ins and gradually adding more complex subsystems.
