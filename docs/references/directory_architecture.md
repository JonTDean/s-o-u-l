```zsh
src/
├── main.rs                 (entry point)
├── app/                   # Application orchestration (initialization, plugin setup)
│   ├── mod.rs
│   ├── startup.rs         (App initialization logic)
│   └── plugin_registry.rs (Central registration of all plugins)
├── core/                  # Core ECS state and engine domain
│   ├── state/             # Global state management
│   │   ├── mod.rs
│   │   ├── app_state.rs   (AppState enum for game states)
│   │   ├── resources.rs   (Settings, Session, other long-lived resources)
│   │   └── plugin.rs      (StatePlugin – inserts global resources, tick counter)
│   └── engine/            # Simulation engine core
│       ├── mod.rs
│       ├── grid/          # Grid storage backends
│       │   ├── mod.rs
│       │   ├── dense.rs   (Contiguous grid implementation)
│       │   └── sparse.rs  (Sparse grid/quadtree implementation)
│       ├── stepper/       # Stepping algorithms for simulation update
│       │   ├── mod.rs
│       │   ├── parallel.rs    (Parallel stepping logic)
│       │   └── sequential.rs  (Sequential stepping logic)
│       ├── render_bridge/ # Bridges from simulation to rendering
│       │   ├── mod.rs
│       │   ├── bridge2d.rs    (2D rendering bridge)
│       │   └── bridge3d.rs    (3D rendering bridge)
│       ├── components.rs  (Core ECS components, e.g. Cell, Grid metadata)
│       ├── events.rs      (Engine events, e.g. GenerationTick, CellChanged)
│       └── plugin.rs      (EnginePlugin – core systems for grid setup & stepping)
├── automata/              # Rule-set plugins for cellular automata
│   ├── mod.rs
│   ├── type1_elementary/  # 1D elementary CA (Wolfram codes)
│   │   ├── mod.rs
│   │   ├── rules.rs       (Defines elementary rule functions)
│   │   ├── components.rs  (Type1-specific ECS components, if any)
│   │   ├── systems.rs     (Systems applying 1D rules to the grid)
│   │   └── plugin.rs      (Type1Plugin – implements AutomatonRule for 1D)
│   ├── type2_surface/     # 2D Life-family CA (Conway/Dean’s Life, multi-state)
│   │   ├── mod.rs
│   │   ├── conway.rs      (Conway’s Game of Life rule implementation)
│   │   ├── dean_life.rs   (Dean’s Life rule implementation – N-state variant)
│   │   ├── components.rs  (Type2-specific components, e.g. energy levels)
│   │   ├── systems.rs     (Systems applying 2D rules each tick)
│   │   └── plugin.rs      (Type2Plugin – N-state surface automata logic)
│   └── type3_volume/      # 3D volumetric CA (Lenia and beyond)
│       ├── mod.rs
│       ├── lenia.rs       (Lenia rule implementation – continuous states)
│       ├── components.rs  (Type3-specific components)
│       ├── systems.rs     (Systems for 3D CA updates)
│       └── plugin.rs      (Type3Plugin – volumetric automata logic)
├── input/                 # Input handling domain (user, network, scripted)
│   ├── mod.rs
│   ├── devices/           # Direct hardware input (keyboard, mouse, etc.)
│   │   ├── mod.rs
│   │   ├── keyboard.rs    (Keyboard key input handling)
│   │   ├── mouse.rs       (Mouse click/position handling)
│   │   └── plugin.rs      (DeviceInputPlugin – aggregates device input systems)
│   ├── network/           # Network-based input (remote control or multiplayer)
│   │   ├── mod.rs
│   │   ├── protocol.rs    (Networking protocol definitions)
│   │   ├── client.rs      (Client-side network event handling)
│   │   └── server.rs      (Server-side sync logic)
│   │   └── plugin.rs      (NetworkInputPlugin – processes incoming network events)
│   ├── scripting/         # Scripted or AI-driven input sequences
│   │   ├── mod.rs
│   │   ├── scenario.rs    (Predefined scenario scripts for input)
│   │   └── plugin.rs      (ScriptInputPlugin – runs automated input scenarios)
│   ├── events.rs          (Input event definitions, e.g. CellToggleEvent)
│   └── plugin.rs          (InputPlugin – orchestrates all input sources/plugins)
├── output/                # Output and user interface domain
│   ├── mod.rs
│   ├── rendering/         # Visual rendering systems (graphics)
│   │   ├── mod.rs
│   │   ├── camera.rs      (Camera setup for viewing the grid)
│   │   ├── draw2d.rs      (Render cells as 2D sprites/tiles)
│   │   ├── draw3d.rs      (Render cells as 3D voxels)
│   │   └── plugin.rs      (RenderingPlugin – manages camera and draw systems)
│   ├── ui/                # GUI panels and overlays (using egui)
│   │   ├── mod.rs
│   │   ├── panels/        (UI panels for various controls)
│   │   │   ├── mod.rs
│   │   │   ├── main_menu/     (Main menu UI MVC components)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── view.rs       (UI layout for main menu)
│   │   │   │   ├── model.rs      (State data for main menu UI)
│   │   │   │   ├── controller/   (Input handlers for main menu)
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── scenario/     (New/Load scenario sub-screens)
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── new.rs        (New scenario screen logic)
│   │   │   │   │   │   └── load.rs       (Load scenario screen logic)
│   │   │   │   │   └── options.rs    (Options menu controller logic)
│   │   │   ├── sim_controls.rs (Simulation controls panel UI)
│   │   │   └── stats_panel.rs  (Stats/FPS display panel UI)
│   │   ├── file_io.rs     (UI for loading/saving patterns, e.g. .rle files)
│   │   └── plugin.rs      (UIPlugin – sets up egui context and UI panels)
│   └── export/            # Data export features (images, recordings, etc.)
│       ├── mod.rs
│       ├── image.rs       (PNG/GIF export logic for simulation frames)
│       ├── data.rs        (Export simulation data to files, e.g. .rle patterns)
│       └── plugin.rs      (ExportPlugin – systems for saving screenshots/data)
└── dev_tools/             # Developer and debugging utilities
    ├── mod.rs
    ├── logging/           # Logging and performance metrics
    │   ├── mod.rs
    │   ├── fps.rs         (FPS counter resource & system)
    │   ├── profiler.rs    (Performance profiling tools)
    │   └── plugin.rs      (LoggingPlugin – configures logging, FPS tracking)
    ├── debug/             # In-game debug tools and cheats
    │   ├── mod.rs
    │   ├── cheats.rs      (Cheat codes or debug shortcuts)
    │   ├── inspector.rs   (Integration with inspector UI for entities)
    │   └── plugin.rs      (DebugPlugin – toggles debugging features)
    └── plugin.rs          (DevToolsPlugin – master plugin enabling all dev sub-tools)
```