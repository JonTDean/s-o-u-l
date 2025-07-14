# v0‑0‑2_menus Kanban – UI Menus & Scenario Flow

> **Sprint goal:** implement an out‑of‑game menu flow (Start ▶ New Scenario / Load Scenario / Options) that boots *before* the grid is instantiated.  Integrate Bevy Egui and hook menu selections into existing engine setup.

---

## Backlog

* **Research** – look at existing Bevy Egui menu patterns and decide on a minimal app‑state enum structure.
* **Design wireframes** for Start / New‑Scenario wizard / Load / Options panels (Figma or ASCII mock‑ups).
* **Save‑file format** – decide JSON vs. RON; include grid size, backend type, rule‑ID, params, seed pattern.

---

## In Progress

* **AppState enum & schedule**
  `MenuState { Start, NewWizard, Load, Options }` plus a `Running(WorldId)` state – wiring via `States` plugin.
* **Start menu panel**
  three buttons → push new state; fade‑in animation.

---

## Review / PR

* **New‑Scenario wizard step 1** – grid size sliders, cell‑size, backend radio (dense/sparse).
* **Wizard step 2** – rule dropdown, JSON param text‑box with live validation.
* **Create‑world hook** – consumes wizard model, spawns `World2D` resource & switches state to `Running`.
* **Options panel** – basic settings saved to `assets/config.ron` (screen size, UI scale).

---

## Done

*(empty – move tickets here when merged)*
