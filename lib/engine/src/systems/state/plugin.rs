//! Bevy plugin that injects & maintains the global resources defined in
//! `state::resources`.
//
//! Put *logic* here – the plain structs live next door in `resources.rs`.

use bevy::prelude::*;

use crate::systems::state::resources::{RuntimeFlags, Session, Settings};

/// Adds [`Settings`] + [`Session`] and a tiny housekeeping system.
pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        // --- one-time resource initialisation --------------------------------
        app.insert_resource(Settings::load())
            .insert_resource(RuntimeFlags::default())
            .init_resource::<Session>()
           // --- recurring systems -------------------------------------------
           .add_systems(Update, tick_frame);
    }
}

/// Increments the global frame counter every `Update`.
fn tick_frame(mut session: ResMut<Session>) {
    session.frame += 1;
}
