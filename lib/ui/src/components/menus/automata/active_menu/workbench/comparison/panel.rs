use super::*;
use bevy_egui::{EguiContexts, egui};
use models::workbench::{experiment::metrics, plugin::Workbench, session::Model};
pub(super) fn show(
    mut contexts: EguiContexts,
    mut state: ResMut<Comparison>,
    mut bench: ResMut<Workbench>,
    files: Res<super::super::persistence::Files>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    egui::Window::new("Compare and export")
        .default_pos([910.0, 80.0])
        .default_width(700.0)
        .default_open(false)
        .vscroll(true)
        .max_height(800.0)
        .show(ctx, |ui| {
            if !bench.sessions.is_empty() {
                let last = bench.sessions.len() - 1;
                state.left = state.left.min(last);
                state.right = state.right.min(last);
                ui.horizontal(|ui| {
                    let state = &mut *state;
                    for (id, selection) in [("Left", &mut state.left), ("Right", &mut state.right)]
                    {
                        egui::ComboBox::from_id_salt(id)
                            .selected_text(format!("{id}: {}", *selection + 1))
                            .show_ui(ui, |ui| {
                                for (index, session) in bench.sessions.iter().enumerate() {
                                    ui.selectable_value(
                                        selection,
                                        index,
                                        format!("{} · {}", index + 1, session.kind.title()),
                                    );
                                }
                            });
                    }
                });
                if ui.button("Pause and step both").clicked() {
                    state.status = metrics::step_pair(&mut bench.sessions, state.left, state.right)
                        .map_or_else(str::to_owned, |()| {
                            "Both sessions advanced one transition".into()
                        });
                }
                ui.columns(2, |columns| {
                    for (column, index) in columns.iter_mut().zip([state.left, state.right]) {
                        let s = &bench.sessions[index];
                        column.label(s.kind.title());
                        column.label(format!(
                            "Tick {} · {}",
                            s.ticks,
                            if s.paused { "paused" } else { "running" }
                        ));
                        column.label(metrics::describe(s));
                        match &s.model {
                            Model::Symbolic(m) => super::super::symbols::draw(column, m),
                            model => super::super::canvas::draw(column, model),
                        }
                    }
                });
            }
            super::exports::show(ui, &mut state, &bench, &files.path);
        });
}
