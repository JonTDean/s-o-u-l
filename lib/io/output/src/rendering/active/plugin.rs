//! Runtime renderer: keeps a textured quad in sync with *every* running
//! automaton.  Works for both dense and sparse grids.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::{Material2dPlugin, MeshMaterial2d};

use computational_intelligence::registry::{
    AutomataRegistry, AutomatonAdded, AutomatonId, AutomatonRemoved,
};
use engine_core::engine::grid::GridBackend;

use input::controls::camera_control::WorldCamera;

use crate::rendering::material::{AutomataMaterial, AutomataParams};

/* --------------------------------------------------------------------- */

/// Map **automaton ID → (quad entity, texture, material)**.
#[derive(Resource, Default)]
pub struct AutomataRenderMap {
    pub map: std::collections::HashMap<
        AutomatonId,
        (Entity, Handle<Image>, Handle<AutomataMaterial>),
    >,
}

/// GPU texture of the *primary* automaton (exposed to UI / screenshots).
#[derive(Resource, Deref, DerefMut)]
pub struct ActiveImageHandle(pub Handle<Image>);

/* --------------------------------------------------------------------- */

pub struct ActiveAutomataRenderPlugin;
impl Plugin for ActiveAutomataRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutomataRenderMap>()
            .add_plugins(Material2dPlugin::<AutomataMaterial>::default())
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
    }
}

/* ───────────────────────── systems ─────────────────────────────────── */

#[allow(clippy::too_many_arguments)]
fn handle_automata_added(
    mut commands:   Commands,
    mut render_map: ResMut<AutomataRenderMap>,
    mut images:     ResMut<Assets<Image>>,
    mut materials:  ResMut<Assets<AutomataMaterial>>,
    mut meshes:     ResMut<Assets<Mesh>>,
    mut added:      EventReader<AutomatonAdded>,
    automata:       Res<AutomataRegistry>,
    image_handle_opt: Option<Res<ActiveImageHandle>>,   // <- new
) {
    for ev in added.read() {
        let Some(info) = automata.get(ev.id) else { continue };

        /* -------- texture ------------------------------------------- */
        let (w, h) = match &info.grid {
            GridBackend::Dense(g)  => (g.size.x, g.size.y),
            GridBackend::Sparse(_) => (512, 512),
        };

        let mut image = Image::new_fill(
            Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            TextureDimension::D2,
            &[0u8],
            TextureFormat::R8Unorm,
            RenderAssetUsages::default(),   // binding + copy‑dst
        );
        image.sampler = ImageSampler::nearest();

        let tex_handle = images.add(image);

        // Convert Color to LinearRgba and then to a Vec4
        let linear_color = LinearRgba::from(info.background_color);
        let dead_color_vec4 = Vec4::from_array(linear_color.to_f32_array());

        /* -------- material ------------------------------------------ */
        let mat_handle = materials.add(AutomataMaterial {
            params: AutomataParams {
                camera_pos:   Vec2::ZERO,
                zoom:         1.0,
                cell_size:    info.cell_size,
                texture_size: Vec2::new(w as f32, h as f32),
                dead_color: dead_color_vec4,
                alive_color:  Vec4::ONE,
            },
            grid_texture: tex_handle.clone(),
        });

        /* -------- quad ---------------------------------------------- */
        let quad_w = w as f32 * info.cell_size;
        let quad_h = h as f32 * info.cell_size;
        let unit_rect = Rectangle::from_size(Vec2::ONE);   // 1×1 rectangle

        let entity = commands
            .spawn((
                MeshMaterial2d(mat_handle.clone()),
                Mesh2d(meshes.add(Mesh::from(unit_rect)).into()),
                Transform {
                    translation: Vec3::new(quad_w * 0.5, quad_h * 0.5, 1.0),
                    scale:       Vec3::new(quad_w, quad_h, 1.0),
                    ..default()
                },
                GlobalTransform::default(),
            ))
            .id();

        /* -------- bookkeeping --------------------------------------- */
        render_map.map.insert(ev.id, (entity, tex_handle.clone(), mat_handle));

        // First automaton → publish its texture for UI / screenshots.
        if image_handle_opt.is_none() {
            commands.insert_resource(ActiveImageHandle(tex_handle));
        }
    }
}

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

fn upload_all_automata(
    automata:   Res<AutomataRegistry>,
    render_map: Res<AutomataRenderMap>,
    mut images: ResMut<Assets<Image>>,
) {
    use engine_core::core::cell::CellState;

    for info in automata.list() {
        let Some((_, tex, _)) = render_map.map.get(&info.id) else { continue };
        let Some(image)       = images.get_mut(tex)            else { continue };

        if let Some(buf) = image.data.as_mut() {
            buf.fill(0);   // clear everything first

            match &info.grid {
                GridBackend::Dense(g) => {
                    for (idx, cell) in g.cells.iter().enumerate() {
                        if !matches!(cell.state, CellState::Dead) {
                            buf[idx] = 255;
                        }
                    }
                }
                GridBackend::Sparse(s) => {
                    let w = image.texture_descriptor.size.width as i32;
                    let h = image.texture_descriptor.size.height as i32;
                    for (pos, cell) in s.iter() {
                        if !matches!(cell.state, CellState::Dead)
                            && (0..w).contains(&pos.x)
                            && (0..h).contains(&pos.y)
                        {
                            let idx = (pos.y as u32 * image.texture_descriptor.size.width
                                + pos.x as u32) as usize;
                            buf[idx] = 255;
                        }
                    }
                }
            }
        }
    }
}

fn sync_all_materials(
    cam:        Query<(&GlobalTransform, &Projection), With<WorldCamera>>,
    mut mats:   ResMut<Assets<AutomataMaterial>>,
    render_map: Res<AutomataRenderMap>,
) {
    let Ok((xf, proj)) = cam.single() else { return };

    for (_, _, mat) in render_map.map.values() {
        if let Some(m) = mats.get_mut(mat) {
            m.params.camera_pos = xf.translation().truncate();
            if let Projection::Orthographic(o) = proj {
                m.params.zoom = o.scale;
            }
        }
    }
}
