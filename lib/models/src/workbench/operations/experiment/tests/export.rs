use crate::workbench::{
    catalog::Kind,
    experiment::{export::write_bundle, metrics::step_pair},
    plugin::Workbench,
    project::Project,
};
#[test]
fn export_decodes_and_preserves_live_state() {
    let mut bench = Workbench::default();
    bench.spawn(Kind::Life);
    bench.spawn(Kind::Rule110);
    step_pair(&mut bench.sessions, 0, 1).unwrap();
    assert_eq!(bench.sessions[0].ticks, 1);
    assert_eq!(bench.sessions[1].ticks, 1);
    assert!(step_pair(&mut bench.sessions, 0, 0).is_err());
    let project = Project::capture(&bench, Vec::new()).unwrap();
    let before = serde_json::to_value(&project).unwrap();
    let directory = std::env::temp_dir().join(format!("soul-export-test-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    write_bundle(&project, &directory, 4, 100).unwrap();
    assert_eq!(serde_json::to_value(&project).unwrap(), before);
    let png = image::open(directory.join("snapshot.png")).unwrap();
    assert_eq!((png.width(), png.height()), (512, 256));
    let file =
        std::io::BufReader::new(std::fs::File::open(directory.join("animation.gif")).unwrap());
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = options.read_info(file).unwrap();
    let mut frames = Vec::new();
    while let Some(frame) = decoder.read_next_frame().unwrap() {
        frames.push(frame.clone());
    }
    assert_eq!(frames.len(), 4);
    assert_eq!((frames[0].width, frames[0].height), (512, 256));
    assert_ne!(frames[0].buffer, frames[1].buffer);
    assert_eq!(frames[0].delay, 10);
    let saved = crate::workbench::project::read(&directory.join("project.json")).unwrap();
    assert_eq!(saved.sessions[0].ticks, 1);
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["final_cpu_ticks"], serde_json::json!([4, 4]));
    assert!(write_bundle(&project, &directory, 4, 100).is_err());
    std::fs::remove_dir_all(directory).unwrap();
}
