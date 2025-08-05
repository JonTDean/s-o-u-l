use bevy::prelude::*;
use bevy_egui::egui::{self, Align2, CentralPanel};
use engine_core::{
    prelude::AppState,
    systems::state::resources::{RuntimeFlags, Settings},
};
use crate::{components::menus::MenuScreen, styles};

pub mod plugin;

#[derive(Clone, Default)]
struct Draft {               // local edit buffer
    font_size:         f32,
    autosave:          bool,
    autosave_interval: u64,
    gpu_compute:       bool,
}

#[derive(Resource)]
pub struct OptionsMenu { draft: Draft }

impl FromWorld for OptionsMenu {
    fn from_world(w: &mut World) -> Self {
        let s = w.resource::<Settings>();
        Self { draft: Draft {
            font_size: s.ui_font_size, autosave: s.autosave,
            autosave_interval: s.autosave_interval, gpu_compute: s.gpu_compute,
        }}
    }
}

impl MenuScreen for OptionsMenu {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        CentralPanel::default().frame(styles::fullscreen_bg()).show(ctx, |ui| {
            egui::Window::new("Options").anchor(Align2::CENTER_CENTER, [0.,0.])
                .resizable(false).frame(styles::fullscreen_bg())
                .show(ui.ctx(), |ui| {
                    ui.label("UI font size");
                    ui.add(egui::DragValue::new(&mut self.draft.font_size).speed(0.5).range(8.0..=64.0));

                    ui.separator();
                    ui.checkbox(&mut self.draft.autosave, "Enable autosave");
                    ui.add_enabled(self.draft.autosave,
                        egui::DragValue::new(&mut self.draft.autosave_interval).range(5..=600).suffix(" s"));

                    ui.separator();
                    ui.checkbox(&mut self.draft.gpu_compute, "Enable GPU compute (restart required)");

                    ui.separator();
                    if ui.button("Apply & Back").clicked() {
                        next.set(AppState::MainMenu);
                    }
                });
        });
    }
}

/// commits the `Draft` back into persistent `Settings`
pub fn commit_on_exit(
    menu:    Res<OptionsMenu>,
    mut set: ResMut<Settings>,
    mut flg: ResMut<RuntimeFlags>,
) {
    set.ui_font_size      = menu.draft.font_size;
    set.autosave          = menu.draft.autosave;
    set.autosave_interval = menu.draft.autosave_interval;
    set.gpu_compute       = menu.draft.gpu_compute;
    set.save();
    flg.gpu_enabled = set.gpu_compute;
}
