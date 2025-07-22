## Road‑map: **Active‑Automata Render Pipeline**

*(goal – a stand‑alone 2‑D renderer that: ① lives side‑by‑side with the legacy `Grid2DRenderPlugin`, ② supports pan/zoom through uniforms, and ③ offers multiple colour‑mapping modes)*

| Phase | Code‑name              | Core outcome                                                                                                                                                                                             | Principal modules / crates touched   |
| ----- | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ |
| **0** | *Boot‑strap*           | Minimal “hello‑quad” driven by a hard‑coded R8 texture.  Confirms shader ↔ material ↔ mesh plumbing on Bevy 0.16.                                                                                        | `io/output::rendering::active` (new) |
| **1** | *Dense Path*           | • Reads `World2D::Dense` → uploads full buffer each frame.<br>• Uniforms: `camera_pos`, `zoom`, `cell_size`, `texture_size`, colours.<br>• Single colour ramp (dead / alive).                            | same                                 |
| **2** | *Sparse Path*          | Differential tex updates for `World2D::Sparse` (prev/next HashSet).  Benchmarked on 1 M × 1 M boards with 0.1 % density.                                                                                 | `world_update.rs` helper             |
| **3** | *Pan & Zoom*           | Camera system computes bottom‑left world coord; uniforms updated each frame.  Verified by integration tests (pixel stays under cursor when panning).                                                     | `camera_uniforms.rs`                 |
| **4** | *Colour Modes*         | Shader + UI toggle for:<br>• greyscale,<br>• “heat” ramp (Viridis),<br>• categorical (8 pre‑set RGBA pairs).  Implemented via `u32 colour_mode` + LUT in WGSL.                                           | shader, `ui::render_settings`        |
| **5** | *Plug‑in Co‑existence* | Renderer behind its own Bevy plugin `ActiveAutomataRenderPlugin`.<br>• Adds its systems to `MainSet::Render`.<br>• Uses unique `RenderLabel` so Grid2D can still draw if enabled.                        | `io/output::plugin` registry         |
| **6** | *Performance Polish*   | • Chunked `copy_nonoverlapping` instead of per‑pixel loops.<br>• Double‑buffer staging to avoid “upload stall”.<br>• Texture atlas tiling for > 4 K² boards.<br>• Benchmarks & frame‑time graph.         | `upload.rs`, `profiling`             |
| **7** | *GPU‑Compute Hook*     | When the **GPU Compute** feature‑flag is active:<br>• Skip CPU upload and simply bind `GpuGridTextures.read` to the material.<br>• Uniforms remain unchanged → render path agnostic of compute location. | interface with `engine_gpu`          |
| **8** | *Editor Tools*         | • HUD overlay for colour‑scheme selector.<br>• Live preview of per‑cell values when hovering.<br>• Film‑strip capture (`png` every N frames).                                                            | `ui/panels/world/render_tools.rs`    |
| **9** | *Docs & Examples*      | mdBook chapter “Rendering Grids”; example `cargo run --example active_renderer`.                                                                                                                         | `docs/`, `examples/`                 |

---

### Phase details

#### **Phase 0 – Boot‑strap**

* **Tasks**

  1. New module tree `io/output/rendering/active/` with:

     * `material.rs` – `AutomataMaterial` (& shader include string).
     * `plugin.rs` – empty plugin that just spawns a 32×32 white‑square texture on a quad.
  2. Cargo feature `active-render` (enabled by default) toggles this plugin in `output::plugin::OutputPlugin`.
* **Exit criteria** – white quad rendered; toggling feature removes it.

---

#### **Phase 1 – Dense Path**

* **Upload system**

  ```rust
  fn upload_dense(world: Res<World2D>, mut images: ResMut<Assets<Image>>) { … }
  ```

  fast `copy_from_slice` of `world.backend::Dense` into `image.data`.
* **Shader** – R8 → `.r` mix between `dead` & `alive`.
* **Unit test** – 8×8 checkerboard pattern hashed against golden PNG.

---

#### **Phase 2 – Sparse Path**

* **Resource** `PrevLive(HashSet<CellCoord>)`.
* Two passes: clear dead, set born. Benchmarked with `bevy_profiling`.

---

