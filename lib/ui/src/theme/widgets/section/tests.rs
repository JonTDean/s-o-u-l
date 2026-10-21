use super::*;
use crate::theme::tokens::{INK_DIM, LINE, SIGNAL};
use bevy_egui::egui::{
    CentralPanel, Color32, Context, Event, PointerButton, Pos2, RawInput, Rect, Shape,
    collapsing_header::CollapsingState,
};

fn ctx() -> Context {
    let ctx = Context::default();
    crate::theme::visuals::install(&ctx);
    ctx
}

fn pass(
    ctx: &Context,
    events: Vec<Event>,
    ui_fn: impl FnOnce(&mut Ui),
) -> bevy_egui::egui::FullOutput {
    let input = RawInput {
        events,
        ..Default::default()
    };
    let mut ui_fn = Some(ui_fn);
    ctx.run(input, |ctx| {
        CentralPanel::default().show(ctx, |ui| ui_fn.take().expect("one pass")(ui));
    })
}

fn texts(out: &bevy_egui::egui::FullOutput) -> Vec<String> {
    out.shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Text(t) => Some(t.galley.text().to_string()),
            _ => None,
        })
        .collect()
}

#[test]
fn first_section_draws_no_divider() {
    let ctx = ctx();
    let out = pass(&ctx, vec![], |ui| {
        section("one").show(ui, |ui| {
            ui.label("a");
        });
        section("two").show(ui, |ui| {
            ui.label("b");
        });
    });
    let lines = out
        .shapes
        .iter()
        .filter(|s| matches!(&s.shape, Shape::LineSegment { stroke, .. } if stroke.color == LINE))
        .count();
    assert_eq!(lines, 1);
}

#[test]
fn fold_state_survives_frames() {
    let ctx = ctx();
    let show = |ui: &mut Ui| section("grid").foldable(false).show(ui, |_| {}).open;
    let (mut a, mut b, mut c, mut d) = (true, true, true, false);
    pass(&ctx, vec![], |ui| a = show(ui));
    pass(&ctx, vec![], |ui| {
        let id = ui.make_persistent_id(("section", "grid"));
        let mut state = CollapsingState::load_with_default_open(ui.ctx(), id, false);
        state.set_open(true);
        state.store(ui.ctx());
        b = show(ui);
    });
    pass(&ctx, vec![], |ui| c = show(ui));
    pass(&ctx, vec![], |ui| {
        section("other").foldable(false).show(ui, |_| {});
    });
    pass(&ctx, vec![], |ui| d = show(ui));
    assert!(!a && c && d, "a={a} b={b} c={c} d={d}");
}

#[test]
fn summary_only_while_folded() {
    let ctx = ctx();
    let text = "64×64×8 · GPU";
    let folded = pass(&ctx, vec![], |ui| {
        section("g").foldable(false).summary(text).show(ui, |_| {});
    });
    assert!(texts(&folded).iter().any(|t| t == text));
    let open = pass(&ctx, vec![], |ui| {
        section("h").foldable(true).summary(text).show(ui, |_| {});
    });
    assert!(!texts(&open).iter().any(|t| t == text));
}

#[test]
fn action_reports_click() {
    let ctx = ctx();
    let run = |events: Vec<Event>| {
        let mut clicked = false;
        let out = pass(&ctx, events, |ui| {
            clicked = section("p").action("reset").show(ui, |_| {}).action_clicked;
        });
        (clicked, out)
    };
    let (_, out) = run(vec![]);
    let rect: Rect = out
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Text(t) if t.galley.text() == "reset" => {
                Some(t.galley.rect.translate(t.pos.to_vec2()))
            }
            _ => None,
        })
        .expect("action text painted");
    let at: Pos2 = rect.center();
    let press = |pressed| Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    run(vec![Event::PointerMoved(at)]);
    run(vec![press(true)]);
    assert!(run(vec![press(false)]).0);
}

#[test]
fn long_summary_elides_instead_of_touching_title() {
    let ctx = ctx();
    let text = "64 cells · 64-row history · CPU · and a very long tail of detail";
    let draw = |ui: &mut Ui| {
        ui.allocate_ui(bevy_egui::egui::vec2(180.0, 40.0), |ui| {
            section("g").foldable(false).summary(text).show(ui, |_| {});
        });
    };
    pass(&ctx, vec![], draw);
    let out = pass(&ctx, vec![], draw);
    assert!(!texts(&out).iter().any(|t| t == text));
    assert!(texts(&out).iter().any(|t| t.ends_with('…')));
}

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
