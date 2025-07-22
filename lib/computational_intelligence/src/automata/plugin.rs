//! All automata families (symbolic + dynamical).

use bevy::prelude::*;

use crate::automata::{
    classical::plugin::ClassicalAutomataPlugin, 
    dynamical::plugin::DynamicalAutomataPlugin
};


/// One umbrella plugin so the CI root can simply do
/// `app.add_plugins(AutomataPlugin)`.
pub struct AutomataPlugin;
impl Plugin for AutomataPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ClassicalAutomataPlugin,
            DynamicalAutomataPlugin,
        ));
    }
}
