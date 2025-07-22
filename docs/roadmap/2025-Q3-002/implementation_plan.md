### 1 · Recommended execution order across *all three* road‑maps

The three plans are independent in the sense that you *can* work on them in parallel, but you will save a lot of merge‑pain (and enjoy a permanent demo at every step) if you execute them in the stacked order below. The list interleaves phases only where a downstream item **needs** an upstream result.

| Global step | Road‑map · Phase                                      | Why this must come first                                                                                                                                        |
| ----------- | ----------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **G‑0**     | **Automata 0 – Decouple**                             | Removes the hard‑coded spawner and gives every family its own plug‑in. Nothing else in the renderer/GPU plans needs to touch `engine_core` after this refactor. |
| **G‑1**     | **Render 0 – Boot‑strap**                             | Proves that a custom Material2d + quad renders correctly on your target platforms.                                                                              |
| **G‑2**     | **Render 1 – Dense Path**                             | Lets you *see* dense Conway in motion before touching any GPU code.                                                                                             |
| **G‑3**     | **Automata 1 – Regular V1**                           | Registers Rule 30 & 110 and their seed systems – content to drive the renderer.                                                                                 |
| **G‑4**     | **Render 2 – Sparse Path**                            | Now you can verify sparse Rule 30 visually.                                                                                                                     |
| **G‑5**     | **Render 3 – Pan & Zoom**                             | User‑visible feature; still independent of colour modes & GPU.                                                                                                  |
| **G‑6**     | **Render 4 – Colour Modes**                           | Piggy‑backs on the stable pan/zoom foundation.                                                                                                                  |
| **G‑7**     | **Render 5 – Plug‑in Co‑existence**                   | Makes the new renderer selectable at run‑time; legacy pipeline still available.                                                                                 |
| **G‑8**     | **GPU 0 – Ground‑work**                               | Creates the crate / feature flags without changing runtime behaviour.                                                                                           |
| **G‑9**     | **GPU 1 – Proof‑of‑Concept**                          | Gives you *one* fully GPU‑driven rule (Conway) **and** the ping‑pong textures required by Render 7.                                                             |
| **G‑10**    | **Render 7 – GPU‑Compute Hook**                       | Swaps the material’s texture handle to the GPU output; eliminates CPU upload for Conway.                                                                        |
| **G‑11**    | **GPU 2 – Multi‑rule Support**                        | Adds Rule 30/110 kernels and registry metadata (`GpuDiscrete`).                                                                                                 |
| **G‑12**    | **Automata 2–4 (CF / CS / TM)**                       | Safe to implement now – they still rely on CPU steppers.                                                                                                        |
| **G‑13**    | **Render 6 – Performance Polish**                     | Now you can measure both CPU‐path and GPU‑path perf.                                                                                                            |
| **G‑14**    | **GPU 3 – Continuous (Lenia)**                        | Requires float‑texture support already proven by Render 7.                                                                                                      |
| **G‑15**    | **Automata 5 – Dynamical V2**                         | Registers Lenia/Swarm and their GPU flags.                                                                                                                      |
| **G‑16**    | **GPU 4 – Interactivity & Sync**                      | Works with Lenia and the new seed events.                                                                                                                       |
| **G‑17**    | **Render 8 – Editor Tools**                           | Piggy‑backs on stable render + GPU features.                                                                                                                    |
| **G‑18**    | **GPU 5 – Scaling & Polish**                          | Needs all earlier optimisation tasks in place.                                                                                                                  |
| **G‑19**    | **Automata 6 / 7** & **Render 9** & **Docs/Examples** | Final metadata + UI enumeration, book chapters, CLI examples.                                                                                                   |

> **TL;DR:**
> *Automata 0 → Render 0/1 → Automata 1 → Render 2/3/4/5 → GPU 0/1 → Render 7 → GPU 2 …*

You will have a playable, visually stable build after **G‑7** and a first GPU demo after **G‑10**.

---

### 2 · Anomalies & factual corrections

