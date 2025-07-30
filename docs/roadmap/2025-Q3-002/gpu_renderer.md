## Road‑map: GPU Compute Shaders for Automata

**Target:** move the heavy‐weight per‑cell updates from the CPU (**`StepperPlugin`**) to the GPU, while keeping the existing zoom/pan render path and the plug‑in hierarchy intact.

---

### 📑 High‑level deliverables

| Phase | Code‑name              | Core outcome                                                                     | Primary sub‑crates touched                                                   |
| ----- | ---------------------- | -------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| 0     | *Groundwork*           | Compile‑time feature flags, decide crate boundaries, scaffolding for GPU assets. | `engine_core`, `io/output`                                                   |
| 1     | *Proof‑of‑Concept*     | Conway Life evolves **entirely on GPU** for dense grids.                         | `engine_core::gpu` (new), `io/output::rendering`                             |
| 2     | *Multi‑Rule Support*   | Rule registry knows which rules have a GPU kernel; add Wolfram Rule 30 + 110.    | `computational_intelligence`, `engine_core::gpu`                             |
| 3     | *Continuous CAs*       | Float 32 textures, Gaussian convolution → Lenia on GPU.                          | `models::automata::dynamical::lenia`, `engine_core::gpu` |
| 4     | *Interactivity & Sync* | Real‑time spawning, save/load, HUD analytics with minimal CPU↔GPU stalls.        | `engine_core::gpu`, `input`, `io/output`                                     |
| 5     | *Scaling & Polish*     | 4 K boards, sparse fallback, performance counters, WASM graceful‑degrade.        | all render/engine crates                                                     |

---

## Phase 0 – Groundwork

| Goal                     | Tasks                                                                                                                       | Notes                                                  |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| **0‑A. Crate structure** | 1. Create new crate **`engine_gpu`** (or module `engine_core::gpu`) <br>2. Re‑export a single `GpuAutomataComputePlugin`.   | Keeps compute‑only code isolated from pure CPU logic.  |
| **0‑B. Feature flags**   | • Cargo feature `gpu-compute` (default **on**). <br>• Runtime env‑var `SOUL_CPU=1` bypasses GPU plugin.               | Allows desktop ↔ web target differences.               |
| **0‑C. Asset plumbing**  | • Add shader folder `assets/shaders/` to Bevy asset server. <br>• Stub `automata_compute.wgsl` with empty `@compute` entry. | Verified by unit test that shader loads without panic. |

✅ **Exit criterion:** empty compute pipeline compiles & submits (NO‑OP) without crashing any platform we support.

---

## Phase 1 – Proof‑of‑Concept (Conway Life)

| Goal                         | Tasks                                                                                                                                                                                            | Sub‑crate touches |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------- |
| **1‑A. Ping‑pong textures**  | 1. On `Startup` (after `World2D` exists) allocate **two** `Image`s of identical size & format (RGBA8). <br>2. Store in resource `GpuGridTextures { read: Handle<Image>, write: Handle<Image> }`. | `engine_gpu`      |
| **1‑B. Bind‑group layout**   | Implement `GpuBindLayouts` resource containing: <br>• `state_in` **read‑only storage texture**. <br>• `state_out` **write‑only storage texture**. <br>• Uniform buffer `AutomataParams`.         | `engine_gpu`      |
| **1‑C. Conway kernel WGSL**  | Complete `automata_compute.wgsl` for rule ID 0 `life:conway`. 16×16 work‑group.                                                                                                                  | `assets/shaders`  |
| **1‑D. Compute pass Node**   | • Add custom render‑graph node **`AutomataUpdateNode`** scheduled *before* `MainPass`. <br>• Dispatch groups = ⌈W/16⌉×⌈H/16⌉. <br>• Swap `read`/`write` after pass.                              | `engine_gpu`      |
| **1‑E. Render integration**  | Modify `io/output::rendering::grid2d` to sample **whatever texture is in `GpuGridTextures.read`** instead of the fixed `GridTexture.handle`.                                                     | `io/output`       |
| **1‑F. Disable CPU stepper** | If `GpuAutomataComputePlugin` is active **and** rule supports GPU, disable `StepperPlugin` via `run_if(!UseGpu)`.                                                                                | `engine_core`     |

✅ **Exit criterion:** 256×256 Conway board runs > 100 fps on a mid‑range GPU; toggling `SOUL_CPU=1` falls back to original CPU code with identical visual results for 100 steps (deterministic hash).

---

## Phase 2 – Multi‑Rule Support (Discrete Integer)

| Goal                             | Tasks                                                                                                                                 |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| **2‑A. Rule metadata**           | Extend `RuleRegistry::register()` to include an enum `{ CpuOnly, GpuDiscrete, GpuFloat }`.                                            |
| **2‑B. Shader specialisation**   | Build one WGSL per rule family (`life_discrete.wgsl`, `wolfram1d.wgsl`). Use `PipelineCache::queue_compute_pipeline()` once per rule. |
| **2‑C. Wolfram rules**           | Implement Rule 30 & 110 kernels (1D wrapped across X dimension). Provide compile‑time const LUT (8‑bit).                              |
| **2‑D. Dynamic pipeline switch** | When scenario contains multiple GPU‑capable rules, run **one dispatch per rule** with its own bind group (cheap).                     |

✅ Conway + Rule30 + Rule110 can coexist in one scenario. Visual match against CPU reference for 512×512 board across 1 000 steps.

