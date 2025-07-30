//! Full‑screen fade‑in / fade‑out transition for Bevy 0.16.
//!
//! Two `Update`‑phase systems are registered:
//!   1. [`spawn_overlay`] — creates the black overlay and seeds [`FadeData`].
//!   2. [`animate_overlay`] — drives the alpha, swaps `AppState` mid‑fade
//!      and cleans everything up afterwards.
//!
//! The systems are data‑parallel and need no external synchronisation.

use bevy::{
    prelude::*,
    ui::{BackgroundColor, GlobalZIndex, Node, Val},
    color::Alpha
};
use engine_core::prelude::AppState;

/* ─────────────────────────── Events ──────────────────────────── */

/// Request a transition to the given [`AppState`].
#[derive(Event, Debug, Clone, Copy)]
pub struct FadeTo {
    pub target: AppState,
}

/* ───────────────────────── Internals ─────────────────────────── */

#[derive(Component)] struct FadeOverlay;

#[derive(Resource)]
struct FadeData {
    timer:  Timer,
    phase:  Phase,
    target: Option<AppState>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Phase { Out, In }

/* ───────────────────────── Plugin ────────────────────────────── */

pub struct FadePlugin;
impl Plugin for FadePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FadeTo>()
           .add_systems(Update, (spawn_overlay, animate_overlay));
    }
}

/* ───────────── phase 0 – overlay creation ───────────────────── */

fn spawn_overlay(
    mut cmd:        Commands,
    mut requests:   EventReader<FadeTo>,
    overlay_q:      Query<Entity, With<FadeOverlay>>,
) {
    for FadeTo { target } in requests.read() {
        if overlay_q.is_empty() {
            // `Node` now holds all layout fields that used to sit in `Style`
            cmd.spawn((
                FadeOverlay,
                Node {
                    width:  Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.0)),
                GlobalZIndex(10_000),
            ));
        }

        cmd.insert_resource(FadeData {
            timer:  Timer::from_seconds(0.35, TimerMode::Repeating),
            phase:  Phase::Out,
            target: Some(*target),
        });
    }
}

/* ───────────── phase 1 & 2 – animate & cleanup ──────────────── */

fn animate_overlay(
    mut cmd:       Commands,
    time:          Res<Time>,
    fade:      Option<ResMut<FadeData>>,
    mut overlay_q: Query<(Entity, &mut BackgroundColor), With<FadeOverlay>>,
    mut next:      ResMut<NextState<AppState>>,
) {
    let Some(mut fade) = fade else { return };

    fade.timer.tick(time.delta());
    let t = fade.timer.elapsed_secs() / fade.timer.duration().as_secs_f32();

    for (_, mut bg) in &mut overlay_q {
        let alpha = match fade.phase {
            Phase::Out =>  t,
            Phase::In  => 1.0 - t,
        };
        bg.0.set_alpha(alpha);      // new helper name
    }

    if fade.timer.finished() {
        fade.timer.reset();
        match fade.phase {
            Phase::Out => {
                if let Some(target) = fade.target.take() {
                    next.set(target);          // swap state while black
                }
                fade.phase = Phase::In;
            }
            Phase::In => {
                for (e, _) in &overlay_q {     // `despawn()` is recursive now
                    cmd.entity(e).despawn();
                }
                cmd.remove_resource::<FadeData>();
            }
        }
    }
}
