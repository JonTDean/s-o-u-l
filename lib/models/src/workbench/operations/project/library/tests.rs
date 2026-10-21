use super::*;
use crate::workbench::{catalog::Kind, plugin::Workbench};
#[test]
fn imported_and_duplicated_specimens_preserve_every_family() {
    let root = std::env::temp_dir().join(format!("soul-library-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    for kind in Kind::ALL {
        let mut bench = Workbench::default();
        bench.spawn(kind);
        bench.sessions[0].step();
        let project = Project::capture(&bench, Vec::new()).unwrap();
        let original = path(&root, &format!("{kind:?}")).unwrap();
        let duplicate = path(&root, &format!("{kind:?}-copy")).unwrap();
        create(&original, &project).unwrap();
        copy(&original, &duplicate).unwrap();
        assert_eq!(fs::read(&original).unwrap(), fs::read(&duplicate).unwrap());
        assert!(copy(&original, &duplicate).is_err());
        let mut restored = read(&duplicate).unwrap();
        bench.sessions[0].step();
        restored.sessions[0].step();
        assert_eq!(
            serde_json::to_value(&bench.sessions[0].model).unwrap(),
            serde_json::to_value(&restored.sessions[0].model).unwrap()
        );
    }
    assert_eq!(entries(&root).unwrap().len(), 24);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn names_and_invalid_imports_do_not_escape_or_create_entries() {
    let root = Path::new("/unused-library");
    for name in [
        "",
        "../escape",
        "/absolute",
        "a/b",
        "a\\b",
        ".hidden",
        " name",
        "CON",
    ] {
        assert!(path(root, name).is_err(), "{name}");
    }
    let mut bench = Workbench::default();
    let empty = Project::capture(&bench, Vec::new()).unwrap();
    assert!(validate(&empty).is_err());
    bench.spawn(Kind::Life);
    bench.sessions[0].paused = false;
    assert!(validate(&Project::capture(&bench, Vec::new()).unwrap()).is_err());
    bench.sessions[0].paused = true;
    bench.spawn(Kind::Rule30);
    assert!(validate(&Project::capture(&bench, Vec::new()).unwrap()).is_err());
}
#[test]
fn published_gpu_specimen_preserves_kernel_seed_and_exact_float_state() {
    let project = Project::capture(&Workbench::default(), vec![orbium::specimen()]).unwrap();
    validate(&project).unwrap();
    let restored: Project = serde_json::from_slice(&serde_json::to_vec(&project).unwrap()).unwrap();
    validate(&restored).unwrap();
    assert_eq!(
        restored.lenia[0].continuous.as_ref().unwrap().values,
        project.lenia[0].continuous.as_ref().unwrap().values
    );
    assert_eq!(restored.lenia[0].params, project.lenia[0].params);
    assert_eq!(
        restored.lenia[0].continuous.as_ref().unwrap().kernel,
        project.lenia[0].continuous.as_ref().unwrap().kernel
    );
}
