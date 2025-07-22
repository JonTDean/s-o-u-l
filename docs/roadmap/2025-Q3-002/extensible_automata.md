## Road‑map: **Extensible Automata Plugin Layer**

*(moving all rule/seed logic into the `computational_intelligence::automata` tree and keeping `engine_core` agnostic)*

| Phase | Code‑name              | Core outcome                                                                                                                                                                                                                                                                                 | Key sub‑crates / modules touched                                                             |
| ----- | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| **0** | *Decouple*             | No automaton‑specific code remains in `engine_core`.  `AutomataCommand::SeedPattern` is still emitted from UI, but **all** handlers live in the relevant automata plugins.                                                                                                                   | `engine_core::systems::spawner` (delete); `computational_intelligence::registry` (+metadata) |
| **1** | *Regular V1*           | `RegularAutomataPlugin` handles:<br>• Rule (de)registration in `RuleRegistry`.<br>• `StepperPlugin` attach/detach.<br>• `seed_rule30`, `seed_rule110` event system.<br>Unit tests for registry + seeds.                                                                                      | `ci::automata::classical::regular::*`                                                        |
| **2** | *Context‑Free*         | New `ContextFreeAutomataPlugin` for push‑down / L‑systems:<br>• Register `ll1:balanced_parens`, `lsys:koch`.<br>• Provide **string‑based** seed helpers (inject tape/axiom into `World2D` memory field).                                                                                     | `ci::automata::classical::contextless::*`                                                    |
| **3** | *Context‑Sensitive*    | `ContextSensitiveAutomataPlugin` registers LBA demo; seeds tape when pattern requested.                                                                                                                                                                                                      | `ci::automata::classical::contextful::*`                                                     |
| **4** | *Turing*               | `TuringAutomataPlugin` registers `tm:replace_a` rule; seed writes initial tape; CPU stepper.                                                                                                                                                                                                 | `ci::automata::classical::turing::*`                                                         |
| **5** | *Dynamical V2*         | `DynamicalAutomataPlugin` split into sub‑plugins:<br>• `LifePlugin` (Conway, Dean) – with `seed_glider`, `seed_gun`.<br>• `LeniaPlugin` – `seed_orbium_blob`.<br>• `SwarmPlugin` – ant colony, boids seeding.<br>Decides at runtime whether to use CPU or GPU stepper (Phase 1 of GPU plan). | `ci::automata::dynamical::*`                                                                 |
| **6** | *Metadata & Queries*   | Extend `RuleRegistry::Entry` → `{ boxed_rule, family, has_gpu_impl, default_seed_fn }`.  Provide helper `RuleRegistry::spawn_default(&id, &mut World2D)`.                                                                                                                                    | `ci::registry`                                                                               |
| **7** | *Scenario Integration* | UI “New Scenario” screen enumerates rule IDs **directly from `RuleRegistry`** instead of hard‑coded arrays.  `AutomataPanel` HUD shows friendly names via metadata.                                                                                                                          | `io/output::ui::*`                                                                           |
| **8** | *Docs & Examples*      | mdBook chapter *“Writing a new Automata Plugin”*; two cargo examples: `cargo run --example new_rule_cpu` and `new_rule_gpu`.                                                                                                                                                                 | root docs, `examples/`                                                                       |

---

### Phase details & deliverables

#### **Phase 0 – Decouple**

* **Tasks**

  1. Delete `engine_core::systems::spawner`.
  2. Introduce `AutomataPattern` trait (`fn seed(&self, world: &mut World2D)`).
  3. Add `fn register_with_seed(id, rule, seed_fn)` helper in `RuleRegistry`.
* **Exit criteria**
  *`cargo check` succeeds; no `match id` blocks left in `engine_core`.*

---

#### **Phase 1 – Regular V1**

* **Plugin build()**

  ```rust
  fn build(&self, app: &mut App) {
      let mut reg = app.world_mut().remove_resource::<RuleRegistry>().unwrap_or_default();
      reg.register_with_seed("wolfram:rule30", Rule30::boxed(), seed_rule30);
      reg.register_with_seed("wolfram:rule110", Rule110::boxed(), seed_rule110);
      app.insert_resource(reg);

      app.add_plugins((
          StepperPlugin::<Rule30>{ rule: Rule30, params: Value::Null },
          StepperPlugin::<Rule110>{ rule: Rule110, params: Value::Null },
      ))
      .add_systems(Update, Self::on_seed_event);
  }
  ```
