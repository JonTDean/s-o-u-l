## Updated State-Flow Diagram

The updated application initialization and state flow reflects the new plugin architecture (automata plugins, GPU compute, and active renderer) while preserving the state management. The main app now conditionally includes the GPU compute plugin and uses the extensible automata plugin layer instead of hard-coding rules in the core. The state plugin still controls which systems run in each game state (e.g. MainMenu vs InGame):

```mermaid
flowchart LR
    subgraph MainApp["App::new()"]
    direction TB
        MainApp -->|add_plugin| DevToolsPlugin
        MainApp -->|add_plugin| EngineCorePlugin
        MainApp -->|add_plugin| AutomataPlugin
        MainApp -->|add_plugin| OutputPlugin
        MainApp -->|conditional add| GpuComputePlugin
        MainApp -->|conditional add| NetworkPlugin
        MainApp --> StatePlugin
    end

    %% Dev/Engine/Automata/Output plugin internals
    DevToolsPlugin -->|inserts| LoggingSystems & DebugTools
    EngineCorePlugin -->|inits| WorldResources
    EngineCorePlugin -->|schedules| StepperSystems
    AutomataPlugin -->|registers| RuleRegist ry & SeedHandlers
    AutomataPlugin -->|activates| FamilySubplugins
    GpuComputePlugin -->|overrides| GpuUpdatePipeline
    OutputPlugin -->|adds| UIPlugins & RenderPlugins

    %% State management and gating
    StatePlugin -->|drives| StateScheduler
    StateScheduler -->|On Enter InGame| setup_world & camera_init
    StateScheduler -->|On Exit InGame| teardown_world
    StateScheduler -->|MainMenu ↔ InGame| toggle UI & Engine systems
    NetworkPlugin -->|synchronizes| MultiplayerSystems
```

In this diagram, **AutomataPlugin** is the extensible root that adds all automaton family sub-plugins (Regular, L-system, Life, etc.), and **GpuComputePlugin** (if enabled) injects the GPU compute pipeline and disables the CPU stepper for supported rules. The **OutputPlugin** encompasses UI panels (Main Menu, Automata/Spawn panels, etc.) and the rendering plugins. The legacy `Grid2DRenderPlugin` is replaced or complemented by the new `ActiveAutomataRenderPlugin` inside OutputPlugin, which can be toggled via runtime flag (e.g. `SOUL_RENDER_MODE=active`). The **StatePlugin** manages game states and ensures that core simulation systems only run in the In-Game state, while UI menu systems run in the MainMenu state (using Bevy’s state run conditions). This decoupled, extensible flow ensures new automata or render features can be added without modifying the engine core, fulfilling the extensibility goal.

## Comprehensive Directory Architecture (Mermaid)

The project’s directory structure is updated to reflect the new GPU compute module and the automata plugin layer. Below is a mermaid diagram of the key crates and modules after integrating the **Active Automata Renderer**, **Extensible Automata Plugins**, and **GPU Compute** roadmap features:

