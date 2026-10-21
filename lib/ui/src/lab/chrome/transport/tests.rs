use super::*;
use bevy_egui::egui::{Rect, pos2};

/// Run the bar row at `width` px; returns the bar, text shapes and rect shapes.
fn laid_out(width: f32) -> (Rect, Vec<(String, Rect)>, Vec<Rect>) {
    let ctx = egui::Context::default();
    crate::theme::visuals::install(&ctx);
    let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(width, 300.0));
    let input = egui::RawInput {
        screen_rect: Some(screen),
        ..Default::default()
    };
    let mut bar = Rect::NOTHING;
    let out = ctx.run(input, |ctx| {
        egui::TopBottomPanel::bottom("t")
            .exact_height(44.0)
            .frame(Frame::new().inner_margin(Margin::symmetric(MARGIN as i8, 0)))
            .show(ctx, |ui| {
                bar = ui.max_rect();
                ui.horizontal_centered(|ui| {
                    tail(ui, None, Some(10.0));
                });
            });
    });
    let mut texts = Vec::new();
    let mut rects = Vec::new();
    for s in &out.shapes {
        match &s.shape {
            egui::Shape::Text(t) => {
                texts.push((t.galley.text().to_string(), t.visual_bounding_rect()))
            }
            egui::Shape::Rect(r) => rects.push(r.visual_bounding_rect()),
            _ => {}
        }
    }
    (bar, texts, rects)
}

#[test]
fn rate_selector_stays_on_one_row_inside_the_bar() {
    for width in [700.0, 1400.0] {
        let (bar, texts, rects) = laid_out(width);
        let panel = Rect::from_min_max(pos2(0.0, bar.top()), pos2(width, bar.bottom()));
        assert_eq!(panel.height(), 44.0);
        let labels: Vec<_> = RATE_PRESETS.iter().map(|p| format!("{p:.0}")).collect();
        let rows: Vec<Rect> = labels
            .iter()
            .map(|l| texts.iter().find(|(t, _)| t == l).expect("label drawn").1)
            .collect();
        assert!(rows.windows(2).all(|w| w[0].min.y == w[1].min.y));
        for r in rows.iter().chain(rects.iter().filter(|r| r.is_positive())) {
            assert!(
                panel.contains_rect(*r),
                "{r:?} outside {panel:?} at {width}"
            );
        }
    }
}
