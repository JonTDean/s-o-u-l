//! Publish a complete export directory only after every artifact is written.
use super::render;
use crate::workbench::project::{self, Project};
use gif::{Encoder, Frame, Repeat};
use std::{fs, path::Path};
pub fn write_bundle(
    project: &Project,
    destination: &Path,
    frames: u32,
    milliseconds: u32,
) -> Result<(), String> {
    project.validate()?;
    if !(1..=120).contains(&frames)
        || !(10..=1000).contains(&milliseconds)
        || !milliseconds.is_multiple_of(10)
    {
        return Err(
            "Export requires 1..120 frames and 10..1000 ms/frame in multiples of 10".into(),
        );
    }
    if destination.exists() {
        return Err("Export folder already exists; choose a new folder".into());
    }
    let temporary = destination.with_extension(format!("export-tmp-{}", std::process::id()));
    fs::create_dir(&temporary).map_err(|e| e.to_string())?;
    let result = generate(project, &temporary, frames, milliseconds)
        .and_then(|()| fs::rename(&temporary, destination).map_err(|e| e.to_string()));
    if result.is_err() {
        let _ = fs::remove_dir_all(&temporary);
    }
    result
}
fn generate(
    project: &Project,
    directory: &Path,
    frames: u32,
    milliseconds: u32,
) -> Result<(), String> {
    let mut copy: Project =
        serde_json::from_value(serde_json::to_value(project).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    project::write(&directory.join("project.json"), project)?;
    render(project)
        .save(directory.join("snapshot.png"))
        .map_err(|e| e.to_string())?;
    let file = fs::File::create(directory.join("animation.gif")).map_err(|e| e.to_string())?;
    let width = render(project).width() as u16;
    let mut encoder = Encoder::new(file, width, 256, &[]).map_err(|e| e.to_string())?;
    encoder
        .set_repeat(Repeat::Infinite)
        .map_err(|e| e.to_string())?;
    for frame in 0..frames {
        let mut pixels = render(&copy).into_raw();
        let mut encoded = Frame::from_rgba_speed(width, 256, &mut pixels, 10);
        encoded.delay = (milliseconds / 10) as u16;
        encoder.write_frame(&encoded).map_err(|e| e.to_string())?;
        if frame + 1 < frames {
            for session in &mut copy.sessions {
                session.step();
            }
            for saved in &mut copy.lenia {
                saved.densities = simulation_kernel::core::lenia::step(
                    &saved.densities,
                    saved.size,
                    saved.params,
                )?;
                saved.tick += 1;
            }
        }
    }
    let manifest = serde_json::json!({
        "version":1, "frames":frames, "milliseconds_per_frame":milliseconds,
        "layout":"256px panels: CPU session order followed by Lenia maximum XY projections",
        "sampling":"initial state then one transition per frame; halted symbolic sessions remain fixed",
        "lenia_evolution":"quantized CPU reference from completed GPU checkpoint",
        "symbolic_view":"first 256 tape cells, head underline; full state in project.json",
        "initial_cpu_ticks":project.sessions.iter().map(|s|s.ticks).collect::<Vec<_>>(),
        "final_cpu_ticks":copy.sessions.iter().map(|s|s.ticks).collect::<Vec<_>>(),
        "initial_lenia_ticks":project.lenia.iter().map(|s|s.tick).collect::<Vec<_>>(),
        "final_lenia_ticks":copy.lenia.iter().map(|s|s.tick).collect::<Vec<_>>()
    });
    fs::write(
        directory.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
