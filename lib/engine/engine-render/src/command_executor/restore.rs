//! Reconstitute GPU owners through the normal registry and allocation events.
use super::*;
use engine_core::automata::lenia::{LeniaSimulation, SavedLenia};
pub(super) fn restore(
    saved: &SavedLenia,
    registry: &mut AutomataRegistry,
    rules: &RuleRegistry,
    commands: &mut Commands,
    added: &mut EventWriter<AutomatonAdded>,
) {
    if saved.validate().is_err() {
        return;
    }
    let Some((rule, _)) = rules.get("lenia") else {
        return;
    };
    let id = registry.register(AutomatonInfo {
        id: AutomatonId(0),
        name: "lenia".into(),
        rule: Arc::clone(rule),
        params: Value::Null,
        slice: GpuGridSlice {
            layer: 0,
            offset: UVec2::ZERO,
            size: UVec2::new(saved.size[0], saved.size[1]),
            depth: saved.size[2],
            rule: "lenia".into(),
            rule_bits: 0,
        },
        voxel_size: DEFAULT_VOXEL,
        world_offset: Vec3::ZERO,
        background_color: BG,
        palette: None,
    });
    let entity = commands
        .spawn((
            id,
            LeniaSimulation {
                restored: Some(Arc::new(saved.clone())),
                params: saved.params,
                seed: saved.seed,
                paused: saved.paused,
                target_tick: saved.tick,
                epoch: 0,
            },
        ))
        .id();
    added.write(AutomatonAdded { id, entity });
}
