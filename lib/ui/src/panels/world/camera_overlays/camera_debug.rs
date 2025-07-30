use bevy::{ecs::system::{Local, Res, ResMut}, input::{keyboard::KeyCode, ButtonInput}};
use bevy_egui::{egui, EguiContexts};
use tooling::debugging::camera::CameraDebug;

pub fn debug_camera_menu(
    mut egui_ctx: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
    mut flags: ResMut<CameraDebug>,
    mut open: Local<bool>,
) {
    if keys.just_pressed(KeyCode::F3) { *open = !*open; }
    if !*open { return; }

    let ctx = egui_ctx.ctx_mut().unwrap();
    egui::Window::new("Camera debug").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Hot-keys:  F3 show/hide  •  Ctrl+Click resets all");
        });
        ui.separator();

        for (label, bit) in [
            ("Clamp camera",  CameraDebug::CLAMP),
            ("Draw bounds",   CameraDebug::DRAW_BOUNDS),
            ("Draw frustum",  CameraDebug::FRUSTUM),
            ("Freeze input",  CameraDebug::FREEZE),
            ("Log snaps",     CameraDebug::LOG_SNAP),
        ] {
            let mut v = flags.contains(bit);
            if ui.checkbox(&mut v, label).clicked() {
                flags.set(bit, v);
            }
        }
    });
}