```mermaid
flowchart LR
    %% top-level nodes for each crate
    subgraph AppCrate [app/ (Executable)]
        direction TB
        app_mod[mod.rs]
        builder_rs[builder.rs]
        plugin_reg_rs[plugin_registry.rs]
        runner_rs[runner.rs]
        state_mod_rs[state/mod.rs]:::c_app_leaf
        state_plugin_rs[state/plugin.rs]:::c_app_leaf
    end

    subgraph EngineCoreCrate [engine_core/ (Core Engine)]
        direction TB
        ec_plugin_rs[engine_core/src/engine/plugin.rs]:::c_core_leaf
        %% Core world and components
        subgraph core_module ["core/"]
            direction TB
            cell_rs[cell.rs]
            dim_rs[dim.rs]
            world_rs[world.rs]
        end
        %% Grid backends
        subgraph grid_module ["engine/grid/"]
            direction TB
            grid_mod_rs[mod.rs]:::c_core_leaf
            dense_grid_rs[dense.rs]:::c_core_leaf
            sparse_grid_rs[sparse.rs]:::c_core_leaf
        end
        %% Stepping logic
        subgraph stepper_module ["engine/stepper/"]
            direction TB
            stepper_mod_rs[mod.rs]:::c_core_leaf
            step_dense_rs[dense.rs]:::c_core_leaf
            step_sparse_rs[sparse.rs]:::c_core_leaf
            step_plugin_rs[plugin.rs]:::c_core_leaf
        end
        %% Render bridge (for legacy pipeline or generic hooks)
        subgraph render_bridge_module ["engine/render_bridge/"]
            direction TB
            rb_mod_rs[mod.rs]:::c_core_leaf
            bridge2d_rs[render2d.rs]:::c_core_leaf
            %% (bridge3d.rs removed for 2D focus if not used)
        end
        %% Engine systems (spawner removed in new architecture)
        subgraph systems_module ["systems/"]
            direction TB
            /* spawner_rs[spawner.rs] */ 
        end
        events_rs_core[events.rs]:::c_core_leaf
    end

    subgraph EngineGpuCrate [engine_gpu/ (GPU Compute)]
        direction TB
        gpu_mod_rs[src/lib.rs]:::c_core_leaf
        gpu_pipeline_rs[pipeline.rs]:::c_core_leaf
        gpu_node_rs[node.rs]:::c_core_leaf
        gpu_bind_rs[bind.rs]:::c_core_leaf
        gpu_systems_rs[systems.rs]:::c_core_leaf
        shaders_dir[[assets/shaders/*.wgsl]]:::c_core_leaf
    end

    subgraph CompIntelCrate [computational_intelligence/ (Automata & AI)]
        direction TB
        ci_plugin_rs[plugin.rs]:::c_aut_leaf
        ci_registry_rs[registry.rs]:::c_aut_leaf
        ci_prelude_rs[prelude.rs]:::c_aut_leaf
        %% Classical automata families
        subgraph classical_module ["automata/classical/"]
            direction TB
            class_plugin_rs[plugin.rs]:::c_aut_leaf
            regular_dir[[regular/…]]:::c_aut_child
            contextless_dir[[contextless/…]]:::c_aut_child
            contextful_dir[[contextful/…]]:::c_aut_child
            turing_dir[[turing/…]]:::c_aut_child
        end
        subgraph dynamical_module ["automata/dynamical/"]
            direction TB
            dynam_plugin_rs[plugin.rs]:::c_aut_leaf
            life_rs[life/mod.rs]:::c_aut_leaf
            lenia_dir[[lenia/…]]:::c_aut_child
            swarm_rs[swarm/mod.rs]:::c_aut_leaf
            reservoir_rs[reservior/*.rs]:::c_aut_leaf
        end
        %% Bridges and analytics
        bridges_mod_rs[bridges/mod.rs]:::c_aut_leaf
        world_stepper_rs[bridges/world_stepper.rs]:::c_aut_leaf
        analytics_mod[[analytics/...]]:::c_aut_child
    end

    subgraph InputCrate [io/input/ (Input Handling)]
        direction TB
        input_plugin_rs[input/plugin.rs]:::c_inp_child
        devices_mod[[devices/*]]:::c_inp_child
        network_mod[[network/*]]:::c_inp_child
        scripting_mod[[scripting/*]]:::c_inp_child
    end

    subgraph OutputCrate [io/output/ (Output & UI)]
        direction TB
        output_plugin_rs[output/plugin.rs]:::c_out_child
        %% Rendering module (2D)
        subgraph rendering_module ["output/rendering/"]
            direction TB
            rend_mod_rs[mod.rs]:::c_out_leaf
            automata_mat_rs[automata_material.rs]:::c_out_leaf
            grid2d_rs[grid2d.rs (legacy)]:::c_out_leaf
            subgraph active_renderer ["active/"]
                direction TB
                active_mod_rs[mod.rs]:::c_out_leaf
                active_plugin_rs[plugin.rs]:::c_out_leaf
                upload_rs[upload.rs]:::c_out_leaf
                camera_uniforms_rs[camera_uniforms.rs]:::c_out_leaf
                material_rs_active[material.rs]:::c_out_leaf
            end
        end
        %% UI module (egui panels and HUD)
        subgraph ui_module ["output/ui/"]
            direction TB
            ui_mod_rs[mod.rs]:::c_out_leaf
            subgraph panels_module ["panels/"]
                direction TB
                panels_mod_rs[mod.rs]:::c_out_leaf
                main_menu_dir[[main_menu/*]]:::c_out_child
                file_io_rs_panel[file_io/mod.rs]:::c_out_leaf
                subgraph world_panels ["world/"]
                    direction TB
                    world_panel_mod[mod.rs]:::c_out_leaf
                    world_panel_plugin[plugin.rs]:::c_out_leaf
                    render_tools_rs[render_tools.rs]:::c_out_leaf
                    subgraph automata_panel ["automata/"]
                        direction TB
                        automata_panel_mod[mod.rs]:::c_out_leaf
                        automata_panel_plugin[plugin.rs]:::c_out_leaf
                        spawn_panel_rs[spawn_panel.rs]:::c_out_leaf
                        show_active_rs[show_active_automata.rs]:::c_out_leaf
                    end
                end
            end
            styles_rs[styles/mod.rs]:::c_out_leaf
        end
        export_mod_rs[export/mod.rs]:::c_out_leaf
    end

    subgraph DevToolsCrate [tooling/ (Dev Tools)]
        direction TB
        dev_plugin_rs[tooling/src/lib.rs]:::c_dev_child
        logging_mod[[logging/*]]:::c_dev_child
        debug_mod[[debug/*]]:::c_dev_child
        tools_mod[[tools/*]]:::c_dev_child
    end

    %% Cross-crate relationships
    MainApp -.uses.-> EngineCoreCrate
    MainApp -.uses.-> CompIntelCrate
    MainApp -.uses.-> InputCrate
    MainApp -.uses.-> OutputCrate
    MainApp -.uses.-> DevToolsCrate
    EngineCoreCrate -.integrates.-> CompIntelCrate
    EngineCoreCrate -.integrates.-> EngineGpuCrate
    EngineCoreCrate -.integrates.-> OutputCrate
    CompIntelCrate -.provides rules to.-> EngineCoreCrate
    OutputCrate -.renders.-> EngineCoreCrate

    %% Style classes for visual grouping (optional, can be omitted in text)
    classDef c_app_root fill:#00264d,stroke:#001223,color:#ffffff;
    classDef c_app_child fill:#2672B2,stroke:#001223,color:#ffffff;
    classDef c_app_leaf fill:#6AAFE6,stroke:#2672B2,color:#000000;
    classDef c_core_root fill:#4d2600,stroke:#331900,color:#ffffff;
    classDef c_core_child fill:#996633,stroke:#331900,color:#000000;
    classDef c_core_leaf fill:#CC9966,stroke:#996633,color:#000000;
    classDef c_aut_root fill:#004d33,stroke:#00261a,color:#ffffff;
    classDef c_aut_child fill:#339980,stroke:#004d33,color:#000000;
    classDef c_aut_leaf fill:#66CCB3,stroke:#339980,color:#000000;
    classDef c_inp_root fill:#4d004d,stroke:#330033,color:#ffffff;
    classDef c_inp_child fill:#993399,stroke:#330033,color:#000000;
    classDef c_inp_leaf fill:#E580E5,stroke:#993399,color:#000000;
    classDef c_out_root fill:#660000,stroke:#330000,color:#ffffff;
    classDef c_out_child fill:#CC3333,stroke:#660000,color:#000000;
    classDef c_out_leaf fill:#F2B3B3,stroke:#CC3333,color:#000000;
    classDef c_dev_root fill:#3d3d3d,stroke:#262626,color:#ffffff;
    classDef c_dev_child fill:#999999,stroke:#3d3d3d,color:#000000;
    classDef c_dev_leaf fill:#D0D0D0,stroke:#999999,color:#000000;
```

