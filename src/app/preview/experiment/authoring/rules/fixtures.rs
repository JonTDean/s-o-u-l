use models::workbench::{
    catalog::Kind,
    experiment::config::{
        Pattern, Settings,
        authored::{
            Rule,
            composition::{Blend, Layer, Program},
        },
        energy::Energy,
    },
    project::Project,
    session::Session,
};
pub(super) fn project() -> Project {
    Project {
        version: 2,
        selected: 0,
        sessions: (0..3).map(|_| Session::new(Kind::Life)).collect(),
        lenia: vec![],
    }
}
pub(super) fn settings(index: usize) -> Settings {
    let mut s = Settings::for_kind(Kind::Life);
    s.world.width = 32;
    s.world.height = 32;
    s.pattern = Pattern::Noise;
    s.program = Some(Program {
        primary: Rule::copy_west(),
        secondary: (index > 0).then(|| Layer {
            rule: Rule::life(8, 12),
            blend: Blend::Union,
        }),
    });
    if index == 2 {
        s.energy = Some(Energy::default());
    }
    s
}
