//! CPU family sessions alongside the GPU Lenia workbench.
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use models::workbench::{plugin::Workbench, session::Model};

mod canvas;
mod koch;
mod symbols;

pub fn show(mut contexts: EguiContexts, mut bench: ResMut<Workbench>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if bench.sessions.is_empty() {
        return;
    }
    egui::Window::new("Automata workbench")
        .default_pos([320.0, 80.0])
        .default_width(560.0)
        .max_height(570.0)
        .resizable(true)
        .show(ctx, |ui| {
            let mut selected = bench.selected.min(bench.sessions.len() - 1);
            egui::ComboBox::from_id_salt("session")
                .selected_text(format!(
                    "{} · {}",
                    selected + 1,
                    bench.sessions[selected].kind.title()
                ))
                .show_ui(ui, |ui| {
                    for (index, session) in bench.sessions.iter().enumerate() {
                        ui.selectable_value(
                            &mut selected,
                            index,
                            format!("{} · {}", index + 1, session.kind.title()),
                        );
                    }
                });
            bench.selected = selected;
            if ui.button("Remove selected session").clicked() {
                bench.sessions.remove(selected);
                bench.selected = selected.saturating_sub(1);
                return;
            }
            let session = &mut bench.sessions[selected];
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !session.halted(),
                        egui::Button::new(if session.paused { "Resume" } else { "Pause" }),
                    )
                    .clicked()
                {
                    session.paused = !session.paused;
                }
                if ui
                    .add_enabled(
                        session.paused && !session.halted(),
                        egui::Button::new("Single step"),
                    )
                    .clicked()
                {
                    session.single_step();
                }
                if ui.button("Reset").clicked() {
                    let _ = session.reset();
                }
                ui.label(format!("Tick {}", session.ticks));
            });
            if matches!(session.model, Model::Symbolic(_)) {
                ui.label("Initial input (apply with Reset):");
                ui.add(egui::TextEdit::singleline(&mut session.input).char_limit(256));
            }
            ui.label(if session.halted() {
                "Halted"
            } else if session.paused {
                "Paused"
            } else {
                "Running · 10 steps/s"
            });
            if let Model::Symbolic(machine) = &session.model {
                symbols::draw(ui, machine);
            } else {
                canvas::draw(ui, &session.model);
            }
        });
}