**Notes:** This structure shows the **Engine Core** crate managing fundamental types (`Cell`, `World`, grid implementations, stepping logic) and a new **Engine GPU** module/crate that contains the compute pipeline for automata (WGSL shaders, bind group layouts, and the render graph node for GPU simulation). The **Computational Intelligence** crate now houses all automata logic, organized by automata class: *classical* (Regular, Context-Free, Context-Sensitive, Turing) and *dynamical* (Life, Lenia, Swarm, etc.), each potentially as plugins. Each automaton family plugin registers its rules and a default seeding function in the global `RuleRegistry`. The **Output** crate contains rendering and UI: the `rendering/active` module is the new GPU-friendly renderer (added in parallel to the legacy `grid2d` renderer), and the `ui/panels` now include an **Automata** panel for spawning patterns and showing active automata, as well as a **Render Tools** panel for selecting color schemes and capturing frames (added in later phases). This modular layout allows the engine to be extended with new automata or render capabilities by adding new files/plugins without altering core logic, satisfying the extensibility and modularity goals of the Q3 2025 roadmap.

## Integrated Roadmap Phases (Architecture Evolution)

The following sequence lists all phases from the three roadmaps (**Extensible Automata**, **Active Renderer**, and **GPU Compute**) interwoven in the recommended execution order. This ordering ensures that foundational changes occur first (decoupling and basic rendering), followed by incremental feature additions, ultimately transforming the architecture into the desired GPU-accelerated, plugin-extensible design:

