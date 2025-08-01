```mermaid
kanban
    globalTodo[To Do]
        global1[Design overarching ECS architecture (confirm use of Bevy ECS and patterns for scheduling and parallelism)]
        global2[Define trait-based plugin model for extensibility (e.g. standardize how automaton plugins implement `AutomatonRule`, how UI uses `RenderBridge`)]
        global3[Establish plugin registration order and dependencies (document the correct initialization sequence for all plugins)]
        global4[Standardize naming conventions across crates (consistent module names, clear semantic renaming for old components)]
        global5[Draft architecture documentation (module breakdowns, crate relationships, UML/diagrams for new design)]
        global6[Plan for concurrent plugins (ensure architecture allows multiple rule plugins active and compare results side by side)]
        global7[Set up continuous integration (CI) pipelines and testing frameworks to support the new multi-crate project structure]
        global8[Coordinate removal of legacy code (systematically eliminate or update old design artifacts in favor of new architecture)]

    globalProg[In Progress]
        global9[Refactoring codebase into multi-crate structure – **in progress**]
        global10[Iterating on ECS trait designs (`AutomatonRule` and others) with prototypes – **in progress**]

    globalRev[Review]
        global11[Architectural design review meeting (validate the redesign aligns with project goals)]
        global12[Cross-crate integration testing in review (checking that all plugins communicate correctly)]
        
    globalDone[Done]
        global13[Initial project scaffolding for new architecture completed (directories and crates created)]
        global14[Core concepts approved (ECS approach and plugin-based modular design green-lit)]
```