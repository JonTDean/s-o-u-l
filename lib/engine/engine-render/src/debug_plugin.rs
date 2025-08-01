use bevy::prelude::*;
use engine_core::prelude::*;
use simulation_kernel::{core::cell::CellState, grid::GridBackend};

use crate::{render::AutomataRenderMap, AutomataMaterial, WorldCamera};

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (dump_registry, dump_render_map.after(dump_registry), dump_camera.after(dump_render_map)),
        );
    }
}

fn dump_registry(reg: Res<AutomataRegistry>) {
    for a in reg.list() {
        let (live, total) = match &a.grid {
            GridBackend::Dense(g)  => {
                let live = g.cells.iter().filter(|c| !matches!(c.state, CellState::Dead)).count();
                (live, g.cells.len())
            }
            GridBackend::Sparse(s) => (s.map.len(), usize::MAX),
        };
        info!(target: "soul::auto", id=?a.id, ?a.name, %live, %total, "registry");
    }
}

fn dump_render_map(
    reg:     Res<AutomataRegistry>,
    map:     Res<AutomataRenderMap>,
    images:  Res<Assets<Image>>,
    mats:    Res<Assets<AutomataMaterial>>,
) {
    for (id, (_, tex, mat)) in &map.map {
        let live_tex = images.get(tex).map(|img| {
            img.data.as_ref().map(|d| d.iter().filter(|&&b| b != 0).count()).unwrap_or(0)
        });
        let zoom = mats.get(mat).map(|m| m.params.zoom);
        let name = reg.get(*id).map(|a| a.name.clone()).unwrap_or("<gone>".into());
        info!(target: "soul::render", id=%id.0, %name, live_tex=?live_tex, zoom=?zoom, "quad/material");
    }
}

fn dump_camera(cam_q: Query<(&Transform, &Projection), With<WorldCamera>>) {
    if let Ok((tf, Projection::Orthographic(o))) = cam_q.single() {
        info!(target: "soul::cam",
              pos = ?tf.translation.truncate(),
              scale = %o.scale,
              "world-camera");
    }
}