1. **G‑0 – Automata Phase 0: Decouple.** Remove all automata-specific code from `engine_core`. Delete the old spawn systems (`spawner.rs`), and introduce an `AutomataPattern` trait and a global `RuleRegistry` to handle rule registration and seeding in a data-driven way. *(*Rationale*: This foundational refactor ensures the core engine is agnostic to specific automata, paving the way for plugin-based rules.)*

2. **G‑1 – Render Phase 0: Boot-strap.** Implement a new `ActiveAutomataRenderPlugin` with a minimal example: render a simple quad with a test texture to verify the rendering pipeline (Material2D, shader, mesh setup) on Bevy 0.16. Add a cargo feature flag `active-render` to allow toggling this plugin. *(*Rationale*: Establishes a parallel rendering pipeline without disturbing the legacy renderer.)*

3. **G‑2 – Render Phase 1: Dense Path.** Expand the active renderer to support drawing the entire grid by uploading a full texture each frame from a dense `World2D` buffer. Implement a shader that maps an R8 texture’s value to either dead or alive color. Verify with a unit test (e.g., an 8×8 checkerboard pattern). *(*Result*: We can now visualize a Conway Life board fully on the new renderer, using CPU updates.)*

4. **G‑3 – Automata Phase 1: Regular V1.** Introduce `RegularAutomataPlugin` in the automata layer. It registers classic 1D cellular automata (e.g., Wolfram’s Rule 30 and Rule 110) via `RuleRegistry::register_with_seed(...)` and adds corresponding `StepperPlugin<Rule30>` and `StepperPlugin<Rule110>` systems. Also include an event handler system to listen for `AutomataCommand::SeedPattern` and spawn patterns via the registry. *(*Result*: The engine can now support 1D elementary automata rules as plugins, and these rules can be seeded and stepped, demonstrating the new extensibility.)*

5. **G‑4 – Render Phase 2: Sparse Path.** Optimize the renderer for sparse worlds. Implement a differential update system that only updates changed cells (using a `PrevLive` HashSet to track live cells) each frame instead of uploading the full texture. Benchmark with large board sizes (e.g., 1M × 1M with 0.1% population) to ensure the sparse path meets performance targets. *(*Result*: Rendering performance is improved for sparse automata like Rule 30, and the correctness can be visually confirmed in the active renderer.)*

