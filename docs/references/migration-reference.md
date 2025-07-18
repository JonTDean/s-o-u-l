# Deeply Nested ECS Architecture for Swarm Orchestrator for aUtonomous Learners

## Proposed Directory Hierarchy

&#x20;**Figure 1:** **5-level nested file hierarchy** for the Swarm Orchestrator for aUtonomous Learners project, reorganized into semantic domains. Each top-level folder (e.g. `core`, `automata`, `input`, `output`, etc.) contains multiple layers of submodules, ensuring at least 5 levels of depth with meaningful names at every level. This structure reflects distinct runtime roles (state management, simulation logic, input handling, output presentation, developer tools) and fosters self-documentation by letting the module path describe its purpose (for example, `input::devices::keyboard` clearly denotes keyboard input handling). The deep nesting (with sibling modules at each layer) makes the architecture highly modular and extensible, as new features can be added by introducing new submodules in the appropriate domain without disrupting existing code.

Below is the **renamed and restructured** directory tree, using snake\_case naming and grouping all existing crates/plugins (`ca_engine`, `automata`, `ui`, `network`, `dev_tools`, etc.) into hierarchical domains that mirror their ECS roles:

```text
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

This hierarchy groups code by **runtime concerns**. For example, **`core/engine`** encapsulates dimension-agnostic simulation logic (grid management, stepping, rendering bridges) similar to the original `ca_engine` crate, while **`automata/`** contains pluggable rule sets (Conway’s Life, Dean’s Life, Lenia, etc.) analogous to the original rule-set plugins. The **`input/`** domain centralizes all input sources (hardware, network, scripted), **`output/`** covers rendering and UI, and **`dev_tools/`** wraps utility plugins (like quitting, logging) that were previously under `dev_utils`. Each nested folder name is **semantic** (e.g. `rendering/draw2d.rs` vs a generic name) to make the code self-explanatory.

## Plugin Registration and Execution Order

&#x20;**Figure 2:** **Plugin registration graph** – The main `App` registers each feature as a Bevy plugin in a layered order. Arrows indicate how plugins are added and how some plugins in turn register sub-plugins. This ordering ensures that global state and utilities initialize first, the simulation engine next (which then loads specific automata plugins), and output modules last. Solid arrows show the main registration sequence, and side branches show nested plugin insertion. This sequence aligns with Bevy’s plugin-per-feature model: each major subsystem is a plugin, promoting modular startup and clear execution grouping.

The recommended **initialization sequence** is:

1. **StatePlugin** – first, to initialize global resources like configuration settings and a `Session` (frame counter) before other systems run.
2. **DevToolsPlugin** – sets up developer utilities early (e.g. quit-on-ESC, logging) so they apply globally from the start.
3. **InputPlugin** – to collect user input events (and possibly network input) each frame, ready for when the simulation begins.
4. **EnginePlugin** – the core simulation engine; on build, it inserts the grid and stepping systems and then **registers the rule-set plugins** for each automata type needed. This nested plugin graph means the engine can dynamically include one or multiple automata concurrently (e.g. to compare two rules side-by-side).
5. *Automata Type Plugins:* **Type1Plugin**, **Type2Plugin**, **Type3Plugin** – added *by* the EnginePlugin to provide concrete rule logic and parameters. Each of these plugins injects a specific `AutomatonRule` implementation along with its default parameters, palette, and any UI components for tuning that rule (e.g. Type2Plugin registers the rules for Conway’s and Dean’s Life and their UI controls).
6. **NetworkPlugin** – (if networking is enabled) connects after the engine so it can broadcast or receive world state. The `NetworkInputPlugin` (here shown as part of input) could also be initialized within InputPlugin; either way, it hooks into the simulation loop to apply remote events and sync state.
7. **UIPlugin** – sets up the UI infrastructure (egui context, panel systems) after the engine so that UI can display or modify simulation state. It may depend on resources from earlier plugins (e.g. `Session` or automata parameters) to populate the UI.
8. **RenderingPlugin** – adds rendering systems (cameras, drawing) that run after the simulation logic, ensuring that what’s drawn reflects the latest state. In a 2D run, only the 2D draw system would be active; a 3D view could be another plugin or a variant of the RenderingPlugin.
9. **ExportPlugin** – last, so that it can capture outputs (images or data dumps) after all other systems have updated. It listens for final frame states or user commands (via UI) to save snapshots, ensuring nothing is saved before state is ready.

This ordered registration guarantees that **global state** and dev tools are available to all, **inputs** are collected before simulation updates, the **engine** runs with all rule plugins in place, and **outputs** (UI, rendering, export) occur after the simulation logic. Each plugin cleanly encapsulates a feature, and their initialization order can be adjusted easily if dependencies change (thanks to the centralized `plugin_registry.rs`).

## Domain-Specific Plugin Breakdown

Each domain (core, automata, input, output, dev) is implemented as a collection of **Bevy plugins**. Below we describe each plugin’s responsibilities in terms of its ECS **systems**, **components/resources**, and **events**:

* **StatePlugin** – Handles application-wide state. *Systems:* Inserts global resources on startup (e.g. loads `Settings` from config, initializes `Session`) and runs a tiny system to tick a frame counter every update. *Components/Resources:* Defines config resources like `Settings` (simulation parameters loaded from file or defaults) and a `Session` resource to track running status (e.g. current frame number). *Events:* Could emit state-change events (e.g. `AppStateChange`) when transitioning between states (MainMenu → InGame, etc.), enabling other plugins to respond to mode changes.

* **DevToolsPlugin** – Aggregates quality-of-life utilities for developers. *Systems:* Registers sub-plugins like **LoggingPlugin** (which spawns a system to log FPS or other metrics each second) and **DebugPlugin** (which may add systems for cheat codes or entity inspection). Also includes simple global systems like `quit_on_esc` to handle exiting on keypress. *Components/Resources:* Provides debug-only resources (e.g. an `FpsCounter` resource storing recent frame times) and config flags (like enabling/disabling certain dev features). *Events:* Might listen for debug toggle events (e.g. a key event to enable a debug view) or emit profiling data events to be consumed by a debug UI.

* **InputPlugin** – Master plugin for all input sources. *Systems:* Registers and coordinates **DeviceInputPlugin**, **NetworkInputPlugin**, and **ScriptInputPlugin** (each can also be individual `Plugin` structs). It ensures that on each frame (in the `Update` stage) input systems run to collect inputs. For example, it might call keyboard/mouse handling systems to produce game events or mark components (like a “wants\_to\_toggle\_cell” component on an entity) based on user actions. *Components/Resources:* Defines input state resources (e.g. a `InputState` resource aggregating current input context) and processes raw hardware input into high-level game commands. *Events:* Defines events such as `CellToggleEvent` or `SpawnPatternEvent` that are fired when the user triggers an action (via keyboard, mouse, or UI) – these events are then consumed by the simulation logic (or by UI for feedback).

* **NetworkInputPlugin** – (within input domain) Handles multiplayer or remote control input. *Systems:* Runs alongside device input systems; listens for network packets and translates them into the same events/resources used by local input (e.g. treating a message from a remote client as a `CellToggleEvent` on a specific cell). Also could send local player events over the network to a server/peers. *Components/Resources:* Maintains network connection resources (sockets or client state) and a `NetworkBuffer` component or resource to accumulate incoming messages. *Events:* Emits events like `NetworkEvent` or uses the same input events, enabling the rest of the game to remain agnostic to input origin. This plugin ensures that whether input comes from a local user or a network message, the subsequent systems handle them uniformly.

* **EnginePlugin** – The core simulation engine plugin (formerly `ca_engine`). *Systems:* Initializes the simulation world structure (e.g. creating the grid entity and attaching the chosen grid backend component), and sets up the **FixedUpdate** stepping system that advances the cellular automata each tick. It adds the primary game loop system `step_world` which runs on a fixed timestep to apply the automata rules to the grid, updating cell states each tick. It also may manage systems for neighbor indexing or other engine-level tasks. *Components:* Defines core ECS **components** such as `Cell` (with current state, e.g. alive/dead or energy level), `Grid` (the data structure for cells, possibly with variants for dense vs sparse storage), and maybe `Neighborhood` or `CellHistory` components if needed for rules. *Resources:* Holds global simulation parameters (e.g. rule parameters as a resource blob, grid dimensions, tick rate) – for instance, a `RuleParams` resource holds the active rule’s parameters. *Events:* Could emit a `GenerationTickEvent` after each tick (so UI or other systems know a generation advanced), or events when major changes occur (e.g. grid reset or simulation paused). Notably, EnginePlugin **embeds the rule-set plugins** on build, passing in the specific rule implementations (it might call `app.add_plugin(StepperPlugin{ rule: MyRule, params: … })` for each selected automaton).

* **Automata Type Plugins** – Domain-specific logic for each family of rules. Each provides an implementation of the `AutomatonRule` trait for a certain dimensionality and rule-set, plus related assets (default params, palette, UI hooks).

  * *Type1Plugin* – 1D elementary cellular automata (e.g. Wolfram’s rules). It registers a concrete rule struct implementing `AutomatonRule` for 1-dimensional tape evolution (2-state). *Systems:* It may add a system to load a default 1D pattern or handle wrapping of the 1D world. Typically, however, most of its logic is in the rule trait impl (the `next_state` function for the 1D neighborhood). *Components/Resources:* Could provide a resource with the current rule number (e.g. Wolfram code) and any precomputed lookup tables. *Events:* Possibly none unique, though it could emit an event if the rule supports something like randomize or specific pattern placement triggers.
  * *Type2Plugin* – 2D “Life” family automata (Conway’s Life and Dean’s Life). This plugin covers the classic 2-state Conway rule and the extended **Swarm Orchestrator for aUtonomous Learners** which allows multiple states (e.g. fading cells). It provides an `AutomatonRule` impl for 2D grids (implements `next_state` for Moore-neighborhood rules). *Systems:* Adds the rule-specific system that might initialize a color palette for states, and potentially an overlay system for highlighting special cells (if needed). It might also register an egui section for rule parameters (e.g. sliders for birth/survival thresholds or state count) using a trait like Inspectable. *Components/Resources:* Introduces rule configuration resources (for example, a `LifeRules` resource encoding the B/S rulestring for Conway or an extended set of parameters for Dean’s variant). It also might use a component for cell “age” or “energy” if Dean’s Life uses graduated states, so the rendering can depict cells in intermediate states. *Events:* Could emit an `ExplosionEvent` or other pattern-specific events if certain configurations arise (for extensibility), but generally the rule operates via pure state transitions rather than discrete events.
  * *Type3Plugin* – 3D Lenia or other volumetric automata. Implements `AutomatonRule` for 3D grids (or continuous-state automata). *Systems:* May set up a 3D-specific world initialization (e.g. spawning a 3D grid of cells or configuring a voxel rendering plugin) and any continuous update resources (Lenia might use floating-point field updates on FixedUpdate as well). *Components/Resources:* Provides a resource struct for Lenia parameters (like growth rate, convolution kernel, etc.) and might attach a component to each cell for storing continuous state (e.g. an energy level float). *Events:* Possibly emits an event for certain thresholds (like “organism spawned” if patterns form), but primarily relies on state fields updated each tick. All type plugins also register a default parameter set and a color palette for their cells, making it easy to add new rule families by following the same pattern.

* **UIPlugin** – Manages all user interface elements (built on Bevy’s egui integration). *Systems:* Initializes the UI camera and egui context (often by adding `EguiPlugin` from bevy\_egui) and then spawns UI panels. For example, it sets up the persistent side panel with simulation controls and binds it to simulation state/resources. It also coordinates with automata plugins to include their parameter controls – e.g. if a Type2Plugin implements an interface for UI, the UIPlugin will include that in a panel section. In the main menu state, UIPlugin (or a sub-plugin like MainMenuPlugin) spawns menu screens and runs their update systems (using an ECS approach to UI screens). *Components/Resources:* Uses resources to store UI state (for instance, the data models for each UI panel, like `MainMenu` struct as a resource while in the menu, or a `SimControls` resource for in-game UI state). It defines components for UI interactive elements if needed (though with egui typically one uses immediate mode patterns rather than permanent UI entities). *Events:* Processes UI events (like button clicks or menu selections) – often these come through as egui events which the UIPlugin translates into game events or state changes (e.g. user clicks “Start” -> triggers state change to AppState::InGame, or clicks “Load Pattern” -> triggers a FileLoadEvent). It may emit events like `FileLoaded` or `ExportRequested` that the ExportPlugin or other systems listen for. Internally, the UI code is organized in an **MVC pattern** per panel (as shown by submodules `view`, `model`, `controller` in the main\_menu panel) for clarity, which makes the UI code self-documenting and easy to extend with new panels.

* **RenderingPlugin** – Handles all rendering of the simulation world (apart from egui UI which is handled in UIPlugin). *Systems:* Creates and configures the main camera on startup (depending on 2D or 3D view) and runs rendering systems after the simulation update. For example, it adds a system in the `PostUpdate` stage to iterate over all cell entities and update their `Sprite` or `Mesh` based on the current cell state. It might have separate sub-systems or toggles for 2D vs 3D: e.g. a `draw2d` system active when in 2D mode, and a `draw3d` system when in 3D mode. *Components/Resources:* Defines components used purely for rendering, such as a `CellSprite` component (holding a handle to a sprite or color) that mirrors a cell’s state, or a `CellVisibility` component for culling in 3D. It may also use a resource for rendering settings (like zoom level, color palette mapping – the palette likely comes from the automata plugin’s resources). *Events:* Could react to events like window resize or a toggling of view mode (e.g. an event to switch from 2D to 3D view), adjusting cameras or re-spawning render components accordingly. Essentially, this plugin bridges the gap between the **simulation state** and the graphics: whenever the engine produces new cell states, the RenderingPlugin’s systems ensure the visuals update to match (cells appearing/disappearing or changing color).

* **ExportPlugin** – Provides the ability to export simulation data or imagery (useful for saving states or making presentation visuals). *Systems:* Listens for user commands or periodic triggers to capture output. For instance, if the user clicks “Save GIF”, this plugin’s system will start capturing frames each tick and then encode them to a GIF file once enough frames are collected. Another system might handle saving the current grid to a file (e.g. outputting the pattern in *.rle* or a custom format). These systems likely run in the **PostUpdate stage** or on demand, so they capture the state *after* the latest simulation step and rendering. *Components/Resources:* Could include a `Recorder` resource that holds frame data for a GIF being recorded, or an `ExportSettings` resource describing what to export (image sequence, data dump, etc.). If exporting data, it might temporarily tag entities or use a resource to gather all cell states. *Events:* Reacts to events like `ExportScreenshotEvent` or `SavePatternEvent` (fired by UI buttons) to actually perform the export. It may also emit an event on completion, such as `FileSavedEvent`, which the UIPlugin can use to show a confirmation to the user. By isolating these tasks in an ExportPlugin, we ensure the heavy file I/O or image encoding work is kept separate from core simulation loops, maintaining a clean separation of concerns.

Each plugin above is **highly modular** – it defines clear boundaries (via the ECS data it owns and the systems it runs). This makes the system extensible: for example, introducing a new automata type (say Type4) would involve adding a new `automata/type4` module with its `Type4Plugin` implementing the `AutomatonRule` trait and any UI hooks, then registering it in the Engine or via user selection. Similarly, one could add a new input source (e.g. a MIDI device controlling parameters) by creating a new plugin under `input/` without touching the engine or output code. The deep nesting of modules helps keep each feature’s code localized and named according to its function, aiding maintainability.

## Trait Abstractions for Extensibility

A cornerstone of this design is the use of **trait-based abstractions** to allow plugging in new behaviors for rules, stepping algorithms, and rendering strategies:

* **AutomatonRule Trait:** At the heart of the simulation is the `AutomatonRule` trait, which each rule plugin implements. This trait defines the function to compute a cell’s next state (`next_state`) given the current context (neighborhood, memory, etc.). By having a trait, the engine can remain generic over the rule logic – the EnginePlugin simply stores a trait object or generic type for “the current rule” and calls `next_state` polymorphically. This means new rule implementations (Conway, Dean’s Life, Lenia, or any custom rule) can be “dropped in” without altering engine code, as long as they adhere to the trait. The trait uses an associated `D: Dim` (dimension) to generalize over 1D, 2D, 3D rules, ensuring that the same interface can support different grid dimensionalities. Each automata plugin provides a concrete type that implements `AutomatonRule` (for example, `ConwayRule` for Conway’s Life), along with any rule-specific data. The engine’s stepping system is written against this trait, calling it for each cell to evolve the grid. This **backend rule trait** approach cleanly separates rule logic from the engine loop, and makes the rule code testable in isolation. It also enables features like runtime rule swapping – e.g., the user could select a different rule from a menu and the game could switch out the resource implementing `AutomatonRule` to a different plugin’s rule struct.

* **Stepping Strategy Abstraction:** The architecture separates *what* the rule is from *how* the grid update is executed. The **Stepper** functionality is abstracted such that the engine supports different update strategies (dense vs sparse grid, sequential vs parallel iteration) without changing the rule logic. In practice, this isn’t a single trait in the current code, but rather an abstraction achieved via module structure and generics. For example, the `step_world` system in EnginePlugin checks the selected `GridBackend` and then dispatches to either `step_dense` or `step_sparse` implementation. We could formalize this with a trait like `GridStepper` or by implementing the stepping as methods on the grid itself; however, the current design uses an enum and simple branch for clarity. The **FixedUpdate scheduling** is also part of this abstraction – by running the step system in a fixed timestep schedule, the engine can maintain a stable simulation tick independent of rendering frame rate. The `StepperPlugin` shown in the core inserts the rule and its parameters as resources and adds the `step_world` system on a fixed timestep. This pattern allows easy changes to how stepping is done: for example, one could introduce a multi-threaded stepper that divides the grid into chunks (by implementing a different `step_world` or using Bevy’s task pool) without modifying the rule logic or other systems. In summary, the stepping mechanism is designed to be **swappable** and scalable – if a new stepping algorithm or a continuous-time update was needed, it could be integrated by extending the engine’s stepping module (or adding a new plugin) that respects the same trait contracts or resource interfaces for rules and grids.

* **Rendering/Visualization Traits:** Rendering is decoupled from game logic via a **bridge interface**. In our structure, `core/engine/render_bridge` contains code that translates game state into renderable data. One could define a trait (e.g. `RenderBridge`) that has methods like `spawn_visual(cell_state)` or `update_visual(cell_entity, cell_state)`. In practice, the code might simply use systems operating on query filters (for example, a system that queries all entities with a `Cell` component and a missing `Sprite` component to spawn a sprite for new cells). Whether using an explicit trait or not, the concept is that the engine doesn’t know about Bevy’s rendering details – instead, the RenderingPlugin takes the authoritative state (the cell components) and produces the on-screen representation. If we wanted to support multiple visualization modes, we could leverage Rust traits or Rust’s type system: for example, a generic Renderer plugin `R: CellRenderer` could be added, where `CellRenderer` is a trait with methods to initialize and draw cells. A `SpriteRenderer` struct would implement it for 2D and a `VoxelRenderer` for 3D. Then the app could choose which to add. Alternatively, using compile-time features or run-time flags, the `RenderingPlugin` could instantiate different systems. The key abstraction is that **renderers subscribe to simulation state** (often via querying components or events) and can be extended independently. In our architecture, adding a new rendering approach (say, an ASCII text renderer for the grid) would mean writing a new plugin that also observes the same cell state and creates a different output, without touching the core engine. This follows the open/closed principle – engine code outputs states (components/events) in a consistent way, and any number of renderer plugins can consume that to visualize the simulation in different forms.

* **UI and Parameter Traits:** To keep the UI extensible and decoupled, we use traits or interfaces for exposing data to UI panels. For instance, each automata plugin might implement an **Inspectable** or **ParameterProvider** trait that the UIPlugin knows about. In the context of Bevy and egui, one approach is using the `bevy_inspector_egui` crate which allows deriving an Inspectable for resource structs. The architecture document mentions an “optional trait InspectableParameters” – implying that rule plugins can optionally provide an inspectable parameter struct to auto-generate UI controls. By defining a common interface for UI parameter tweaking, we ensure that when a new rule plugin is added, if it provides an implementation of (or data for) this trait, the UIPlugin can automatically include controls for it without hardcoding each rule’s fields. This trait-abstraction approach is also seen in how the main menu is structured: the UI runner function is generic over a screen type that implements a certain interface (like a trait or simply a common pattern of having a `model` and `view`), allowing the same `ui_runner` system to drive different screens by trait bounds. In summary, the UI architecture uses Rust’s trait and generics to treat different screens or different rule parameter panels uniformly. This makes the UI highly extensible – new panels or new rule configs can slot in as long as they implement the expected trait/struct interface (for example, a new automaton plugin could provide a struct of settings that implements `Inspectable`, and the UI panel code can automatically render sliders for it).

* **Decoupling via the Plugin Trait:** Finally, it's worth noting that each module leverages Bevy’s own `Plugin` trait to encapsulate its setup. By using the `Plugin` trait pattern per feature (engine, each automaton, UI, etc.), the codebase leans on Bevy’s scheduling and dependency management. The `build(&self, App)` method of each plugin is effectively where we enforce boundaries – e.g., the EnginePlugin’s `build` will only deal with setting up engine systems and adding automata plugins, and not call any UI code directly. This use of plugin traits ensures each part of the code only exposes a controlled API (the plugin itself), keeping internal details hidden. The result is a **plug-and-play architecture** – for example, one can disable a plugin at compile time (by toggling a feature flag or not adding it in `plugin_registry.rs`) and the system should still compile and run the remaining parts, thanks to minimal coupling.

Together, these trait abstractions make the design **flexible**. New implementations of rules, stepping strategies, or renderers can be added by implementing the respective trait or following the established pattern, with minimal changes to existing code. This fulfills the goal of a “fully semantic, n::n breadth” architecture: at each level of abstraction, there are multiple interchangeable implementations (n of each type) ready to be composed as needed.

## ECS System Flow: Input → Logic → Output

&#x20;**Figure 3:** **ECS data flow across system stages** – illustrating how input events propagate to game state updates, which then propagate to outputs. The simulation uses distinct **system sets/stages** to separate concerns: first, input systems run (during the `Update` stage) to collect user or network input; next, the core logic systems run (during a fixed-timestep stage, e.g. `FixedUpdate`) to update the simulation state; finally, output systems run (during `PostUpdate`) to render and present the new state. This pipelined stage separation ensures a clear flow: no output system reads stale data, and no logic runs on partial input.

In more detail, the **input stage** (Update) reads external events each frame. For example, if the player clicks a cell, the `handle_mouse_clicks` system in the Renderer2DPlugin will capture that and, if in-game, translate it into a `CellToggleEvent` or directly mark a cell's component to flip state. Similarly, keyboard input (via DeviceInputPlugin) could set a resource like `PendingPattern = "GLIDER"` when the user requests spawning a glider pattern. These inputs are either stored in resources or dispatched as events.

Next, on the fixed simulation tick (e.g. every 0.1 seconds), the **logic stage** runs the stepping system. The `step_world` system (from EnginePlugin) reads the current world state (e.g. the `World2D` grid resource and the active rule resource) and applies the rule to compute the next generation. It will also consume any input events/resources from the prior stage – for instance, if a `CellToggleEvent` was fired, a system in the logic stage might intercept it and directly change that cell’s state or schedule a change before the stepping occurs. The stepping system then updates all cells’ states, possibly incrementing an internal generation counter (Session.frame) as well. By isolating this in a FixedUpdate stage, the simulation can run at a stable rate; multiple input frames might accumulate, but they will be processed on the next tick in batch, and rendering can interpolate or just reflect the last tick.

After the simulation state is updated, the **output stage** (PostUpdate) systems run. Here, the RenderingPlugin’s systems will read the updated components (e.g. which cells are now alive/dead) and draw the appropriate sprites or meshes. The UIPlugin might also have systems in this stage or in a separate UI-specific stage (after PostUpdate or in parallel) that read the latest state to update GUI elements (for example, updating an on-screen text showing the current generation count, which comes from the `Session` resource updated by the logic). Because these output systems run after the logic, they see a consistent, fully updated world state for that tick. They then produce side effects only related to presentation: drawing to screen, playing sounds (if any), recording frame data, etc. Notably, **events** can also flow from logic to output – e.g., the EnginePlugin could emit a `GenerationTickEvent` each tick, and the UIPlugin might have a system listening to that to flash a UI indicator or log the tick in a graph. The PostUpdate stage is ideal for such event-driven UI updates because it ensures the logic has finished computing the tick.

Finally, the cycle repeats: input systems for the next frame will pick up new inputs (or persistent ones like holding a key), logic will run on the fixed schedule to incorporate them, and outputs will render the result. If the simulation is paused (say via an AppState or a pause resource), the EnginePlugin’s systems might be gated (not running while paused), but the input and UI could continue (so the user can, for instance, adjust settings while paused, which update resources, then resume the tick). This ECS flow is highly **extensible** – new system sets can be slotted in as needed (for example, an AI decision-making stage could be inserted between input and logic if needed, or a cleanup stage after output). By following the input→logic→output separation, we uphold a clear order of operations, avoiding race conditions and making the frame processing easier to reason about.

## Modularity and Extensibility by Design

This deeply nested ECS architecture emphasizes **modularity** at every level. Each plugin and module has a single well-defined purpose (adhering to separation of concerns), and communicates with others only through standard ECS mechanisms (resources, components, events). The semantic naming of directories and modules means that a developer can infer the role of each piece of code from its path, achieving a **self-documenting** structure. Adding new features or swapping out implementations is straightforward: for example, to support a new cellular automaton, one creates a new plugin under `automata/` implementing the required traits, and the EnginePlugin can load it without engine changes; to change the rendering style, one can introduce a new system or plugin under `output/rendering` and disable the old one. The `n::n` breadth at each depth implies multiple choices (n implementations) for each abstraction layer – and indeed our design provides for multiple input methods, multiple rule plugins, multiple output forms operating concurrently. This not only future-proofs the **Swarm Orchestrator for aUtonomous Learners** project for extension (new rules, dimensions, UIs, etc.) but also makes it easier to test and maintain (each module can be understood and developed in isolation). In sum, the proposed architecture creates a robust foundation that is **highly modular, extensible, and clear by construction**, leveraging Rust’s and Bevy’s strengths to implement a scalable ECS plugin system.

**Sources:**

1. Swarm Orchestrator for aUtonomous Learners repository README – Project overview and original module structure
2. Architecture sketch – Nested plugin graph and plugin responsibilities
3. Core engine code – AutomatonRule trait definition (generic rule interface) and fixed-step update system (StepperPlugin)
4. State plugin code – Global resource initialization and frame tick example
5. UI module code – MVC pattern in UI (view, model, controller separation for MainMenu) and state-driven UI system scheduling
