//! The automata lab shell: rail, drawers, channel tabs, stage, inspector, transport, presenter.
//!
//! Systems here only read snapshots and write `ChannelCommand`/`AutomataCommand`;
//! `Workbench` and `LeniaSimulation` are read to paint models and show parameters.
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use engine_common::controls::camera::{manager::stage::StageRect, motion::freecam::FreeCam};
use engine_core::{
    automata::{
        GpuGridSlice,
        channel::{ChannelCommand, ChannelId, Channels, metrics::Metrics},
        lenia::LeniaSimulation,
    },
    events::{AutomataCommand, DebugSeedSquare},
    prelude::{AppState, RuleRegistry},
};
use models::workbench::{
    catalog::{Catalog, specimens::Specimens},
    operations::channels::ConfigureSession,
    plugin::Workbench,
};
use std::collections::HashSet;
pub mod chrome;
pub mod inspector;
pub mod shortcuts;
pub mod stage;
pub mod state;
#[cfg(test)]
mod tests;

/// Everything the UI wants to change this frame, flushed once at the end.
#[derive(Default)]
pub struct Effects {
    /// Channel commands for the backends.
    pub commands: Vec<ChannelCommand>,
    /// Spawn and clear requests.
    pub automata: Vec<AutomataCommand>,
    /// Experiment settings for CPU sessions.
    pub configure: Vec<ConfigureSession>,
    /// DOI link to open in the browser.
    pub open_url: Option<String>,
    /// Write the debug centre square into the selected GPU slice.
    pub mark_centre: bool,
}

/// Read-only inputs to every lab panel.
#[derive(SystemParam)]
pub struct LabData<'w, 's> {
    /// Open channels and the selection.
    pub channels: Res<'w, Channels>,
    /// Headline metric per channel.
    pub metrics: Res<'w, Metrics>,
    /// Authored specimen copy.
    pub specimens: Res<'w, Specimens>,
    /// Registered CPU families.
    pub catalog: Res<'w, Catalog>,
    /// Registered GPU rules.
    pub rules: Res<'w, RuleRegistry>,
    /// CPU sessions, read to paint models.
    pub bench: Res<'w, Workbench>,
    /// Lenia owners, read to show parameters.
    pub lenia: Query<'w, 's, (&'static ChannelId, &'static LeniaSimulation)>,
    /// GPU atlas slices per channel.
    pub slices: Query<'w, 's, (&'static ChannelId, &'static GpuGridSlice)>,
    /// Present while the perspective free camera is active.
    pub freecam: Query<'w, 's, (), With<FreeCam>>,
}

impl LabData<'_, '_> {
    /// Ids the library may spawn: registered CPU kinds plus registered GPU rules.
    pub fn available(&self) -> HashSet<String> {
        let mut ids: HashSet<String> = self.catalog.0.iter().map(|k| k.id().to_string()).collect();
        ids.extend(self.rules.ids().cloned());
        ids
    }
}

/// Registers the lab; added by `OutputPlugin` (Task 16).
pub struct LabPlugin;

impl Plugin for LabPlugin {
    fn build(&self, app: &mut App) {
        use chrome::drawers::{Comparison, Files, project};
        app.init_resource::<state::LabState>()
            .init_resource::<Files>()
            .init_resource::<Comparison>()
            .init_resource::<engine_core::automata::lenia::ReadbackBusy>()
            .add_systems(PreUpdate, swallow_escape.after(bevy::input::InputSystem))
            .add_systems(Update, project::claim_mailbox)
            .add_systems(
                EguiPrimaryContextPass,
                chrome::drawers::compare::capture::poll
                    .before(draw)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                project::persist.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                draw.after(crate::theme::ThemeSet)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

/// Keep Escape from also opening the pause menu when the lab uses it.
pub fn swallow_escape(lab: Res<state::LabState>, mut keys: ResMut<ButtonInput<KeyCode>>) {
    if lab.consumes_escape() {
        keys.clear_just_pressed(KeyCode::Escape);
    }
}

/// Keys pressed this frame and whether a text field has keyboard focus.
///
/// `ctx.input` holds the context lock, so the focus query must run outside its closure;
/// calling back into `ctx` from inside deadlocks the egui pass.
pub fn frame_input(ctx: &egui::Context) -> (Vec<egui::Key>, bool) {
    let keys = ctx.input(|i| {
        i.events
            .iter()
            .filter_map(|e| match e {
                egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } => Some(*key),
                _ => None,
            })
            .collect::<Vec<_>>()
    });
    (keys, ctx.wants_keyboard_input())
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut contexts: EguiContexts,
    mut lab: ResMut<state::LabState>,
    data: LabData,
    mut stage_rect: ResMut<StageRect>,
    time: Res<Time>,
    mut commands: EventWriter<ChannelCommand>,
    mut automata: EventWriter<AutomataCommand>,
    mut debug: EventWriter<DebugSeedSquare>,
    mut configure: EventWriter<ConfigureSession>,
    mut drawers: chrome::drawers::DrawerState,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    // The theme's fonts bind one pass after install; drawing earlier panics in epaint.
    if !crate::theme::fonts::ready(ctx) {
        return;
    }
    let mut fx = Effects::default();
    let (keys, typing) = frame_input(ctx);
    lab.settle_launch_library(!data.channels.rows.is_empty());
    let freecam = !data.freecam.is_empty();
    for shortcut in shortcuts::collect(&keys, typing) {
        shortcuts::apply(shortcut, &mut lab, &data.channels, freecam, &mut fx);
    }
    let dt = time.delta_secs();
    lab.note_pointer(ctx.input(|i| i.pointer.delta() != egui::Vec2::ZERO), dt);
    match lab.mode {
        state::LabMode::Workbench => {
            chrome::rail::show(ctx, &mut lab, &data, &mut fx);
            chrome::tabs::show(ctx, &mut lab, &data, &mut fx);
            chrome::transport::show(ctx, &data, &mut fx);
            inspector::show(ctx, &mut lab, &data, &mut fx);
            *stage_rect = stage::show(ctx, &mut lab, &data, &mut fx, typing);
            chrome::drawers::show(ctx, &mut lab, &data, &mut drawers, &mut fx);
        }
        state::LabMode::Presenter => {
            *stage_rect = stage::presenter::show(ctx, &mut lab, &data, &mut fx, typing)
        }
    }
    stage::overlay::toast(ctx, &mut lab, dt);
    if let Some(url) = fx.open_url.take() {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
    if fx.mark_centre
        && let Some((_, slice)) = data
            .slices
            .iter()
            .find(|(id, _)| Some(**id) == data.channels.selected)
    {
        debug.write(DebugSeedSquare {
            slice: slice.clone(),
            value: 255,
        });
    }
    commands.write_batch(fx.commands);
    automata.write_batch(fx.automata);
    configure.write_batch(fx.configure);
}