6. **G‑5 – Render Phase 3: Pan & Zoom.** Add camera control uniforms and a system to update them. The camera system calculates the world coordinate of the window’s bottom-left and passes `camera_pos` and `zoom` (inverse scale) to the shader so that pan and zoom transformations are handled in the GPU. Include an integration test: after panning, a given cell remains under the same cursor position (to verify coordinate math). *(*Result*: The new renderer supports smooth panning and zooming of the simulation view, critical for user experience.)*

7. **G‑6 – Render Phase 4: Colour Modes.** Extend the shader and UI for multiple color schemes. Introduce a uniform `color_mode` (0 = grayscale, 1 = “heat” viridis map, 2 = categorical palette, etc.) and implement corresponding logic in WGSL (with helper functions like `palette_heat(v)`). Add a UI toggle (e.g., dropdown or radio buttons in a Render Settings panel) to allow the user to switch color modes at runtime. *(*Result*: The renderer becomes more informative and customizable, supporting different visualizations of cell states.)*

8. **G‑7 – Render Phase 5: Plug-in Co-existence.** Register the new ActiveAutomataRenderPlugin alongside the legacy `Grid2DRenderPlugin`. Ensure they don’t conflict: schedule the active renderer’s systems in `MainSet::Render` **before** Bevy’s sprite rendering, or use a separate render graph node, so both can draw if enabled. Provide a runtime setting/env (`SOUL_RENDER_MODE=legacy|active`) to choose between the old and new renderer. *(*Result*: Both rendering paths can live side-by-side, allowing testing and fallback to the legacy renderer if needed.)*

9. **G‑8 – GPU Phase 0: Ground-work.** Set up the foundation for GPU compute. Create a new crate or module `engine_gpu` with a `GpuAutomataComputePlugin`. Add a Cargo feature `gpu-compute` (default on), and allow forcing CPU via an env var `SOUL_FORCE_CPU`. Importantly, load a placeholder compute shader (e.g., `automata_compute.wgsl`) and ensure it compiles and can be dispatched as a no-op. *(*Result*: The project is ready to include compute shaders without altering behavior yet, and we verify the rendering pipeline integration points for compute passes.)*

10. **G‑9 – GPU Phase 1: Proof-of-Concept (Conway on GPU).** Implement a working GPU update pipeline for Conway’s Game of Life. On startup, allocate two `Image` textures for the grid state (ping-pong). Establish a bind group with these as storage textures and a uniform buffer for parameters. Write a Conway compute kernel in WGSL (workgroup size e.g. 16×16) that reads from the “read” texture and writes next state into the “write” texture. Create a render-graph node (`AutomataUpdateNode`) that dispatches the compute shader before the main render pass each frame, then swaps the textures. Modify the renderer (both legacy and active) to sample from `GpuGridTextures.read` handle when the GPU plugin is active. Disable the CPU `StepperPlugin` for Life when the GPU plugin is running (using `run_if(!UseGpuCompute)` conditions). *(*Result*: Conway’s Life now steps entirely on the GPU, yielding >100 FPS on large grids, and we can toggle GPU vs CPU for comparison—both produce identical results for validation.)*

11. **G‑10 – Render Phase 7: GPU-Compute Hook.** Integrate the GPU compute with the active renderer. The Active renderer’s material was already sampling a texture; now we ensure that if the GPU pipeline is active, we bind its output texture in place of any CPU-updated texture. When the `GpuAutomataComputePlugin` creates the ping-pong textures, it should fire an event or resource update (e.g., `BindActiveTexture`) that the renderer listens for to switch its material’s texture handle to the GPU’s `read` texture. Also, skip the CPU upload systems (`upload_dense`/`upload_sparse`) when GPU compute is in use. *(*Result*: The rendering path becomes agnostic to whether the simulation is stepped on CPU or GPU—the data source is abstracted behind the texture handle binding.)*