---

## Phase 3 – Continuous Automata (Lenia)

| Goal                          | Tasks                                                                                                     |
| ----------------------------- | --------------------------------------------------------------------------------------------------------- |
| **3‑A. Float textures**       | Add RGBA32‐float format support, new layout entry. Introduce `GpuGridFormat` resource chosen per rule.    |
| **3‑B. Gaussian convolution** | In WGSL, perform separable (or full) convolution using radius ≤ 20. Use group‑shared memory optimisation. |
| **3‑C. Growth curve**         | Encode `mu`, `sigma`, `dt` into uniform block.                                                            |
| **3‑D. Kernel pre‑bake**      | Pre‑compute kernel weights on CPU → upload to 2D texture; sample in compute shader.                       |
| **3‑E. Visualisation**        | Keep fragment shader unchanged – it interprets float (0‒1) → greyscale or two‑colour ramp.                |

✅ Lenia Orbium self‑replicates with smooth motion, > 60 fps at 512×512.

---

## Phase 4 – Interactivity & Sync

| Goal                    | Tasks                                                                                                                                                                                                         |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **4‑A. Seeding events** | On `AutomataCommand::SeedPattern`, write directly into `GpuGridTextures.read` via `RenderQueue::write_texture()`; avoid CPU world mutation.                                                                   |
| **4‑B. HUD analytics**  | Implement optional **small** compute pass that reduces alive‑cell count into a 1×1 storage buffer each second; map buffer to CPU for `swarm_summary`.                                                         |
| **4‑C. Save / load**    | • On save, schedule read‑back of full texture into staging buffer once, translate to `GridBackend::Dense`. <br>• On load, populate both ping‑pong textures via `write_texture` before first compute dispatch. |

✅ User can spawn Glider gun (Life) at run‑time and instantly observe GPU evolution; saving & loading round‑trips without data loss.

---

## Phase 5 – Scaling & Polish

| Goal                        | Tasks                                                                                                                           |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| **5‑A. Very large boards**  | Implement **tiled** compute: split world into N×M 256² tiles, each its own image; loop over tiles each frame or multi‑dispatch. |
| **5‑B. Sparse fallback**    | If density < 4 %, switch to CPU sparse grid for that rule.                                                                      |
| **5‑C. Profiling overlays** | Add `--gpu-profiler` flag: shows dispatch time, memory throughput, FPS in‑game overlay.                                         |
| **5‑D. WASM degrade**       | Detect `webgpu` feature; if compute unavailable, automatically switch to CPU stepper; warn user.                                |
| **5‑E. Docs & examples**    | New mdbook chapter *“GPU Compute Back‑end”*; cargo example `cargo run --example gpu_life`.                                      |

✅ 4 096×4 096 Lenia world updates at ≥ 30 fps on RTX 3070; profiler overlay confirms < 12 ms/frame compute time.

---

## Sub‑crate / Module Breakdown

| Sub‑crate                              | New / Modified modules                                                                      | Responsibility                                                               |
| -------------------------------------- | ------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| **`engine_core`**                      | `gpu` (new), `world`, `stepper` (toggle)                                                    | GPU resources, render‑graph node, runtime flags.                             |
| **`engine_gpu`** *(optional separate)* | `pipeline.rs`, `bind.rs`, `node.rs`, `systems.rs`                                           | Encapsulate all GPU‑specific Bevy code.                                      |
| **`computational_intelligence`**       | `automata::<rule>::gpu_kernel` (WGSL snippets embedded as `include_str!`)                   | Keep rule logic and WGSL in same sub‑module; unit tests for LUT correctness. |
| **`io/output`**                        | `rendering::grid2d` (texture handle swap), maybe `automata_material` (float format support) | Visualisation unchanged but now samples GPU‑updated texture.                 |
| **`input`**                            | `plugin::events` (SpawnPattern direct GPU write)                                            | User interactions that mutate grid.                                          |
| **`assets/shaders`**                   | `life_discrete.wgsl`, `wolfram1d.wgsl`, `lenia.wgsl`                                        | Each compute kernel plus utility functions.                                  |

---

## Milestone Checklist

| Phase | Unit tests                                        | Benchmarks                 | Docs               |
| ----- | ------------------------------------------------- | -------------------------- | ------------------ |
| 0     | Shader loads test; feature‑flag compile           | n/a                        | Cargo docs updated |
| 1     | CPU vs GPU equality hash for 100 steps            | `cargo bench life_gpu`     | README POC section |
| 2     | Registry correctly routes Rule30 GPU path         | Rule30 1 K cells < 1 ms    | Changelog          |
| 3     | Lenia float precision within ±1 e‑5 vs CPU 32‑bit | 512×512 Lenia < 2 ms       | Lenia guide        |
| 4     | Seed pattern arrives in GPU tex                   | Alive count update ≤ 16 ms | HUD doc            |
| 5     | 4 K×4 K stability 10 K steps                      | 30 fps sustained           | mdBook chapter     |

---

### 🌟 Final state

*Simulation stepping is fully GPU‑accelerated for all grid‑based automata; CPU steppers remain as a fallback. The plugin‑based architecture is preserved: each automaton family simply **registers an additional WGSL kernel** when available. Users enjoy smooth zoom/pan interaction, rapid spawning, and the ability to simulate multi‑million‑cell worlds in real time.*
