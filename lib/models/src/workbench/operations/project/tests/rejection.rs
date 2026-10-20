use crate::workbench::{catalog::Kind, plugin::Workbench, project::Project};
#[test]
fn malformed_projects_fail_before_mutation() {
    let mut bench = Workbench::default();
    bench.spawn(Kind::Life);
    let mut p = Project::capture(&bench, Vec::new()).unwrap();
    p.version = 99;
    assert!(p.validate().is_err());
    p.version = 1;
    p.sessions[0].kind = Kind::Ants;
    assert!(p.validate().is_err());
    assert_eq!(bench.sessions[0].kind, Kind::Life);
    p.sessions[0].kind = Kind::Life;
    if let crate::workbench::session::Model::Grid(g) = &mut p.sessions[0].model {
        g.grid.voxels.pop();
    }
    assert!(p.validate().is_err());
}

#[test]
fn file_roundtrip_and_unknown_fields() {
    let mut bench = Workbench::default();
    bench.spawn(Kind::Ants);
    let p = Project::capture(&bench, Vec::new()).unwrap();
    let path = std::env::temp_dir().join(format!("soul-project-test-{}.json", std::process::id()));
    super::super::write(&path, &p).unwrap();
    let loaded = super::super::read(&path).unwrap();
    assert_eq!(loaded.sessions.len(), 1);
    let mut value = serde_json::to_value(&p).unwrap();
    value["sessions"][0]["unknown"] = serde_json::json!(true);
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(super::super::read(&path).is_err());
    std::fs::remove_file(path).unwrap();
}
