//! Native configure, compare, save/load and export qualification for authored rules.
use super::*;
mod exchange;
mod fixtures;
mod flow;
pub(super) fn install(app: &mut App) {
    let Some(output) = std::env::var_os("SOUL_RULES_VERIFY") else {
        return;
    };
    let output = std::path::PathBuf::from(output);
    std::fs::create_dir(&output).expect("fresh rule verification directory");
    app.insert_resource(Verification {
        output,
        phase: 0,
        frames: 0,
        wait: 0,
        captured: default(),
        expected: None,
    })
    .add_systems(Last, flow::update);
}
