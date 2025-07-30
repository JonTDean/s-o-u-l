//! Options screen – font size + autosave preferences.

use bevy::prelude::*;
use bevy_egui::egui::{self, Align2, CentralPanel};
use engine_core::{prelude::AppState, systems::state::resources::Settings};

use crate::{panels::MenuScreen, styles};

/// Local work-buffer for the Options panel.
/// We copy the persistent [`Settings`] into this struct, let the user
/// tweak the values, and only write them back when they hit “Apply”.
#[derive(Clone)]
struct Draft {
    font_size:         f32,
    autosave:          bool,
    autosave_interval: u64,
}

#[derive(Resource)]
pub struct OptionsScreen {
    draft: Draft,
}

impl FromWorld for OptionsScreen {
    fn from_world(world: &mut World) -> Self {
        let settings = world.resource::<Settings>();
        Self {
            draft: Draft {
                font_size:         settings.ui_font_size,
                autosave:          settings.autosave,
                autosave_interval: settings.autosave_interval,
            },
        }
    }
}

impl MenuScreen for OptionsScreen {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        CentralPanel::default()
            .frame(styles::fullscreen_bg())
            .show(ctx, |ui| {
                egui::Window::new("S.O.U.L. – Options")
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .resizable(false)
                    .frame(styles::fullscreen_bg())
                    .show(ui.ctx(), |ui| {
                        ui.label("UI font size");
                        ui.add(
                            egui::DragValue::new(&mut self.draft.font_size)
                                .speed(0.5)
                                .range(8.0..=64.0),
                        );

                        ui.separator();
                        ui.checkbox(&mut self.draft.autosave, "Enable autosave");
                        ui.add_enabled(
                            self.draft.autosave,
                            egui::DragValue::new(&mut self.draft.autosave_interval)
                                .range(5..=600)
                                .suffix(" s"),
                        );

                        ui.separator();
                        if ui.button("Apply & Back").clicked() {
                            next.set(AppState::MainMenu);
                        }
                    });
            });
    }
}

pub fn apply_settings_on_exit(
    screen: Res<OptionsScreen>,
    mut settings: ResMut<Settings>,
) {
    settings.ui_font_size      = screen.draft.font_size;
    settings.autosave          = screen.draft.autosave;
    settings.autosave_interval = screen.draft.autosave_interval;
    settings.save();
}
