//! Native family qualification; stale success markers never qualify a rerun.
use bevy::prelude::*;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

#[derive(Resource)]
pub(super) struct Verification {
    pub output: PathBuf,
    pub index: usize,
    pub phase: u8,
    pub frames: u32,
    pub hold: u32,
    pub ticks: u64,
    pub capture: Option<(PathBuf, Arc<AtomicBool>)>,
}

pub(super) fn install(app: &mut App) {
    let Some(output) = std::env::var_os("SOUL_AUTOMATA_VERIFY") else {
        return;
    };
    let output = PathBuf::from(output);
    std::fs::create_dir_all(&output).expect("family evidence directory");
    if output.join("SUCCESS").exists() {
        std::fs::remove_file(output.join("SUCCESS")).expect("remove stale success");
    }
    std::fs::write(output.join("sessions.csv"), "kind,ticks,status\n").expect("evidence header");
    app.add_systems(Last, super::flow::update);
    app.insert_resource(Verification {
        output,
        index: 0,
        phase: 0,
        frames: 0,
        hold: 0,
        ticks: 0,
        capture: None,
    });
}
