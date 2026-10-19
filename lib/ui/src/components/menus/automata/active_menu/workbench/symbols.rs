//! Machine state, consumed input, stacks and acceptance remain visible together.
use bevy_egui::egui;
use models::workbench::symbolic::machine::Symbolic;

pub(super) fn draw(ui: &mut egui::Ui, machine: &Symbolic) {
    ui.separator();
    let status = match machine.result() {
        Some(true) => "Accepted",
        Some(false) => "Rejected",
        None if machine.halted() => "Expansion limit reached (4096 symbols)",
        _ => "In progress",
    };
    ui.heading(status);
    match machine {
        Symbolic::Finite(m) => {
            let mut states: Vec<_> = m.states.iter().copied().collect();
            states.sort_unstable();
            ui.label(format!("Active states: {states:?} · accepting state: 1"));
            tape(ui, &m.input, m.cursor);
        }
        Symbolic::Pushdown(m) => {
            ui.label(format!(
                "Stack (bottom -> top): {}",
                m.stack.iter().collect::<String>()
            ));
            tape(ui, &m.input, m.cursor);
        }
        Symbolic::Bounded(m) => {
            ui.label("Tape stays within the original input; matched triples become A, B, C.");
            tape(ui, &m.tape, m.head);
        }
        Symbolic::Turing(m) => {
            ui.label(format!(
                "State {} · accept {} · reject {}",
                m.state, m.accept_state, m.reject_state
            ));
            tape(ui, &m.tape, m.head);
        }
        Symbolic::Koch { word, .. } => {
            super::koch::draw(ui, word);
            ui.label(format!("{} symbols · F -> F+F--F+F", word.chars().count()));
            egui::ScrollArea::vertical()
                .max_height(120.0)
                .show(ui, |ui| {
                    ui.monospace(word);
                });
        }
    }
}
fn tape(ui: &mut egui::Ui, symbols: &[char], cursor: usize) {
    ui.label(format!("Head / input position: {cursor}"));
    egui::ScrollArea::horizontal().show(ui, |ui| {
        ui.horizontal(|ui| {
            for (index, symbol) in symbols.iter().chain(std::iter::once(&' ')).enumerate() {
                let text = if *symbol == ' ' {
                    "□".into()
                } else {
                    symbol.to_string()
                };
                let text = egui::RichText::new(text).monospace().size(20.0);
                ui.label(if index == cursor {
                    text.background_color(egui::Color32::DARK_BLUE)
                } else {
                    text
                });
            }
        });
    });
}
