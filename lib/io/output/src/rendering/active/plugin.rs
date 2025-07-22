//! Active‑automata GPU renderer: uploads the simulation texture and keeps
//! the material’s camera / zoom uniforms in sync with the world camera.

use bevy::{
    asset::RenderAssetUsages,
    ecs::schedule::common_conditions::{resource_added, resource_exists},
    image::ImageSampler,
    math::primitives::Rectangle,
    prelude::*,
    render::{
        camera::Projection,
        prelude::Mesh2d,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    sprite::{Material2dPlugin, MeshMaterial2d},
};

use engine_core::{core::World2D, engine::grid::GridBackend, schedule::MainSet};
use input::controls::camera_control::{WorldCamera, ZoomInfo};

use super::{
    super::material::{AutomataMaterial, AutomataParams},
    upload::{upload_dense, upload_sparse, PrevLive},
};

/// Handle to the **R8** grid texture (one byte / cell).
#[derive(Resource)]
pub struct ActiveImageHandle(pub Handle<Image>);
/// Handle to the material so we can tweak its uniforms every frame.
#[derive(Resource)]
pub struct ActiveMaterialHandle(pub Handle<AutomataMaterial>);

/// Plugin that registers all systems required to display the *live* automata
/// layer (texture upload, quad drawing, camera synchronisation).
pub struct ActiveAutomataRenderPlugin;

impl Plugin for ActiveAutomataRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<AutomataMaterial>::default())
            .init_resource::<PrevLive>()
            // one–shot setup as soon as the world exists
            .add_systems(Update, setup_active_quad.run_if(resource_added::<World2D>))
            // per‑frame streaming + uniform updates
            .add_systems(
                Update,
                (upload_dense, upload_sparse, sync_material)
                    .in_set(MainSet::Render)
                    .run_if(resource_exists::<World2D>)
                    .run_if(resource_exists::<ActiveImageHandle>),
            );
    }
}

/* ───────────────────────── per‑frame uniforms ────────────────────────── */

/// Copies the world‑camera position & zoom factor into the material uniforms
/// every frame, so the shader can translate texture addressing into
/// world‑space coordinates.
fn sync_material(
    cam_q: Query<(&GlobalTransform, &Projection), With<WorldCamera>>,
    mut mats: ResMut<Assets<AutomataMaterial>>,
    handle: Res<ActiveMaterialHandle>,
) {
    let (cam_tf, proj) = match cam_q.single() {
        Ok(v) => v,
        Err(_) => return,
    };
    let Some(mat) = mats.get_mut(&handle.0) else { return };

    mat.params.camera_pos = cam_tf.translation().truncate();
    if let Projection::Orthographic(o) = proj {
        mat.params.zoom = o.scale;
    }
}

/* ─────────────────── one‑shot texture / quad setup ──────────────────── */

/// Creates:
/// * the *R8* texture that stores the live/dead mask,
/// * a material that samples said texture,
/// * the quad that covers the whole automata board,
/// * (if still missing) the world camera + `ZoomInfo`.
fn setup_active_quad(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<AutomataMaterial>>,
    mut images: ResMut<Assets<Image>>,
    world: Res<World2D>,
    cam_q: Query<Entity, With<WorldCamera>>,
) {
    /* ── 1. pick texture size ─────────────────────────────────────────── */
    let (w, h) = match &world.backend {
        GridBackend::Dense(g) => (g.size.x, g.size.y),
        GridBackend::Sparse(_) => (512, 512), // atlas for sparse grids
    };

    /* ── 2. allocate an 8‑bit single‑channel texture ──────────────────── */
    let mut img = Image::new_fill(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0u8],              // one byte per texel (R8Unorm)
        TextureFormat::R8Unorm,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    img.sampler = ImageSampler::nearest();
    let tex = images.add(img);

    /* ── 3. build the material (transparent dead cells) ───────────────── */
    let material = mats.add(AutomataMaterial {
        params: AutomataParams {
            camera_pos: Vec2::ZERO,
            zoom: 1.0,
            cell_size: world.cell_size,
            texture_size: Vec2::new(w as f32, h as f32),
            dead_color: Vec4::new(0.06, 0.06, 0.06, 0.0), // fully transparent
            alive_color: Vec4::splat(1.0),                // bright white
        },
        grid_texture: tex.clone(),
    });

    /* ── 4. make sure a world camera exists ───────────────────────────── */
    if cam_q.is_empty() {
        let cam_pos = Vec3::new(
            w as f32 * world.cell_size * 0.5,
            h as f32 * world.cell_size * 0.5,
            999.0,
        );
        cmd.spawn((
            Camera2d,
            Camera { order: 1, ..default() },
            Transform::from_translation(cam_pos),
            WorldCamera,
        ));
        cmd.insert_resource(ZoomInfo {
            base: 1.0,
            current: 1.0,
        });
    }

    /* ── 5. spawn the rendering quad (in front of debug layers) ───────── */
    let centre = Vec3::new(
        w as f32 * world.cell_size * 0.5,
        h as f32 * world.cell_size * 0.5,
        1.0, // Z‑depth: render above background
    );

    cmd.spawn((
        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0)).into()),
        MeshMaterial2d(material.clone()),
        Transform {
            translation: centre,
            scale: Vec3::new(
                w as f32 * world.cell_size,
                h as f32 * world.cell_size,
                1.0,
            ),
            ..default()
        },
    ));

    /* ── 6. expose handles as resources for the upload systems ────────── */
    cmd.insert_resource(ActiveImageHandle(tex));
    cmd.insert_resource(ActiveMaterialHandle(material));
}
