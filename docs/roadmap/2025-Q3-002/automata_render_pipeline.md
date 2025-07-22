## Road-map: **Active-Automata Render Pipeline**

*(goal – a stand-alone 2-D renderer that: ① lives side-by-side with the legacy `Grid2DRenderPlugin`, ② supports pan/zoom through uniforms, and ③ offers multiple colour-mapping modes)*

| Phase | Code-name              | Core outcome                                                                                                                                                                                                  | Principal modules / crates touched   |
| ----- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ |
| **0** | *Boot-strap* ✅        | Minimal “hello-quad” driven by a hard-coded R8 texture. Confirms shader → material → mesh plumbing on Bevy 0.16.                                                                                              | `io/output::rendering::active` (new) |
| **1** | *Dense Path* ✅        | • Reads `World2D::Dense`, uploads full buffer each frame.<br>• Uniforms: `camera_pos`, `zoom`, `cell_size`, `texture_size`, colours.<br>• Single colour ramp (dead / alive).                                | same                                 |
| **2** | *Sparse Path* ✅       | Differential tex updates for `World2D::Sparse` (PrevLive diff). Bench-marked on 1 M × 1 M boards @ 0.1 % density.                                                                                            | `upload.rs` helper                   |
| **3** | *Pan & Zoom*           | Camera system computes bottom-left world-coord; uniforms updated each frame. Integration test: pixel stays under cursor when panning.                                                                        | `camera_uniforms.rs`                 |
| **4** | *Colour Modes*         | Shader + UI toggle:<br>• greyscale,<br>• “heat” ramp (Viridis),<br>• categorical (8 pre-set RGBA pairs). Implemented via `u32 colour_mode` + LUT in WGSL.                                                   | shader, `ui::render_settings`        |
| **5** | *Plug-in Co-existence* | Renderer behind its own `ActiveAutomataRenderPlugin`.<br>• Adds systems to `MainSet::Render`.<br>• Uses unique render-graph label so Grid2D can still draw if enabled.                                      | `io/output::plugin`                  |
| **6** | *Performance Polish*   | • Chunked `copy_from_slice` (≤ 16 KiB) instead of per-pixel loops.<br>• Double-buffer staging to avoid “upload stall”.<br>• Texture-atlas tiling for > 4 K² boards.<br>• Benchmarks & frame-time graph.      | `upload.rs`, `profiling`             |
| **7** | *GPU-Compute Hook*     | With **gpu-compute** feature on:<br>• Skip CPU upload and bind `GpuGridTextures.read` directly to the material.<br>• Uniforms unchanged → render path agnostic of compute location.                          | interface with `engine_gpu`          |
| **8** | *Editor Tools*         | • HUD overlay for colour-scheme selector.<br>• Live per-cell probe when hovering.<br>• Film-strip capture (`png` every N frames).                                                                            | `ui/panels/world/render_tools.rs`    |
| **9** | *Docs & Examples*      | mdBook chapter “Rendering Grids”; CLI example `cargo run --example active_renderer`.                                                                                                                          | `docs/`, `examples/`                 |

---

### Phase details

#### **Phase 0 – Boot-strap** ✅
*New module tree `io/output/rendering/active/`; feature-flag `active-render` (default **on**). Exit criterion: white quad renders.*

#### **Phase 1 – Dense Path** ✅
*`upload_dense()` copies full dense grid; unit test hashes 8 × 8 checkerboard against golden PNG.*

#### **Phase 2 – Sparse Path** ✅
*Resource `PrevLive(HashSet<CellCoord>)` holds last-frame live cells; two passes clear/set changed texels.*

#### **Phase 3 – Pan & Zoom** *(next up)*
```rust
let half = Vec2::new(win.width(), win.height()) * proj.scale * 0.5;
let camera_pos = cam_tf.translation.truncate() - half;
material.params.camera_pos = camera_pos;
material.params.zoom       = 1.0 / proj.scale;
```
*Integration test: drag camera 100 units, GPU read-back confirms same sample.*

#### **Phase 4 – Colour Modes**
Uniform `mode: u32` (0 = grey, 1 = heat, 2 = categorical).  
Core WGSL:
```wgsl
let rgb = select( greyscale(v),
                  select( heat(v), categorical(v), mode == 2u ),
                  mode == 0u );
```

<!-- remaining phases unchanged -->

---

### File / module skeleton (after Phase 3)

```
io/output/
└─ rendering/
   ├─ active/
   │  ├─ material.rs         // AutomataMaterial + shader include_str!
   │  ├─ upload.rs           // dense & sparse upload systems
   │  ├─ camera_uniforms.rs  // pan/zoom updater  ← NEW
   │  ├─ plugin.rs           // ActiveAutomataRenderPlugin
   │  └─ mod.rs              // pub use plugin::ActiveAutomataRenderPlugin;
   └─ grid2d.rs              // legacy renderer unchanged
```

---

### Scheduling & System sets

| Set               | Systems added by Active plugin                              | Order                                                    |
| ----------------- | ----------------------------------------------------------- | -------------------------------------------------------- |
| `MainSet::Render` | `camera_uniforms`, `upload_dense`, `upload_sparse`          | run **before** Bevy’s `RenderSet::Queue` (texture ready) |
| `Update`          | UI panel (`render_tools`)                                   | normal                                                   |

---

### Done-when

* 2 048 × 2 048 dense Conway upload ≤ 1 ms (Ryzen 5600U)  
* 1 M sparse updates ≤ 0.6 ms  
* 60 fps pan/zoom + colour switch  
* GPU compute texture swap works  
* mdBook & example build on `wasm32-unknown-unknown`

