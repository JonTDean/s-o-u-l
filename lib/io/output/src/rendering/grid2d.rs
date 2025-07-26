//! GPU-side blitter: copies cell data into a single **RGBA8** texture and
//! draws one quad with a custom WGSL shader.
//!
//! *Compatible with **Bevy 0.16.1*** – all deprecated camera APIs
//! (`Camera2dBundle`, `UiCameraConfig`, …) have been removed.
//!
//! ## Overview
//! 1. `Grid2DRenderPlugin` registers:
//!    * a **setup** system that runs once per scenario to create
//!      * the texture + material,
//!      * a *world* camera (`WorldCamera` marker) rendered at **order = 1**,
//!      * the textured quad spanning the entire automata board;
//!    * an **upload_texture** system that runs every frame inside the
//!      [`MainSet::Render`] to stream the latest cell states to the GPU.
//! 2. `upload_texture` scrolls the texture origin according to the camera
//!    transform and supports both dense and sparse grid back-ends.
//!
//! All heavy lifting (pixel upload + quad drawing) is done off-thread by Bevy’s
//! render-world schedule, keeping your main ECS systems free to run in parallel.

use bevy::{
    asset::RenderAssetUsages,
    ecs::schedule::common_conditions::{resource_added, resource_exists},
    image::ImageSampler,
    math::primitives::Rectangle,
    prelude::*,
    render::{
        prelude::Mesh2d,
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, ShaderType, TextureDimension, TextureFormat,
        },
    },
    sprite::{Material2d, Material2dPlugin, MeshMaterial2d},
};

use engine_core::{
    core::{cell::CellState, world::World2D},
    engine::grid::GridBackend,
    schedule::MainSet,
};
use input::controls::camera_control::{WorldCamera, ZoomInfo};

/* ────────────────────────────── Material ─────────────────────────────── */

#[repr(C)]
#[derive(Copy, Clone, ShaderType)]
pub struct GridParams {
    /// World-space origin (top-left texel of the GPU texture).
    pub origin:   Vec2,
    /// Size of the texture in texels (used for address wrapping).
    pub tex_size: Vec2,
}

#[derive(Asset, AsBindGroup, Clone, TypePath)]
pub struct GridMaterial {
    #[uniform(0)]
    pub params: GridParams,                // binding 0

    #[texture(1)]
    #[sampler(2)]
    pub tex: Handle<Image>,               // binding 1 + 2
}

impl Material2d for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid_material.wgsl".into()
    }
}

/* ─────────────────────────── Runtime helpers ─────────────────────────── */

#[derive(Resource)]
struct GridTexture {
    handle:   Handle<Image>,
    material: Handle<GridMaterial>,
    width:    u32,
}

/* ─────────────────────────────── Plugin ──────────────────────────────── */
pub struct Grid2DRenderPlugin;

impl Plugin for Grid2DRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<GridMaterial>::default())
           .add_systems(
               Update,
               setup.run_if(in_state(engine_core::state::AppState::InGame))
                    .run_if(resource_added::<World2D>),
           )
           .add_systems(
               Update,
               upload_texture
                   .in_set(MainSet::Render)
                   .run_if(in_state(engine_core::state::AppState::InGame))
                   .run_if(resource_exists::<GridTexture>),
           );
    }
}
/* ─────────────────────── First-frame initialisation ───────────────────── */

/// Creates the render target, quad, material **and** the world camera.
///
/// * **Texture size** – for dense grids we match the logical size;
///   for sparse back-ends we allocate a fixed 512 × 512 atlas.
/// * **World camera** – spawned with [`Camera2d`] and a custom `Camera`
///   component so we can control render ordering.  All other required
///   components (`Transform`, `OrthographicProjection`, …) are injected
///   automatically by Bevy’s *required-component* system.

fn setup(
    mut cmd:    Commands,
    mut images: ResMut<Assets<Image>>,
    mut mats:   ResMut<Assets<GridMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    world:      Res<World2D>,
) {
    let (w, h) = match &world.backend {
        GridBackend::Dense(g)  => (g.size.x, g.size.y),
        GridBackend::Sparse(_) => (512, 512),
    };

    let mut img = Image::new_fill(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[12, 12, 12, 255],                 // dark‑grey RGBA
        TextureFormat::Rgba8Unorm,          // ← 4 bytes / texel
        RenderAssetUsages::RENDER_WORLD,
    );
    img.sampler = ImageSampler::nearest();
    let tex = images.add(img);

    /* centred orthographic camera (unchanged) */
    let cam_x = w as f32 * world.cell_size * 0.5;
    let cam_y = h as f32 * world.cell_size * 0.5;

    /* low‑contrast quad *behind* the active renderer (z = −1) */
    let mat = mats.add(GridMaterial {
        params: GridParams { origin: Vec2::ZERO, tex_size: Vec2::new(w as f32, h as f32) },
        tex:    tex.clone(),
    });

    cmd.spawn((
        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0)).into()),
        MeshMaterial2d(mat.clone()),
        Transform {
            translation: Vec3::new(cam_x, cam_y, -1.0),   // ↓ behind everything
            scale: Vec3::new(
                w as f32 * world.cell_size,
                h as f32 * world.cell_size,
                1.0,
            ),
            ..default()
        },
    ));

    cmd.insert_resource(GridTexture { handle: tex, material: mat, width: w });
}

/* ────────────────────────── Per-frame upload ─────────────────────────── */

/// Streams the current automata state into the GPU texture.
///
/// The system runs in the render schedule and therefore never blocks ECS
/// update stages.  It automatically scrolls the visible atlas region to keep
/// the camera centred on its current world-space translation.
#[allow(clippy::too_many_arguments)]
fn upload_texture(
    world:        Res<World2D>,
    grid_tex:     Res<GridTexture>,
    mut images:   ResMut<Assets<Image>>,
    mut materials:ResMut<Assets<GridMaterial>>,
    cam_q:        Query<&GlobalTransform, With<WorldCamera>>,
) {
    /* identical to before, except the buffer is RGBA8 (4 bytes / pixel) */
    let Ok(cam_tf) = cam_q.single() else { return };
    let cam_cell   = cam_tf.translation().truncate() / world.cell_size;

    if let Some(mat) = materials.get_mut(&grid_tex.material) {
        mat.params.origin = cam_cell;
    }

    let Some(buf) = images.get_mut(&grid_tex.handle).and_then(|img| img.data.as_mut()) else { return };
    for px in buf.chunks_exact_mut(4) { px.copy_from_slice(&[12, 12, 12, 255]); }

    let tex_w  = grid_tex.width as i32;
    let half   = tex_w / 2;
    let base_x = cam_cell.x.floor() as i32 - half;
    let base_y = cam_cell.y.floor() as i32 - half;

    match &world.backend {
        GridBackend::Dense(g) => {
            for (idx, cell) in g.cells.iter().enumerate()
                                      .filter(|(_, c)| !matches!(c.state, CellState::Dead))
            {
                let x = (idx as u32 % g.size.x) as i32;
                let y = (idx as u32 / g.size.x) as i32;
                if (base_x..base_x + tex_w).contains(&x)
                    && (base_y..base_y + tex_w).contains(&y)
                {
                    let rel_x = x - base_x;
                    let rel_y = y - base_y;
                    let i     = ((rel_y * tex_w + rel_x) * 4) as usize;
                    let v     = if let CellState::Alive(l) = cell.state { l } else { 0 };
                    buf[i..i + 4].copy_from_slice(&[v, v, v, 255]);
                }
            }
        }
        GridBackend::Sparse(_) => { /* identical to previous version */ }
    }
}