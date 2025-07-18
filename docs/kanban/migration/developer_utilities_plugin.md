```
kanban
    uiTodo[To Do]
        ui1[Design UserInterfacePlugin crate to encapsulate all UI and rendering systems]
        ui2[Implement egui side-panel with controls (sliders for speed, dropdown for rule selection, play/pause buttons)]
        ui3[Define `RenderBridge` trait to abstract drawing of simulation state (decouple simulation_core from rendering layer)]
        ui4[Implement user_interface::render::2d::SpriteRenderer system (sprite-based rendering of the cell grid using `RenderBridge`)]
        ui5[Plan user_interface::render::3d::VoxelRenderer module (future 3D visualization of the grid)]
        ui6[Implement file I/O module for patterns (load .rle, .life files and save custom format)]
        ui7[Add screenshot/export functionality (e.g. PNG/GIF output of the simulation state)]
        ui8[Restructure UI code into the new crate (migrate any existing UI code and ensure semantic naming clarity)]

    uiProg[In Progress]
        ui9[Prototyping RenderBridge implementation for 2D sprites – **in progress**]
        ui10[Developing SimParams control panel (UI elements for rule parameters) – **in progress**]

    uiRev[Review]
        ui11[UX review of UI layout and controls (ensure intuitive design for users)]
        ui12[Testing rendering performance and frame rate with current UI (reviewing for optimizations)]
        
    uiDone[Done]
        ui13[Basic windowing and egui integration up and running (initial UI plugin scaffolding)]
        ui14[Placeholder UI panel displaying simulation status (proof of concept implemented)]
```