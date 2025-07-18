```mermaid
kanban
    appTodo[To Do]
        app1[Refactor main application to utilize modular plugins (load DevUtils -> SimulationCore -> Automata Plugins -> UI -> Networking in order)]
        app2[Extract app setup into main_application::app::builder (clean up `main.rs` by moving builder helpers to `app.rs`)]
        app5[Integrate networking startup (conditionally launch server or client components if networking enabled)]
        app6[Handle cross-plugin events (e.g. main app routes simulation events to UI or networking as needed)]
        app7[Remove deprecated monolithic logic after refactor (delete or disable any old code replaced by new plugin structure)]
        app8[Verify naming consistency and clarity in main application (apply semantic renaming where old names persist)]
        app14[Project restructured into multi-crate layout (main application now orchestrates new plugin crates)]
        
    appProg[In Progress]
        app10[Testing multi-plugin integration in a sandbox (ensuring DevUtils, SimulationCore, UI work together) – **in progress**]

    appRev[Review]
        app11[Team review of application start-up sequence (verify no missing plugin dependencies)]
        app12[Integration test review – simulation, UI, networking interplay (checking for runtime errors)]
        app3[Implement config file parsing (e.g. select automaton type, set grid size, headless mode)]
        app4[Ensure proper system orchestration (set up Bevy run schedule so simulation update runs after inputs, before rendering)]
        app9[Modular plugin loading code in development (replacing old init logic) – **in progress**]
        

    appDone[Done]
        app13[Basic Bevy app initialized with window (application bootstrap completed)]
```