#### **Phase 3 – Pan & Zoom**

* Camera query:

  ```rust
  let camera_pos = centre - Vec2::new(win.w, win.h)*proj.scale*0.5;
  material.params.camera_pos = camera_pos;
  material.params.zoom       = 1.0 / proj.scale;
  ```
* Integration test – drag camera 100 units, assert same texture sample under mouse (via GPU read‑back).

---

#### **Phase 4 – Colour Modes**

* **Uniform** `mode: u32` (0=g,1=heat,2=categorical).
* **WGSL**

  ```wgsl
  fn palette_heat(v: f32) -> vec3<f32> { … }
  let rgb = select(select(categorical(u), heat(v), mode==1u), greyscale(v), mode==0u);
  ```
* **UI** drop‑down in “Render Settings” panel sets singleton `RenderSettings` resource.

---

#### **Phase 5 – Plug‑in Co‑existence**

* `ActiveAutomataRenderPlugin` inserted **after** legacy `Grid2DRenderPlugin`.
* Uses render‑graph sub‑graph label `ActiveAutomataPass`.
* Runtime flag `SOUL_RENDER_MODE=legacy|active` chooses one.

---

#### **Phase 6 – Performance Polish**

* **Dense path** – `copy_from_slice` in 4 KiB blocks to exploit L1.
* **Sparse path** – `Vec<u32>` of dirty indices, sorted, then `unsafe memcpy`.
* Add `#[cfg(feature="profiling")]` instrumentation around upload.

---

#### **Phase 7 – GPU‑Compute Hook**

* Material already accepts any `Handle<Image>`; when `engine_gpu` registers ping‑pong textures it emits event `BindActiveTexture(Handle<Image>)`.
* Listener swaps `material.grid_texture = handle;`.
* CPU upload systems are skipped via `.run_if(!UseGpuCompute)`.

---

#### **Phase 8 – Editor Tools**

* Overlay window:

  * colour mode radio‑buttons,
  * slider for dead/alive colours (HSV picker),
  * “Capture PNG” button writing `assets/captures/frame‑####.png`.
* Requires `image` crate in `output` (behind `capture` feature).

---

#### **Phase 9 – Docs & Examples**

* **mdBook**:

  * *render/01‑overview\.md* (math & uniforms),
  * *render/02‑dense‑vs‑sparse.md*,
  * *render/03‑colour‑maps.md*.
* **Example**: loads Glider Gun scenario, press arrow keys to pan, mouse‑wheel to zoom.

---

### File / module skeleton (after Phase 3)

```
io/output/
└─ rendering/
   ├─ active/
   │  ├─ material.rs          // AutomataMaterial + shader include_str!
   │  ├─ upload.rs            // dense & sparse upload systems
   │  ├─ camera_uniforms.rs   // pan/zoom updater
   │  ├─ plugin.rs            // ActiveAutomataRenderPlugin
   │  └─ mod.rs               // pub use plugin::ActiveAutomataRenderPlugin;
   └─ grid2d.rs               // legacy renderer unchanged
```

---

### Scheduling & System sets

| Set               | Systems added by Active plugin                     | Order                                                         |
| ----------------- | -------------------------------------------------- | ------------------------------------------------------------- |
| `MainSet::Render` | `camera_uniforms`, `upload_dense`, `upload_sparse` | run **before** Bevy’s `SpriteRender` so texture is up‑to‑date |
| `Update`          | UI panel (`render_tools`)                          | normal                                                        |

(Use `.before(bevy::sprite::SpriteSystem::RenderSprites)` when registering.)

---

### Done‑when

* ✅  2 048 × 2 048 dense Conway board ≤ 1 ms upload on Ryzen 5600 U.
* ✅  1 M sparse alive cells (out of 4 G space) ≤ 0.6 ms diff update.
* ✅  60 fps pan/zoom with colour mode switch.
* ✅  Compatible with GPU compute pipeline (texture handle swap).
* ✅  mdBook and example compile under `wasm32‑unknown‑unknown`.

This phased plan brings the render path from a proof‑of‑concept quad to a production‑ready, extensible renderer that meets the project’s requirements while playing nicely with existing plugins and the forthcoming GPU‑compute back‑end.
