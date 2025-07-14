```text
┌────────────┐
│ MainApp    │
└────┬───────┘
     │ adds
┌────▼───────────────────────┐
│ DevUtilsPlugin (quit/log)  │
└────┬───────────────────────┘
     │ adds
┌────▼───────────────────────┐
│ CAEnginePlugin             │  ← grid, stepper, common events
└─┬──┴──┬──┬─────────────────┘
  │     │  │ registers
  │     │  ├──────────────┐
  │     │  │              │
  │     ▼  ▼              ▼
┌─▼────┐ ┌─▼────┐ ┌───────▼───┐
│Type1 │ │Type2 │ │Type3      │  ← each supplies AutomatonRule impl + params UI
│Plugin│ │Plugin│ │Plugin     │
└──────┘ └──────┘ └───────────┘
```
   
- At run‑time you can load multiple plugins concurrently (e.g. split‑screen comparing Type 1 vs Type 2).
- Each Type‑plugin registers:
    1. a concrete AutomatonRule instance,
    2. a default parameter blob (serde_json::Value),
    3. a colour palette,
    4. an egui panel section (optional trait InspectableParameters).