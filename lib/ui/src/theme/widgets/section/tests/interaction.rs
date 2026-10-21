use super::*;
use crate::theme::tokens::{INK_DIM, SIGNAL};
use bevy_egui::egui::Color32;

/// Colour a text shape really renders in: its glyphs must be uncoloured
/// (PLACEHOLDER), otherwise `fallback_color` is ignored by epaint 0.31.1.
fn rendered(out: &bevy_egui::egui::FullOutput, text: &str) -> Color32 {
    out.shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Text(t) if t.galley.text() == text => {
                let baked = t
                    .galley
                    .rows
                    .iter()
                    .flat_map(|r| r.visuals.mesh.vertices.iter());
                assert!(baked.map(|v| v.color).all(|c| c == Color32::PLACEHOLDER));
                Some(t.fallback_color)
            }
            _ => None,
        })
        .expect("text painted")
}

#[test]
fn hovered_action_renders_in_signal() {
    let ctx = ctx();
    let run = |events: Vec<Event>| {
        pass(&ctx, events, |ui| {
            section("p").action("reset").show(ui, |_| {});
        })
    };
    let out = run(vec![]);
    assert_eq!(rendered(&out, "reset"), INK_DIM);
    let at = out
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Text(t) if t.galley.text() == "reset" => {
                Some(t.galley.rect.translate(t.pos.to_vec2()).center())
            }
            _ => None,
        })
        .expect("action text painted");
    run(vec![Event::PointerMoved(at)]);
    let mut out = run(vec![]);
    for _ in 0..30 {
        out = run(vec![]);
    }
    assert_eq!(rendered(&out, "reset"), SIGNAL);
}

#[test]
fn header_click_toggles_fold_and_persists() {
    let ctx = ctx();
    let run = |events: Vec<Event>| {
        let mut open = false;
        let out = pass(&ctx, events, |ui| {
            open = section("grid").folds(false).show(ui, |_| {}).open;
        });
        (open, out)
    };
    let (open, out) = run(vec![]);
    assert!(!open);
    let at = out
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Text(t) if t.galley.text() == "GRID" => {
                Some(t.galley.rect.translate(t.pos.to_vec2()).center())
            }
            _ => None,
        })
        .expect("title painted");
    let button = |pressed| Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    run(vec![Event::PointerMoved(at)]);
    run(vec![button(true)]);
    assert!(run(vec![button(false)]).0, "click opens the folded section");
    assert!(run(vec![]).0, "open state persists");
    run(vec![button(true)]);
    run(vec![button(false)]);
    assert!(!run(vec![]).0, "second click folds again");
    assert!(!run(vec![]).0, "folded state persists");
}

#[test]
fn warnings_stay_visible_while_folded_and_follow_the_divider() {
    let ctx = ctx();
    let out = pass(&ctx, vec![], |ui| {
        section("first").show(ui, |_| {});
        section("about")
            .folds(false)
            .warnings(["careful".to_string()])
            .show(ui, |ui| {
                ui.label("hidden body");
            });
    });
    let all = texts(&out);
    assert!(all.iter().any(|t| t == "careful"));
    assert!(!all.iter().any(|t| t == "hidden body"));
    let y = |name: &str| {
        out.shapes.iter().find_map(|s| match &s.shape {
            Shape::Text(t) if t.galley.text() == name => Some(t.pos.y),
            _ => None,
        })
    };
    assert!(y("careful") > y("ABOUT"), "warning sits below the header");
}
