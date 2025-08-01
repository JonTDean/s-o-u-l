use bevy::app::{App, Plugin};

use crate::prelude::AutomataRegistry;

pub struct EngineCorePlugin;
impl Plugin for EngineCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutomataRegistry>();
    }
}