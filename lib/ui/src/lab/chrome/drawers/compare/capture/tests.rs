use super::super::Comparison;
use super::*;
use bevy::prelude::*;
use engine_core::{automata::channel::ChannelSet, events::AutomataCommand};
use models::workbench::{
    catalog::Kind,
    plugin::{Workbench, WorkbenchPlugin},
};
fn app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_event::<AutomataCommand>()
        .add_plugins(WorkbenchPlugin)
        .init_resource::<Comparison>();
    #[cfg(feature = "gpu-compute")]
    app.init_resource::<engine_gpu::LeniaProbe>();
    app.add_systems(Update, poll.after(ChannelSet::Settle));
    app.world_mut()
        .resource_mut::<Workbench>()
        .spawn(Kind::Life);
    app.world_mut()
        .resource_mut::<Workbench>()
        .spawn(Kind::Life);
    app.update();
    let ids: Vec<_> = app
        .world()
        .resource::<Workbench>()
        .sessions
        .iter()
        .map(|s| s.channel)
        .collect();
    let mut c = app.world_mut().resource_mut::<Comparison>();
    c.left = Some(ids[0]);
    c.right = Some(ids[1]);
    app
}
#[test]
fn step_waits_for_completed_states_and_compares_exact_ticks() {
    let mut app = app();
    app.world_mut().resource_mut::<Comparison>().capture.request = Some(true);
    app.update();
    assert!(
        app.world()
            .resource::<Comparison>()
            .capture
            .result
            .is_none()
    );
    app.update();
    let c = app.world().resource::<Comparison>();
    let pair = c.capture.result.as_ref().unwrap();
    assert_eq!(pair[0].stamp.tick, 1);
    assert_eq!(pair[1].stamp.tick, 1);
    assert_eq!(pair[0].field.compare(&pair[1].field).unwrap().maximum, 0.0);
}
#[test]
fn reset_and_closed_channels_cancel_a_pending_pair() {
    let mut app = app();
    app.world_mut().resource_mut::<Comparison>().capture.request = Some(true);
    app.update();
    app.world_mut().resource_mut::<Workbench>().sessions[0]
        .reset()
        .unwrap();
    app.update();
    let c = app.world().resource::<Comparison>();
    assert!(c.capture.result.is_none());
    assert!(c.status.contains("changed"));
    app.world_mut().resource_mut::<Comparison>().capture.request = Some(true);
    app.update();
    app.world_mut()
        .resource_mut::<Workbench>()
        .sessions
        .remove(0);
    app.update();
    assert!(
        app.world()
            .resource::<Comparison>()
            .status
            .contains("closed")
    );
}
