//! Opt-in, fail-closed qualification through real workbench commands and GPU reads.
use bevy::prelude::*;
use simulation_kernel::core::lenia::LeniaParams;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};
mod capture;
mod flow;

#[derive(Resource)]
struct Verification {
    output: PathBuf,
    phase: u8,
    frame: u32,
    hold_since: u32,
    pending: bool,
    validated: bool,
    capture: Option<(PathBuf, Arc<AtomicBool>)>,
    first_owner: Option<Entity>,
    expected: Vec<u8>,
    tick: u64,
    epoch: u64,
    params: LeniaParams,
    /// Channel pass step and the frame it started.
    channel_pass: (u8, u32),
    /// Target tick before the channel Step.
    channel_tick: u64,
}

pub(super) fn install(app: &mut App) {
    let Some(output) = std::env::var_os("SOUL_LENIA_VERIFY") else {
        return;
    };
    let output = PathBuf::from(output);
    std::fs::create_dir_all(&output).expect("Lenia evidence directory");
    let success = output.join("SUCCESS");
    if success.exists() {
        std::fs::remove_file(success).expect("remove previous qualification result");
    }
    std::fs::write(output.join("image-checks.json"), "[]").expect("image checks header");
    std::fs::write(
        output.join("readbacks.csv"),
        "phase,epoch,tick,alive,density_sum,fnv64\n",
    )
    .expect("Lenia evidence header");
    app.insert_resource(Verification {
        output,
        phase: 0,
        frame: 0,
        hold_since: 0,
        pending: false,
        validated: false,
        capture: None,
        first_owner: None,
        expected: Vec::new(),
        tick: 0,
        epoch: 0,
        params: LeniaParams::default(),
        channel_pass: (0, 0),
        channel_tick: 0,
    })
    .add_systems(Last, flow::update);
}
