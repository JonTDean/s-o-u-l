// lib/output/src/ui/fade.rs
//! Simple fade‑in / fade‑out transition layer.

use bevy::prelude::*;            // ←  **everything we need is here**
use engine_core::state::AppState;

/* ───────────── Events ───────────── */

#[derive(Event)]
pub struct FadeTo { pub target: AppState }

/* ──────────── Internals ─────────── */

#[derive(Component)]     struct FadeOverlay;

#[derive(Resource)]
struct FadeData {
    timer:  Timer,
    phase:  Phase,
    target: Option<AppState>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Phase { Out, In }

/* ─────────── Plugin ─────────────── */

pub struct FadePlugin;
impl Plugin for FadePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FadeTo>()
           .add_systems(Update, (begin_fade, animate_fade));
    }
}

/* ───── phase 0 – spawn & set‑up ───── */

fn begin_fade(
    mut cmd:      Commands,
    mut ev:       EventReader<FadeTo>,
    overlay_q:    Query<Entity, With<FadeOverlay>>,
) {
    for FadeTo { target } in ev.read() {
        if overlay_q.is_empty() {
            cmd.spawn((
                FadeOverlay,
                NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::BLACK.with_alpha(0.0)),
                    z_index: ZIndex(10_000),
                    ..default()
                },
            ));
        }

        cmd.insert_resource(FadeData {
            timer:  Timer::from_seconds(0.35, TimerMode::Repeating),
            phase:  Phase::Out,
            target: Some(*target),
        });
    }
}

/* ───── phase 1 / 2 – animate ───── */

fn animate_fade(
    mut cmd:     Commands,
    time:        Res<Time>,
    mut fade:    Option<ResMut<FadeData>>,
    mut overlay: Query<(Entity, &mut BackgroundColor), With<FadeOverlay>>,
    mut next:    ResMut<NextState<AppState>>,
) {
    let Some(mut fade) = fade else { return };

    fade.timer.tick(time.delta());
    let t = fade.timer.elapsed_secs() / fade.timer.duration().as_secs_f32();

    /* update alpha --------------------------------------------------- */
    for (_, mut bg) in &mut overlay {
        let alpha = match fade.phase {
            Phase::Out =>  t,
            Phase::In  => 1.0 - t,
        };
        bg.0 = Color::BLACK.with_alpha(alpha);
    }

    /* phase transitions ---------------------------------------------- */
    if fade.timer.finished() {
        fade.timer.reset();

        match fade.phase {
            Phase::Out => {
                if let Some(target) = fade.target.take() {
                    next.set(target);              // state swap at mid‑fade
                }
                fade.phase = Phase::In;
            }
            Phase::In => {
                for (e, _) in &overlay {
                    cmd.entity(e).despawn();        // recursive by default
                }
                cmd.remove_resource::<FadeData>();
            }
        }
    }
}
