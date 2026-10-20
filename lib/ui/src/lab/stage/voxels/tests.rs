use super::*;
use engine_core::automata::channel::ChannelId;
use models::workbench::catalog::Kind;
#[test]
fn selected_tape_creates_meshes_steps_and_cleans_up() {
    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<Workbench>()
        .init_resource::<Channels>()
        .init_resource::<StageRect>()
        .add_systems(Startup, setup)
        .add_systems(Update, sync);
    let mut bench = app.world_mut().resource_mut::<Workbench>();
    bench.spawn(Kind::Turing);
    bench.sessions[0].channel = ChannelId(7);
    app.world_mut().resource_mut::<Channels>().selected = Some(ChannelId(7));
    app.update();
    let count = |world: &mut World| {
        world
            .query_filtered::<Entity, With<SymbolVoxel>>()
            .iter(world)
            .count()
    };
    assert_eq!(
        count(app.world_mut()),
        6,
        "four symbols, trailing blank and head cap"
    );
    app.world_mut().resource_mut::<Workbench>().sessions[0].single_step();
    app.update();
    let bench = app.world().resource::<Workbench>();
    let Model::Symbolic(machine) = &bench.sessions[0].model else {
        panic!()
    };
    assert_eq!(layout::cells(machine)[0].symbol, 'X');
    assert!(layout::cells(machine)[1].head);
    app.world_mut().resource_mut::<Channels>().selected = None;
    app.update();
    assert_eq!(count(app.world_mut()), 0);
}
#[test]
fn stack_voxels_and_end_of_tape_heads_remain_visible() {
    let mut machine = Symbolic::new(Kind::Pushdown, "aaabbb");
    machine.step();
    assert!(
        layout::cells(&machine)
            .iter()
            .any(|c| c.position.y < 0.0 && c.symbol == 'A')
    );
    let mut machine = Symbolic::new(Kind::Turing, &"a".repeat(100));
    for _ in 0..100 {
        machine.step();
    }
    let cells = layout::cells(&machine);
    assert_eq!(cells.len(), 20);
    assert!(cells.iter().any(|c| c.head && c.symbol == ' '));
}
