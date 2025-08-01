//! *Active* renderer – draws one quad per automaton and uploads the
//! latest CPU grid every frame.

use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    math::primitives::Rectangle,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        view::RenderLayers,
    },
    sprite::{Material2dPlugin, MeshMaterial2d},
    transform::TransformSystem,
};
use engine_core::{
    events::{AutomatonAdded, AutomatonId, AutomatonRemoved},
    prelude::{AppState, AutomataRegistry},
};
use simulation_kernel::{core::cell::CellState, grid::GridBackend};

use crate::{
    command_executor::focus_camera_on_new_auto,
    render::camera::{
        floating_origin::WorldOffset,
        systems::{WorldCamera, WORLD_LAYER},
    },
    AutomataMaterial, AutomataParams,
};

/* ───────────────────────── Resources ───────────────────────── */

#[derive(Resource, Default)]
pub struct AutomataRenderMap {
    pub map: HashMap<AutomatonId, (Entity, Handle<Image>, Handle<AutomataMaterial>)>,
}

#[derive(Resource, Deref, DerefMut)]
pub struct ActiveImageHandle(pub Handle<Image>);

/* ───────────────────────── Plugin ──────────────────────────── */

pub struct ActiveAutomataRenderPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct ActiveSet;

impl Plugin for ActiveAutomataRenderPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, ActiveSet.run_if(in_state(AppState::InGame)))
            .init_resource::<AutomataRenderMap>()
            .add_plugins(Material2dPlugin::<AutomataMaterial>::default())
            /* pipeline: add → remove → upload */
            .add_systems(Update, handle_automata_added.in_set(ActiveSet))
            .add_systems(
                Update,
                handle_automata_removed
                    .in_set(ActiveSet)
                    .after(handle_automata_added),
            )
            .add_systems(
                Update,
                upload_all_automata
                    .in_set(ActiveSet)
                    .after(handle_automata_removed),
            )
            /* uniform sync after final GlobalTransform propagation */
            .add_systems(
                PostUpdate,
                sync_all_materials
                    .after(TransformSystem::TransformPropagate)
                    .run_if(in_state(AppState::InGame)),
            )
            /* camera focus helper */
            .add_systems(
                Update,
                focus_camera_on_new_auto
                    .in_set(ActiveSet)
                    .after(handle_automata_added),
            )
            /* asset cleanup when returning to menu */
            .add_systems(OnEnter(AppState::MainMenu), clear_gpu_assets);
    }
}

/* ───────────────────────── cleanup helper ────────────────────────── */

fn clear_gpu_assets(
    mut cmd: Commands,
    mut map: ResMut<AutomataRenderMap>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<AutomataMaterial>>,
    active_img: Option<Res<ActiveImageHandle>>,
) {
    for (_, (entity, tex, mat)) in map.map.drain() {
        cmd.entity(entity).despawn();
        images.remove(tex.id());
        materials.remove(mat.id());
    }
    if active_img.is_some() {
        cmd.remove_resource::<ActiveImageHandle>();
    }
}


/* ───────────────────────── Systems ─────────────────────────── */

#[allow(clippy::too_many_arguments)]
fn handle_automata_added(
    mut commands: Commands,
    mut render_map: ResMut<AutomataRenderMap>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<AutomataMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut added_events: EventReader<AutomatonAdded>,
    automata_registry: Res<AutomataRegistry>,
    active_image_opt: Option<Res<ActiveImageHandle>>,
) {
    for ev in added_events.read() {
        let Some(info) = automata_registry.get(ev.id) else { continue };

        let (grid_w, grid_h) = match &info.grid {
            GridBackend::Dense(g) => (g.size.x, g.size.y),
            GridBackend::Sparse(_) => (512, 512),
        };

        /* 1 ░ texture -------------------------------------------------- */
        let mut image = Image::new_fill(
            Extent3d {
                width: grid_w,
                height: grid_h,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0u8],
            TextureFormat::R8Unorm,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::nearest();
        let tex = images.add(image);

        /* 2 ░ material ------------------------------------------------- */
        let mat = materials.add(AutomataMaterial {
            params: AutomataParams {
                camera_pos: Vec2::ZERO,
                zoom: 1.0,
                cell_size: info.cell_size,
                texture_size: Vec2::new(grid_w as f32, grid_h as f32),
                dead_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                alive_color: Vec4::ONE,
            },
            grid_texture: tex.clone(),
        });

        /* 3 ░ quad ----------------------------------------------------- */
        let w_world = grid_w as f32 * info.cell_size;
        let h_world = grid_h as f32 * info.cell_size;
        let world_off = info.world_offset;

        let entity = commands
            .spawn((
                MeshMaterial2d(mat.clone()),
                Mesh2d(
                    meshes
                        .add(Mesh::from(Rectangle::from_size(Vec2::ONE)))
                        .into(),
                ),
                Transform {
                    translation: Vec3::new(
                        world_off.x + w_world * 0.5,
                        world_off.y + h_world * 0.5,
                        1.0,
                    ),
                    scale: Vec3::new(w_world, h_world, 1.0),
                    ..Default::default()
                },
                GlobalTransform::default(),
                RenderLayers::layer(WORLD_LAYER.into()),
            ))
            .id();

        render_map
            .map
            .insert(ev.id, (entity, tex.clone(), mat));

        /* expose first texture for screenshots/UI */
        if active_image_opt.is_none() {
            commands.insert_resource(ActiveImageHandle(tex));
        }
    }
}

fn handle_automata_removed(
    mut commands: Commands,
    mut render_map: ResMut<AutomataRenderMap>,
    mut removed: EventReader<AutomatonRemoved>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<AutomataMaterial>>,
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
    automata_registry: Res<AutomataRegistry>,
    render_map: Res<AutomataRenderMap>,
    mut images: ResMut<Assets<Image>>,
) {
    for info in automata_registry.list() {
        let Some((_, tex, _)) = render_map.map.get(&info.id) else { continue };
        let Some(img) = images.get_mut(tex) else { continue };

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
                            let idx = (pos.y as u32
                                * img.texture_descriptor.size.width
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
    cam_q: Query<(&GlobalTransform, &Projection), With<WorldCamera>>,
    mut mats: ResMut<Assets<AutomataMaterial>>,
    render_map: Res<AutomataRenderMap>,
    offset: Res<WorldOffset>,
) {
    let Ok((xf, proj)) = cam_q.single() else { return };
    for (_, _, mat) in render_map.map.values() {
        if let Some(m) = mats.get_mut(mat) {
            /* camera position in *global* world-space (camera + floating-origin) */
            m.params.camera_pos =
                xf.translation().truncate() + offset.0.as_vec2();
            if let Projection::Orthographic(o) = proj {
                m.params.zoom = o.scale;
            }
        }
    }
}