| Location                                                    | Issue                                                                                       | Correction / Rationale                                                                                                                                                                                                                  |
| ----------------------------------------------------------- | ------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Render road‑map – *Scheduling & System sets*                | `bevy::sprite::SpriteSystem::RenderSprites` **does not exist** in Bevy 0.16.                | Use `bevy::sprite::RenderSet::Queue` (runs just before draw) and register with `.in_set(RenderSet::Queue).before(RenderSet::Queue)` or simpler: keep your systems in `MainSet::Render` – Bevy will run them before its own render sets. |
| Render road‑map – Phase 2 bullet “`world_update.rs` helper” | File not present in skeleton list.                                                          | Rename to `upload.rs` (matches the module skeleton) **or** create `world_update.rs` and update skeleton for consistency.                                                                                                                |
| Phase‑4 WGSL snippet                                        | `select(select(categorical(u), heat(v), mode==1u), greyscale(v), mode==0u);` mixes `u`/`v`. | The second argument of `categorical` should also be `v`.  Provide clearer version:                                                                                                                                                      | ref *
| Render Phase 3 code sample | `centre - Vec2::new(win.w, win.h)*proj.scale*0.5` ignores window height definition (`win.height()`). | Use `let half = Vec2::new(win.width(), win.height()) * 0.5 * proj.scale;` |
| Render Phase 6 | “`copy_nonoverlapping` in 4 KiB blocks to exploit L1” – 4 KiB is L1 *line* size only on some CPUs. | Clarify: *“chunked in ≤ 16 KiB blocks (fits into most L1d caches)”*. |
| GPU road‑map Phase 1‑E | says “modify `grid2d` to sample *whatever texture is in* `GpuGridTextures.read`”. The new Active renderer already does this. | Clarify that **only the legacy `grid2d`** needs the swap if you intend to keep it; Active renderer is already wired to the handle. |
| Automata road‑map – Phase 6 code block | `iter_metadata().filter(\|m\| m.family == "regular")` – closure receives `&(id, meta)`, not just `meta`. | ref ** |
| Automata Phase 4 (“seed places `"aa a "` on tape””) | Tape string has an extra space; unclear. | Use `"aaa "` (three a’s + blank) **or** note explicitly that the third character is a blank symbol. |
| All documents | Spelling of **Bevy plugin** addition: use `.add_plugins(...)` not `.add_plugin(...)` after 0.11. | Ensure consistent `.add_plugins` syntax. |


\* Example should be 
```wgsl
let rgb = select(                 // mode 0 = greyscale
    greyscale(v),
    select(                        // mode 1 = heat; mode 2 = categorical
        heat(v), 
        categorical(v), 
        mode == 2u),
    mode == 0u);
```

\*\* Example should read:  
```rust
for (id, meta) in reg.iter_metadata().filter(|(_, m)| m.family == "regular") { … }
``` 

All other cross‑references (module names, feature flags, env‑vars) are internally consistent.

---

### 3 · Cohesion check

* **Dependencies resolved** – The execution order above prevents circular requirements:  
*Render phases* rely on `World2D` and at least one CPU rule (from Automata 1) before they start showing useful output.  
*GPU phases* kick in only after the new renderer can already visualise textures, which avoids “blank screen” debugging.

* **Feature‑flag story** – `active-render`, `gpu-compute`, and optional grammar crates are non‑overlapping and can be combined freely.  Docs call this out consistently.

* **Runtime flags** – `SOUL_RENDER_MODE` and `SOUL_FORCE_CPU` have unique, non‑conflicting semantics.

* **File layout** – After Phase 3 the tree under `io/output/rendering/active` matches the skeleton; later phases only add files, never rename (except the `world_update/upload` note fixed above).

* **Terminology** – All three road‑maps use the same names for:  
* `GpuGridTextures.read/write`  
* `RuleRegistry` (singular, never pluralised)  
* `AutomataCommand::SeedPattern`  

---

### 4 · Updated snippets with fixes applied

**Render Phase 3 (camera)**

```rust
let half = Vec2::new(window.width(), window.height()) * proj.scale * 0.5;
let camera_pos = camera_transform.translation.truncate() - half;
material.params.camera_pos = camera_pos;
material.params.zoom       = 1.0 / proj.scale;
```

**Automata Phase 7 UI enumeration**

```rust
let reg = world.resource::<RuleRegistry>();
for (id, meta) in reg.iter_metadata().filter(|(_, m)| m.family == "regular") {
    // build checkbox based on `id` & `meta.friendly_name`
}
```

**Colour‑mode WGSL core**

```wgsl
fn greyscale(v: f32) -> vec3<f32> { vec3<f32>(v) }
fn heat(v: f32) -> vec3<f32>      { palette_heat(v) }
fn categorical(v: f32) -> vec3<f32> { palette_cat(v) }

var rgb: vec3<f32>;
if (mode == 0u) {
    rgb = greyscale(state);
} else if (mode == 1u) {
    rgb = heat(state);
} else {
    rgb = categorical(state);
}
```

---

### 5 · Final verdict

*The three road‑maps are factually consistent once the small corrections above are applied.*
Following the global order table will let you ship incremental, demo‑ready milestones, and each later phase builds strictly on completed earlier work.
