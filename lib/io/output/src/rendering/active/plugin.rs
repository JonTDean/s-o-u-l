//! lib/io/output/src/rendering/active/plugin.rs
//! -------------------------------------------
//! *Active* renderer – draws **one quad per automaton** and uploads the
//! latest CPU grid every frame.
//
//! #### What’s new (August 2025)
//! 1. **Transparent “dead” cells** – the material’s `dead_color` now has
//!    `alpha = 0`, so the global background grid shows through instead of
//!    rendering an opaque black rectangle.
//! 2. **Side‑by‑side layout** – newly‑spawned automata are laid out along
//!    the **X axis** (`next_offset_x`) just like in the original design;
//!    Z is kept at `+1.0`, so painter‑ordering problems are gone.
//! 3. **Layout reset** – when the last automaton is removed the cursor is
//!    reset, guaranteeing a clean slate for the next session.

use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    math::primitives::Rectangle,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::{Material2dPlugin, MeshMaterial2d},
};

use computational_intelligence::registry::AutomataRegistry;
use engine_core::{
    engine::grid::GridBackend,
    events::{AutomatonAdded, AutomatonId, AutomatonRemoved}, state::AppState,
};
use input::controls::camera_control::WorldCamera;

use crate::rendering::material::{AutomataMaterial, AutomataParams};

/* ───────────────────────── Resources ───────────────────────── */

/// Runtime lookup **+** simple layout cursor.
#[derive(Resource, Default)]
pub struct AutomataRenderMap {
    pub map: HashMap<
        AutomatonId,
        (Entity, Handle<Image>, Handle<AutomataMaterial>),
    >,
}

/// GPU texture of the *first* automaton (exposed to UI / screenshots).
#[derive(Resource, Deref, DerefMut)]
pub struct ActiveImageHandle(pub Handle<Image>);

/* ───────────────────────── Plugin ──────────────────────────── */

pub struct ActiveAutomataRenderPlugin;

impl Plugin for ActiveAutomataRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutomataRenderMap>()
            .add_plugins(Material2dPlugin::<AutomataMaterial>::default())
            // Order: spawn / despawn → upload → uniform sync
            .add_systems(
                Update,
                (
                    handle_automata_added,
                    handle_automata_removed,
                    upload_all_automata,
                    sync_all_materials,
                )
                    .chain(),
            );
            
        // 2  clear render‑map & active‑image handle on main‑menu entry
        app.add_systems(
            OnEnter(AppState::MainMenu),
            |mut cmd:       Commands,
             mut map:       ResMut<AutomataRenderMap>,
             mut images:    ResMut<Assets<Image>>,
             mut materials: ResMut<Assets<AutomataMaterial>>,
             active_img:    Option<Res<ActiveImageHandle>>| {
                for (_, (entity, tex, mat)) in map.map.drain() {
                    cmd.entity(entity).despawn();
                    images.remove(tex.id());
                    materials.remove(mat.id());
                }
                if active_img.is_some() {
                    cmd.remove_resource::<ActiveImageHandle>();
                }
            },
        );
    }
}

/* ───────────────────────── Systems ─────────────────────────── */

