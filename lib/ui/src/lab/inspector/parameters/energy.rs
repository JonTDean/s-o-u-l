//! Two-pool parameters are reset-only; current totals appear in the readout.
use bevy_egui::egui::{self, Ui};
use models::workbench::experiment::config::{Settings, energy::Energy};
pub(super) fn draw(ui: &mut Ui, settings: &mut Settings) {
    egui::CollapsingHeader::new("Two-pool energy")
        .id_salt(("energy", settings.energy.is_some()))
        .default_open(settings.energy.is_some())
        .show(ui, |ui| {
            let mut enabled = settings.energy.is_some();
            if ui
                .checkbox(&mut enabled, "Enable energy-gated births and survival")
                .changed()
            {
                settings.energy = enabled.then(Energy::default);
            }
            let Some(energy) = &mut settings.energy else {
                return;
            };
            ui.label("Editable starting model. Changes require Apply and reset.");
            ui.label(
                "Diffuse → decay → recharge → birth/survival cost. Both pools must cover the cost.",
            );
            for pool in 0..2 {
                ui.push_id(pool, |ui| {
                    ui.label(if pool == 0 { "Pool A" } else { "Pool B" });
                    field(ui, "capacity", &mut energy.capacity[pool], 1..=4096);
                    field(
                        ui,
                        "diffusion / 1000",
                        &mut energy.diffusion[pool],
                        0..=1000,
                    );
                    let cap = energy.capacity[pool];
                    for (label, values) in [
                        ("initial", &mut energy.initial),
                        ("decay / tick", &mut energy.decay),
                        ("recharge / tick", &mut energy.recharge),
                        ("birth cost", &mut energy.birth_cost),
                        ("survival cost", &mut energy.survival_cost),
                    ] {
                        field(ui, label, &mut values[pool], 0..=cap);
                    }
                });
            }
        });
}
fn field(ui: &mut Ui, label: &str, value: &mut u16, range: std::ops::RangeInclusive<u16>) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).range(range));
    });
}