12. **G‑11 – GPU Phase 2: Multi-Rule Support.** Extend GPU compute beyond Conway. Update the `RuleRegistry` metadata to mark which rules have a GPU implementation available (e.g., `GpuSupport::Discrete` or `GpuSupport::None`). Organize multiple compute shaders (perhaps one WGSL file per rule family) and initialize a specialized pipeline for each at runtime (using `PipelineCache`). Implement additional kernels, e.g., for 1D Wolfram rules (Rule 30, 110) that operate on textures (treating a row or small texture as 1D tape). If multiple GPU-enabled rules are active, dispatch each rule’s compute node per frame. *(*Result*: The GPU pipeline can handle different automata in the same run. For example, you could run a 2D Life and a 1D Rule30 simultaneously in different parts of the world or different worlds, with each using its GPU compute shader as appropriate.)*

13. **G‑12 – Automata Phases 2–4: Context-Free, Context-Sensitive, Turing.** Implement additional automata plugin families for more complex rule types, all on the CPU side for now (since GPU for these may not apply). This includes:

    * **Context-Free Automata Plugin (Phase 2):** add a plugin for pushdown automata and L-systems (e.g., a balanced parentheses PDA rule `pushdown:anbn` and an L-system rule like `lsys:koch`). Provide a mechanism (like a `StringTape` component or resource) to visualize their tape or stack.
    * **Context-Sensitive Automata Plugin (Phase 3):** add a plugin for linear bounded automata (e.g., a demo that checks a string with context-sensitive grammar). Seed it with a sample input (like `"aaabbbccc"`) placed into the world memory.
    * **Turing Automata Plugin (Phase 4):** add a plugin for a simple Turing machine (e.g., `tm:replace_a` which scans and replaces characters). Include a visualization of the tape head (for example, rendering the head’s position differently).

These plugins each register their rules and seed logic via the registry, similar to RegularAutomataPlugin. *(*Result*: The engine now supports a variety of automata classes via plugins, demonstrating the flexibility of the plugin system. All these remain on CPU stepping for now, which is fine given their lower performance needs or complexity.)*

14. **G‑13 – Render Phase 6: Performance Polish.** Optimize both rendering paths further. For the CPU upload (dense path), use chunked memory copies (e.g., copy 16KB at a time) instead of per-cell loops to better use CPU cache. For the sparse path, gather dirty cell indices into a contiguous buffer and use `unsafe { copy_nonoverlapping }` to batch update the texture. Also, if very large boards (>4096²) are needed, implement texture tiling (atlas) to bypass GPU texture size limits. Add profiling instrumentation (timing the upload systems) under a feature flag. *(*Result*: The active renderer meets performance targets: e.g., a 2048×2048 dense board uploads in <1ms, and 1 million live cell sparse updates in <0.6ms.*)

15. **G‑14 – GPU Phase 3: Continuous Automata (Lenia).** Enable GPU support for continuous-state cellular automata like Lenia. Introduce support for 32-bit float textures in the GPU pipeline (new bind group layout). Write a separable convolution kernel in WGSL to implement Lenia’s Gaussian blur and growth step. This likely involves precomputing a convolution kernel (on CPU) and uploading it as a small texture, then sampling in the compute shader. Include parameters like growth curve μ and σ in the uniform buffer. The fragment shader (renderer) can remain unchanged, as it will interpret the float output as intensities for coloring. *(*Result*: Lenia’s smooth lifeforms now evolve on GPU with high performance, showcasing the engine’s ability to handle continuous automata.)*

16. **G‑15 – Automata Phase 5: Dynamical V2.** Refactor the Dynamical automata into sub-plugins. Create dedicated plugins for Conway’s Life, Lenia, and Swarm/Boids. Each registers the relevant rules in the registry (with `has_gpu_impl = true` for those that have a GPU shader, such as Life and Lenia). Provide default pattern seed functions (e.g., `seed_glider` in LifePlugin, `seed_orbium_blob` in LeniaPlugin). At runtime, these plugins can decide whether to use the CPU or rely on the GPU compute (for example, LifePlugin might set a flag or simply have both implementations available). *(*Result*: The automata plugins are now neatly organized by family, and the system can automatically leverage the GPU for those rules that support it, fulfilling the extensibility and performance integration.)*