* **`on_seed_event` system** – listens for `AutomataCommand::SeedPattern` and calls the registry’s `spawn_default`.
* **Unit test** – create 32×1 `World2D`, send event, assert centre row becomes alive.

---

#### **Phase 2 – Context‑Free**

* Register PDA (`pushdown:anbn`) and L‑system (`lsys:koch`) with their own seed closures.
* Provide optional `StringTape` component for visualising stack/tape in HUD.
* Add example scenario file.

---

#### **Phase 3 – Context‑Sensitive**

* Similar pattern: LBA function registered; seed places input string `"aaabbbccc"` into per‑cell memory.
* Stepper is still CPU for now.

---

#### **Phase 4 – Turing**

* Register `tm:replace_a`; seed places `"aa a "` on tape (blank at end).
* Add simple visualiser system (render head position as different colour).

---

#### **Phase 5 – Dynamical V2**

* Split plugin (`LifePlugin`, `LeniaPlugin`, `SwarmPlugin`).
* Each sub‑plugin:

  * registers rules;
  * chooses CPU‑stepper **or** writes `rule.has_gpu_impl = true` so the GPU pipeline can pick it up;
  * seeds default patterns.

---

#### **Phase 6 – Metadata & Queries**

```rust
pub struct RuleMeta {
    pub family: &'static str,           // "regular", "life", …
    pub friendly_name: &'static str,
    pub has_gpu_impl: bool,
    pub default_seed: fn(&mut World2D),
}
```

* Extend `RuleRegistry` to store `RuleMeta` next to boxed rule.
* Provide `iter_metadata()` used by UI to auto‑populate check‑boxes.

---

#### **Phase 7 – Scenario Integration**

* Replace hard‑coded arrays in `ui::panels::main_menu::controller::new` with:

  ```rust
  let reg = world.resource::<RuleRegistry>();
  for (id, meta) in reg.iter_metadata().filter(|m| m.family == "regular") { … }
  ```
* HUD reads `friendly_name`.

---

#### **Phase 8 – Docs & Examples**

* **mdBook pages**:

  * *02‑plugins/01‑architecture.md* – explains family ↔ rule ↔ seed hierarchy.
  * *02‑plugins/02‑adding‑gpu‑rule.md* – cross‑links to GPU road‑map.
* **Examples**

  * `examples/new_rule_cpu.rs` – registers Rule 90 in 20 lines.
  * `examples/new_rule_gpu.rs` – Lenia‑variant with custom WGSL.

---

### Cross‑cutting concerns

* **Namespacing** – keep rule strings short but unique (`family:name` or `life:conway`).
* **Testing** – each plugin owns a `#[cfg(test)]` module with deterministic step check.
* **Cargo features** – `--features "ctxt-free"` can optionally exclude heavy grammar crates.
* **Scheduling** – all seed systems go into `MainSet::Logic` but run *before* steppers via `.before(step_world::<R>)`.

---

### Final architecture graph (runtime)

```
+-----------------------------------------------------+
| Bevy App                                            |
|                                                     |
|  +-- AutomataPlugin (root) ----------------------+  |
|  |                                               |  |
|  |  +-- ClassicalAutomataPlugin ----------------+|  |
|  |  |  + RegularAutomataPlugin  (Phase 1)       ||  |
|  |  |  + ContextFreeAutomataPlugin  (Phase 2)   ||  |
|  |  |  + ContextSensitiveAutomataPlugin (P3)    ||  |
|  |  |  + TuringAutomataPlugin        (P4)       ||  |
|  |  +-------------------------------------------+|  |
|  |                                               |  |
|  |  +-- DynamicalAutomataPlugin ----------------+|  |
|  |  |  + LifePlugin        (P5)                 ||  |
|  |  |  + LeniaPlugin       (P5)                 ||  |
|  |  |  + SwarmPlugin       (P5)                 ||  |
|  |  +-------------------------------------------+|  |
|  +-----------------------------------------------+  |
+-----------------------------------------------------+
```

After Phase 8 every rule lives **inside its own family plugin**, has a declarative metadata block, registers itself in one line, and supplies an optional seeding function. The engine core no longer needs to change when new automata are introduced, meeting the extensibility goal.
