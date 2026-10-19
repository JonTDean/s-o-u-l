use crate::workbench::{catalog::Kind, plugin::Workbench, project::Project};
#[test]
fn malformed_projects_fail_before_mutation() {
    let mut bench = Workbench::default();
    bench.spawn(Kind::Life);
    let mut p = Project::capture(&bench, Vec::new()).unwrap();
    p.version = 2;
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
