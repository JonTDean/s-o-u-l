//! Per-family experiment fields in the Marginalia vocabulary.
use crate::theme::{
    tokens::*,
    widgets::{dim, mono, plain, segmented, slider},
};
use bevy_egui::egui::{self, Sense, Ui};
use models::workbench::{
    catalog::Kind,
    experiment::config::{Pattern, Settings},
};

const PATTERNS: [(Pattern, &str); 5] = [
    (Pattern::Default, "default"),
    (Pattern::Single, "single"),
    (Pattern::Glider, "glider"),
    (Pattern::Blinker, "blinker"),
    (Pattern::Noise, "noise"),
];

/// Draw the fields that apply to `kind`.
pub(super) fn draw(ui: &mut Ui, kind: Kind, settings: &mut Settings) {
    slider(ui, "steps/s", &mut settings.rate_hz, 1.0..=60.0, false);
    if matches!(kind, Kind::Rule30 | Kind::Rule110 | Kind::Life | Kind::Hpp) {
        super::world::draw(ui, settings, kind);
        let before = settings.pattern;
        let valid: Vec<(Pattern, &str)> = PATTERNS
            .into_iter()
            .filter(|(pattern, _)| {
                let mut candidate = settings.clone();
                candidate.pattern = *pattern;
                candidate.validate(kind).is_ok()
            })
            .collect();
        segmented(ui, &mut settings.pattern, &valid);
        if before != settings.pattern {
            settings.world.seed = None;
        }
    }
    labelled(ui, "seed", |ui| {
        ui.add(egui::DragValue::new(&mut settings.random_seed))
    });
    if matches!(kind, Kind::Rule30 | Kind::Rule110) {
        labelled(ui, "rule", |ui| {
            ui.add(egui::DragValue::new(&mut settings.elementary).range(0..=255))
        });
    }
    if kind == Kind::Life {
        ui.add_enabled_ui(settings.program.is_none(), |ui| {
            counts(ui, "born", &mut settings.birth);
            counts(ui, "survive", &mut settings.survival);
        });
        if settings.program.is_some() {
            ui.label("Authored rules replace these birth/survival masks.");
        }
    }
    if matches!(kind, Kind::Boids | Kind::Ants) {
        labelled(ui, "agents", |ui| {
            ui.add(egui::DragValue::new(&mut settings.agent_count).range(1..=256))
        });
    }
    if kind == Kind::Boids {
        slider(ui, "cohesion", &mut settings.cohesion, 0.0..=1.0, false);
        slider(ui, "alignment", &mut settings.alignment, 0.0..=1.0, false);
        slider(ui, "separation", &mut settings.separation, 0.0..=1.0, false);
        let distance = &mut settings.separation_distance;
        slider(ui, "spacing", distance, 0.1..=32.0, true);
    }
    if kind == Kind::Ants {
        slider(ui, "evaporate", &mut settings.evaporation, 0.0..=1.0, false);
        slider(ui, "deposit", &mut settings.deposit, 0.0..=10.0, false);
        slider(ui, "explore", &mut settings.exploration, 0.0..=1.0, false);
    }
}

fn labelled(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui) -> egui::Response) {
    ui.horizontal(|ui| {
        ui.label(dim(label));
        add(ui);
    });
}

/// Neighbour counts 0–8 as toggleable mono digits (amber when set).
fn counts(ui: &mut Ui, label: &str, mask: &mut u16) {
    ui.horizontal(|ui| {
        ui.label(plain(label).color(INK_DIM));
        for count in 0..=8 {
            let on = *mask & (1 << count) != 0;
            let digit = mono(count.to_string()).color(if on { SIGNAL } else { INK_DIM });
            if ui
                .add(egui::Label::new(digit).sense(Sense::click()))
                .clicked()
            {
                *mask ^= 1 << count;
            }
        }
    });
}
