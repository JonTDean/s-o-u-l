use bevy::prelude::*;

pub fn quit_on_esc(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        // send a “successful” exit event
        exit.write(AppExit::Success);
    }
}
