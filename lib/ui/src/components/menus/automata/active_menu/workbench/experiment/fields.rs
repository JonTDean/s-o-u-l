use bevy_egui::egui;
use models::workbench::{
    catalog::Kind,
    experiment::config::{Pattern, Settings},
};

pub(super) fn draw(ui: &mut egui::Ui, kind: Kind, settings: &mut Settings) {
    ui.add(egui::Slider::new(&mut settings.rate_hz, 1.0..=60.0).text("Steps per second"));
    if matches!(kind, Kind::Rule30 | Kind::Rule110 | Kind::Life | Kind::Hpp) {
        egui::ComboBox::from_id_salt("seed-pattern")
            .selected_text(format!("{:?}", settings.pattern))
            .show_ui(ui, |ui| {
                for pattern in [
                    Pattern::Default,
                    Pattern::Single,
                    Pattern::Glider,
                    Pattern::Blinker,
                    Pattern::Noise,
                ] {
                    let mut candidate = settings.clone();
                    candidate.pattern = pattern;
                    if candidate.validate(kind).is_ok() {
                        ui.selectable_value(&mut settings.pattern, pattern, format!("{pattern:?}"));
                    }
                }
            });
    }
    ui.horizontal(|ui| {
        ui.label("Random seed");
        ui.add(egui::DragValue::new(&mut settings.random_seed));
    });
    if matches!(kind, Kind::Rule30 | Kind::Rule110) {
        ui.add(egui::Slider::new(&mut settings.elementary, 0..=255).text("Elementary rule"));
    }
    if kind == Kind::Life {
        for (name, mask) in [
            ("Birth", &mut settings.birth),
            ("Survive", &mut settings.survival),
        ] {
            ui.horizontal(|ui| {
                ui.label(name);
                for count in 0..=8 {
                    let mut on = *mask & (1 << count) != 0;
                    if ui.checkbox(&mut on, count.to_string()).changed() {
                        *mask ^= 1 << count;
                    }
                }
            });
        }
    }
    if matches!(kind, Kind::Boids | Kind::Ants) {
        ui.add(egui::Slider::new(&mut settings.agent_count, 1..=256).text("Population (reset)"));
    }
    if kind == Kind::Boids {
        for (name, value) in [
            ("Cohesion", &mut settings.cohesion),
            ("Alignment", &mut settings.alignment),
            ("Separation", &mut settings.separation),
        ] {
            ui.add(egui::Slider::new(value, 0.0..=1.0).text(name));
        }
        ui.add(
            egui::Slider::new(&mut settings.separation_distance, 0.1..=32.0)
                .text("Separation distance"),
        );
    }
    if kind == Kind::Ants {
        ui.add(egui::Slider::new(&mut settings.evaporation, 0.0..=1.0).text("Evaporation"));
        ui.add(egui::Slider::new(&mut settings.deposit, 0.0..=10.0).text("Trail deposit"));
        ui.add(egui::Slider::new(&mut settings.exploration, 0.0..=1.0).text("Exploration"));
    }
}
