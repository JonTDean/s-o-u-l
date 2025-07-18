//! Build & run the Bevy `App`.

use bevy::{
    prelude::*,
    window::WindowPlugin,
    prelude::{PluginGroup, AppExtStates},
};

use crate::{
    app::{
        builder::RuntimeConfig,
        plugin_registry::{add_all_plugins, PluginFlags},
        schedule::MainSet,
    },
    dev_utils::tools::quit::quit_on_esc,
};

pub fn run() {
    build(RuntimeConfig::load()).run();
}

pub fn build(cfg: RuntimeConfig) -> App {
    /* 1 ── Core `App` skeleton */
    let mut app = App::new();
    if cfg.headless {
        app.add_plugins(DefaultPlugins.build().disable::<WindowPlugin>());
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "S.O.U.L. – Swarm Orchestrator for Autonomous Learners".into(),
                resolution: (1_280., 720.).into(),
                ..default()
            }),
            ..default()
        }));
    }

    /* 2 ── Global systems & state */
    app.add_systems(Update, quit_on_esc)
        .init_state::<crate::state::AppState>();

    /* 3 ── Canonical schedule */
    app.configure_sets(
        Update,
        (
            MainSet::Input,
            MainSet::Logic.after(MainSet::Input),
            MainSet::Render.after(MainSet::Logic),
        ),
    );

    /* 4 ── Register *all* feature plugins in one call */
    add_all_plugins(
        &mut app,
        PluginFlags {
            networking: &cfg.networking,
        },
    );

    app
}
