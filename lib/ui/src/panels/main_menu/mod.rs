//! Plugin and module for the Main Menu UI stack (MVC pattern for menus).

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use engine::systems::state::AppState;

use super::{
    ui_runner, 
    main_menu::controller::scenario::new::ScenarioMeta
};

pub mod view;
pub mod model;
pub mod controller;

// Pull in the screen types…
use controller::{NewScenario, LoadScenario, OptionsScreen};
// and the options sub‐module itself, so we can call its exit‐handler.
use controller::options;
// The top‐level view type
use view::MainMenu;

/// Bevy Plugin that sets up all main menu screens and their UI systems.
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Enable egui
            .add_plugins(EguiPlugin::default())

            // ── MAIN MENU ────────────────────────────────────────────────
            .add_systems(OnEnter(AppState::MainMenu), |mut commands: Commands| {
                commands.insert_resource(MainMenu::default());
            })
            .add_systems(
                EguiPrimaryContextPass,
                ui_runner::<MainMenu>.run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(OnExit(AppState::MainMenu), |mut commands: Commands| {
                commands.remove_resource::<MainMenu>();
            })

            // ── NEW SCENARIO ─────────────────────────────────────────────
            .add_systems(OnEnter(AppState::NewScenario), |mut commands: Commands| {
                commands.insert_resource(NewScenario::default());
            })
            .add_systems(
                EguiPrimaryContextPass,
                ui_runner::<NewScenario>.run_if(in_state(AppState::NewScenario)),
            )
            .add_systems(OnExit(AppState::NewScenario), (
                // 1) snapshot the draft into ScenarioMeta
                |mut commands: Commands, new: Res<NewScenario>| {
                    commands.insert_resource(ScenarioMeta(new.model.clone()));
                },
                // 2) tear down the NewScenario resource
                |mut commands: Commands| {
                    commands.remove_resource::<NewScenario>();
                },
            ))

            // ── LOAD SCENARIO ────────────────────────────────────────────
            .add_systems(OnEnter(AppState::LoadScenario), |mut commands: Commands| {
                commands.insert_resource(LoadScenario::default());
            })
            .add_systems(
                EguiPrimaryContextPass,
                ui_runner::<LoadScenario>.run_if(in_state(AppState::LoadScenario)),
            )
            .add_systems(
                Update,
                controller::scenario::load::load_selected_save
                    .run_if(in_state(AppState::LoadScenario)),
            )
            .add_systems(OnExit(AppState::LoadScenario), |mut commands: Commands| {
                commands.remove_resource::<LoadScenario>();
            })

            // ── OPTIONS ──────────────────────────────────────────────────
            .add_systems(OnEnter(AppState::Options), |mut commands: Commands| {
                // Use init_resource so our FromWorld impl for OptionsScreen runs
                commands.init_resource::<OptionsScreen>();
            })
            .add_systems(
                EguiPrimaryContextPass,
                ui_runner::<OptionsScreen>.run_if(in_state(AppState::Options)),
            )
            .add_systems(OnExit(AppState::Options), (
                // First apply & save settings…
                options::apply_settings_on_exit,
                // …then tear down the OptionsScreen resource
                |mut commands: Commands| {
                    commands.remove_resource::<OptionsScreen>();
                },
            ))

            // ── IN-GAME INITIALIZATION ───────────────────────────────────
            .add_systems(
                OnEnter(AppState::InGame),
                controller::scenario::new::init_new_world
            );
    }
}