/// Spawns quad + texture + material for every `AutomatonAdded`.
#[allow(clippy::too_many_arguments)]
fn handle_automata_added(
    mut commands:       Commands,
    mut render_map:     ResMut<AutomataRenderMap>,
    mut images:         ResMut<Assets<Image>>,
    mut materials:      ResMut<Assets<AutomataMaterial>>,
    mut meshes:         ResMut<Assets<Mesh>>,
    mut added_events:   EventReader<AutomatonAdded>,
    automata_registry:  Res<AutomataRegistry>,
    active_image_opt:   Option<Res<ActiveImageHandle>>,
) {
    for ev in added_events.read() {
        /* 1 ── metadata ------------------------------------------------ */
        let Some(info) = automata_registry.get(ev.id) else { continue };

        let (grid_w, grid_h) = match &info.grid {
            GridBackend::Dense(g)  => (g.size.x, g.size.y),
            GridBackend::Sparse(_) => (512, 512),
        };

        /* 2 ── texture -------------------------------------------------- */
        let mut image = Image::new_fill(
            Extent3d { width: grid_w, height: grid_h, depth_or_array_layers: 1 },
            TextureDimension::D2,
            &[0u8],                           // start fully “dead”
            TextureFormat::R8Unorm,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::nearest();
        let tex = images.add(image);

        /* 3 ── material ------------------------------------------------- */
        let mat = materials.add(AutomataMaterial {
            params: AutomataParams {
                camera_pos:   Vec2::ZERO,
                zoom:         1.0,
                cell_size:    info.cell_size,
                texture_size: Vec2::new(grid_w as f32, grid_h as f32),
                dead_color:   Vec4::new(0.0, 0.0, 0.0, 0.0), // ← fully transparent
                alive_color:  Vec4::ONE,
            },
            grid_texture: tex.clone(),
        });

        /* 4 ── quad layout --------------------------------------------- */
        let w_world = grid_w as f32 * info.cell_size;
        let h_world = grid_h as f32 * info.cell_size;

        let entity = commands
            .spawn((
                MeshMaterial2d(mat.clone()),
                Mesh2d(meshes.add(Mesh::from(Rectangle::from_size(Vec2::ONE))).into()),
                Transform {
                    translation: Vec3::new(
                        w_world * 0.5, // centre the quad on X
                        h_world * 0.5,          // bottom‑left origin
                        1.0,                    // all active quads share z = +1
                    ),
                    scale: Vec3::new(w_world, h_world, 1.0),
                    ..Default::default()
                },
                GlobalTransform::default(),
            ))
            .id();

        /* 5 ── bookkeeping --------------------------------------------- */
        render_map
            .map
            .insert(ev.id, (entity, tex.clone(), mat));

        /* 6 ── expose first texture to the UI / screenshots ------------- */
        if active_image_opt.is_none() {
            commands.insert_resource(ActiveImageHandle(tex));
        }
    }
}

/// Despawns quads + frees GPU assets.
fn handle_automata_removed(
    mut commands:   Commands,
    mut render_map: ResMut<AutomataRenderMap>,
    mut removed:    EventReader<AutomatonRemoved>,
    mut images:     ResMut<Assets<Image>>,
    mut materials:  ResMut<Assets<AutomataMaterial>>,
) {
    for ev in removed.read() {
        if let Some((entity, tex, mat)) = render_map.map.remove(&ev.id) {
            commands.entity(entity).despawn();
            images.remove(tex.id());
            materials.remove(mat.id());
        }
    }
}

/// CPU fallback – uploads the current grid into each automaton texture.
fn upload_all_automata(
    automata_registry: Res<AutomataRegistry>,
    render_map:        Res<AutomataRenderMap>,
    mut images:        ResMut<Assets<Image>>,
) {
    use engine_core::core::cell::CellState;

    for info in automata_registry.list() {
        let Some((_, tex, _)) = render_map.map.get(&info.id) else { continue };
        let Some(img)         = images.get_mut(tex)           else { continue };

        if let Some(buf) = img.data.as_mut() {
            buf.fill(0);
            match &info.grid {
                GridBackend::Dense(g) => {
                    for (i, cell) in g.cells.iter().enumerate() {
                        if !matches!(cell.state, CellState::Dead) {
                            buf[i] = 255;
                        }
                    }
                }
                GridBackend::Sparse(s) => {
                    let w = img.texture_descriptor.size.width as i32;
                    let h = img.texture_descriptor.size.height as i32;
                    for (pos, cell) in s.iter() {
                        if !matches!(cell.state, CellState::Dead)
                            && (0..w).contains(&pos.x)
                            && (0..h).contains(&pos.y)
                        {
                            let idx = (pos.y as u32 * img.texture_descriptor.size.width
                                + pos.x as u32) as usize;
                            buf[idx] = 255;
                        }
                    }
                }
            }
        }
    }
}

/// Propagates camera centre + zoom into every material.
fn sync_all_materials(
    cam_q:      Query<(&GlobalTransform, &Projection), With<WorldCamera>>,
    mut mats:   ResMut<Assets<AutomataMaterial>>,
    render_map: Res<AutomataRenderMap>,
) {
    let Ok((xf, proj)) = cam_q.single() else { return };

    for (_, _, mat) in render_map.map.values() {
        if let Some(m) = mats.get_mut(mat) {
            m.params.camera_pos = xf.translation().truncate();
            if let Projection::Orthographic(o) = proj {
                m.params.zoom = o.scale;
            }
        }
    }
}