17. **G‑16 – GPU Phase 4: Interactivity & Sync.** Ensure that using GPU compute doesn’t impede interactivity. Implement features to allow spawning patterns and other interactions to affect the GPU state: for example, on a spawn event, directly write the pattern data into the GPU `read` texture via queue write or using a small compute shader, rather than going through the CPU world. Add a reduction compute shader to count live cells or gather metrics each frame (or periodically) and copy that to a CPU-accessible buffer for UI (so the HUD can display stats like population count without downloading the full texture). Also, update the save/load system: when saving, copy the GPU texture back to CPU memory once to serialize it; when loading, initialize both GPU ping-pong textures with the loaded state. *(*Result*: Users can interact (place cells, spawn patterns) in real-time even with GPU stepping, and state can be saved/loaded seamlessly, with minimal stalls.)*

18. **G‑17 – Render Phase 8: Editor Tools.** Add in-game editor UI for render and debugging tools. Create a HUD overlay (under `panels/world/render_tools.rs`) that provides controls for the active renderer: e.g., color scheme selector (bound to the uniform mode from Phase 4), sliders or color pickers for custom dead/alive colors, and a “capture frame” button that saves a PNG screenshot of the current viewport. This will use an image crate and be behind a `capture` feature flag due to file I/O. Also include a live cell value probe: on mouse hover, display the value of the cell under the cursor (which requires reading from the CPU world or a small GPU read-back in future). *(*Result*: Enhanced user tools for visualizing and capturing the simulation, making the engine more user-friendly and providing testing aids.)*

19. **G‑18 – GPU Phase 5: Scaling & Polish.** Final improvements for production readiness. Implement support for *very large* worlds by dividing the compute work into tiles (e.g., multiple 256×256 textures) and dispatching multiple compute jobs per frame to cover the whole world, or using a looping shader that processes a large texture in sections. Add a runtime check for sparsity: if a rule’s active cell count is below a threshold, automatically fall back to a CPU sparse update (to avoid GPU overhead for mostly-empty worlds). Integrate a profiling overlay (if compiled with a flag) that shows GPU timing, throughput, and FPS in the UI. And ensure graceful degradation: on WebGL/WASM or other platforms where compute shaders aren’t available, detect this and fall back to CPU stepping, perhaps with a warning message in the UI. *(*Result*: The system can handle millions of cells efficiently on capable hardware, while still supporting less capable platforms by falling back. All target goals for performance and scalability are met.)*

20. **G‑19 – Automata Phase 6–8 & Docs: Final Wrap-up.** At this stage, finalize any remaining pieces:

    * **Automata Phase 6 & 7:** Populate the `RuleMeta` in the registry with metadata (friendly names, families, GPU availability) and update the UI **New Scenario** screen to dynamically list available rules from the registry instead of hard-coding them. The Automata HUD panel can show the current rule’s friendly name and allow switching rules if applicable.
    * **Automata Phase 8:** Write documentation (mdBook chapters like *“Writing a New Automata Plugin”* and *“GPU Compute Backend”*) and provide example programs (`new_rule_cpu.rs`, `new_rule_gpu.rs`) demonstrating how to extend the system with custom rules.
    * **Render Phase 9:** Document the rendering system in a mdBook chapter “Rendering Grids” and add an example (`active_renderer` example) for how to use the active renderer in isolation.

Finally, review all naming, feature flags, and environment variables for consistency (e.g., use `.add_plugins(( ... ))` consistently for Bevy 0.11+, ensure `SOUL_RENDER_MODE` and `SOUL_FORCE_CPU` are clearly documented). Apply the small fixes noted (like correct Bevy scheduling labels and closure signatures). *(*Result*: The new architecture is fully documented and developer-friendly. The engine core no longer requires changes to support new automata or rendering modes — it’s all extensible via plugins. The simulation stepping can run on GPU for all heavy automata, achieving high performance, while the rendering is flexible and feature-rich. This completes the 2025-Q3 roadmap objectives, with a clean, future-proof architecture.)*
