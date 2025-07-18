```mermaid
kanban
    scTodo[To Do]
        sc1[Define core ECS traits and components (e.g. `AutomatonRule`, `Dim`, `CellState`)]
        sc2[Implement simulation_core::core::resources (global `SimulationParams`, world size, etc.)]
        sc3[Implement simulation_core::grid::dense::storage module for 2D cell grid (dense array backend)]
        sc4[Implement simulation_core::grid::sparse::storage module for sparse cell grid (memory-efficient backend)]
        sc5[Develop double-buffer stepping system (parallel tick update to apply rules without race conditions)]
        sc6[Define simulation_core::core::events (e.g. `SimulationTickCompleted`, `RuleApplied` events to notify other systems)]
        sc7[Integrate `AutomatonRule` trait into an ECS system (apply rule each tick using current grid state)]
        sc8[Support multi-plugin concurrency (allow multiple automaton rule plugins to run side-by-side for comparison)]
        sc9[Refactor old StatePlugin into SimulationCorePlugin (rename and reorganize core logic into this crate)]
        
    scProg[In Progress]
        sc10[Implement dense grid backend (data structure and ECS resource) – **in progress**]
        sc11[Drafting `AutomatonRule` trait and sample rule implementation – **in progress**]

    scRev[Review]
        sc12[Code review for stepping logic performance (ensure parallel iteration is efficient)]
        sc13[Reviewing trait design for `AutomatonRule` (checking flexibility for various rules)]

    scDone[Done]
        sc14[Basic SimulationCore plugin structure created (crate scaffolding and Bevy plugin hookup)]
        sc15[Initial ECS component for cell state defined (foundation for grid entities/resources)]
```