use super::*;
use bevy_egui::egui::{self, Event, Modifiers, PointerButton, Rect, Sense, pos2, vec2};
#[test]
fn pointer_strokes_generate_one_undo_group_and_fill_drag_gaps() {
    let ctx = egui::Context::default();
    let session = Session::new(Kind::Life);
    let mut editor = Editor {
        tool: Tool::Paint,
        ..Default::default()
    };
    let fit = Fit {
        origin: pos2(0.0, 0.0),
        scale: 10.0,
        content: vec2(64.0, 64.0),
    };
    let mut frame = |events| {
        let mut fx = Effects::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 640.0))),
            events,
            ..Default::default()
        };
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = ui.allocate_rect(
                    Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 640.0)),
                    Sense::click_and_drag(),
                );
                editor.gesture(ui, &response, &fit, &session, &mut fx, true);
            });
        });
        fx.commands
    };
    let pointer = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(vec![]);
    let first = frame(vec![
        Event::PointerMoved(pos2(25.0, 25.0)),
        pointer(pos2(25.0, 25.0), true),
    ]);
    assert!(matches!(
        &first[0].action,
        ChannelAction::Paint { start: true, .. }
    ));
    let drag = frame(vec![Event::PointerMoved(pos2(65.0, 25.0))]);
    let ChannelAction::Paint {
        points,
        start,
        value,
    } = &drag[0].action
    else {
        panic!()
    };
    assert!(!start);
    assert_eq!(*value, 255);
    assert_eq!(points, &vec![[2, 2], [3, 2], [4, 2], [5, 2], [6, 2]]);
    assert!(frame(vec![pointer(pos2(65.0, 25.0), false)]).is_empty());
    let next = frame(vec![pointer(pos2(65.0, 25.0), true)]);
    assert!(matches!(
        &next[0].action,
        ChannelAction::Paint { start: true, .. }
    ));
}
