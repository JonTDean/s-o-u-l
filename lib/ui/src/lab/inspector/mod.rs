//! Right-hand margin inspector: readout, parameters, machine, input, colour, arena, footnote.
use super::{Effects, LabData, state::LabState};
use crate::theme::{tokens::*, widgets::*};
use bevy_egui::egui::{self, Frame, Margin, Sense, Ui};
use engine_core::automata::channel::{
    Backend, ChannelAction, ChannelCommand, ChannelId, ChannelRow, SWATCHES, Srgb,
};
use models::workbench::{
    experiment::metrics::describe,
    session::{Model, Session},
    symbolic::machine::Symbolic,
};
mod lenia;
pub mod machine;
pub mod parameters;
pub mod sections;

/// Draw the inspector panel unless collapsed (N).
pub fn show(ctx: &egui::Context, lab: &mut LabState, data: &LabData, fx: &mut Effects) {
    if !lab.inspector_open {
        return;
    }
    let margin = Margin {
        left: 2,
        right: 12,
        top: 14,
        bottom: 14,
    };
    let panel = egui::SidePanel::right("lab.inspector")
        .resizable(true)
        .default_width(lab.inspector_width)
        .width_range(220.0..=420.0)
        .frame(Frame::new().fill(PANEL).inner_margin(margin))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| body(ui, lab, data, fx));
        });
    lab.inspector_width = LabState::clamp_width(panel.response.rect.width());
}

fn body(ui: &mut Ui, lab: &mut LabState, data: &LabData, fx: &mut Effects) {
    let Some(row) = data.channels.selected_row() else {
        note(ui, "No specimen selected.", false);
        return;
    };
    let id = row.id;
    let metric = data.metrics.get(id);
    let session = data.bench.sessions.iter().find(|s| s.channel == id);
    margin_section(ui, "readout", true, |ui| {
        readout(ui, &tick_digits(row.tick), "tick", true);
        if let Some((m, sample)) = metric.and_then(|m| m.latest.map(|s| (m, s))) {
            readout(
                ui,
                &sections::format_metric(sample.value, &m.id),
                m.id.label,
                true,
            );
        }
        if let Some(session) = session {
            note(ui, &format!("measured · {}", describe(session)), false);
            if let Some(warning) = &session.note {
                note(ui, warning, true);
            }
        }
    });
    if let Some(session) = session {
        cpu_sections(ui, lab, session, fx);
    }
    if row.backend == Backend::Gpu
        && let Some((_, sim)) = data.lenia.iter().find(|(lid, _)| **lid == id)
    {
        lenia::show(ui, id, sim, metric, fx);
        margin_section(ui, "grid", false, |ui| {
            ui.label(mono(super::stage::overlay::gpu_arena(sim)).color(INFO));
        });
    }
    colour(ui, row, fx);
    if let Some(specimen) = data.specimens.get(&row.kind) {
        let figure = data.specimens.figure(&row.kind).unwrap_or(1) as u32;
        let modified = session.filter(|s| sections::customised(s));
        margin_section(ui, "", false, |ui| {
            if let Some(session) = modified {
                let text = format!(
                    "Catalog specimen described below; this channel runs {}.",
                    session.title()
                );
                note(ui, &text, true);
            }
            if data
                .lenia
                .iter()
                .any(|(id, sim)| *id == row.id && sim.kernel.is_some())
            {
                note(
                    ui,
                    "Continuous XYZ model: catalog text below describes the original XY specimen.",
                    true,
                );
            }
            let (explanation, citation) = (&specimen.explanation, &specimen.citation);
            if let Some(url) = footnote(ui, figure, explanation, citation, &specimen.doi) {
                fx.open_url = Some(url);
            }
        });
    }
}

fn cpu_sections(ui: &mut Ui, lab: &mut LabState, session: &Session, fx: &mut Effects) {
    let id = session.channel;
    parameters::show(ui, session, &mut lab.params, fx);
    super::stage::editing::show(ui, &mut lab.editor, session, fx);
    if let Model::Symbolic(machine) = &session.model {
        margin_section(ui, "machine", false, |ui| {
            for (key, value) in machine::summary(machine) {
                ui.horizontal(|ui| {
                    ui.label(serif_italic(key));
                    ui.label(mono(value));
                });
            }
            if let Symbolic::Turing(tm) = machine {
                for line in machine::turing_table(tm) {
                    ui.label(mono(line).color(INK));
                }
            }
        });
        input(ui, lab, id, session, fx);
    }
    margin_section(ui, "grid", false, |ui| {
        ui.label(mono(sections::arena_label(&session.model, session.kind)).color(INK_DIM));
    });
}

fn input(ui: &mut Ui, lab: &mut LabState, id: ChannelId, session: &Session, fx: &mut Effects) {
    margin_section(ui, "input", false, |ui| {
        let draft = lab
            .inputs
            .entry(id)
            .or_insert_with(|| session.input.clone());
        let field = egui::TextEdit::singleline(draft).char_limit(1024);
        if ui.add(field).changed() {
            fx.commands.push(ChannelCommand {
                id,
                action: ChannelAction::SetInput(draft.clone()),
            });
        }
        let (text, warn) = sections::input_note(draft.chars().count());
        note(ui, &text, warn);
    });
}

fn colour(ui: &mut Ui, row: &ChannelRow, fx: &mut Effects) {
    let id = row.id;
    margin_section(ui, "colour", false, |ui| {
        ui.horizontal_wrapped(|ui| {
            for swatch in SWATCHES {
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(14.0, 14.0), Sense::click());
                if row.settings.colour == swatch {
                    ui.painter().circle_filled(rect.center(), 7.0, INK_HI);
                    ui.painter().circle_filled(rect.center(), 5.5, BG);
                }
                ui.painter()
                    .circle_filled(rect.center(), 5.0, color(swatch));
                if response.clicked() {
                    fx.commands.push(ChannelCommand {
                        id,
                        action: ChannelAction::SetColour(swatch),
                    });
                }
            }
            let mut rgb = row.settings.colour.0;
            if egui::color_picker::color_edit_button_srgb(ui, &mut rgb).changed() {
                fx.commands.push(ChannelCommand {
                    id,
                    action: ChannelAction::SetColour(Srgb(rgb)),
                });
            }
        });
    });
}
