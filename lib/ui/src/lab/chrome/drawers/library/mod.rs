//! Specimen library drawer: grouped, filterable, opens a channel on click.
use crate::{
    lab::{Effects, LabData, state::LabState},
    theme::{
        tokens::*,
        widgets::{dim, mono, note, pill, plain, section},
    },
};
use bevy_egui::egui::{self, Sense, Ui};
use engine_core::{automata::channel::ChannelIds, events::AutomataCommand};
use models::workbench::{
    catalog::specimens::{Group, Specimen, Specimens},
    plugin::FULL_NOTE,
};
use std::collections::HashSet;
pub mod authored;
pub mod guides;

/// Library groups in display order, omitting empty groups; filter is case-insensitive.
pub fn entries<'a>(
    specimens: &'a Specimens,
    available: &HashSet<String>,
    filter: &str,
) -> Vec<(Group, Vec<&'a Specimen>)> {
    let needle = filter.trim().to_lowercase();
    let matches = |s: &Specimen, group: Group| {
        needle.is_empty()
            || [s.title.as_str(), s.subtitle.as_str(), group.label()]
                .iter()
                .any(|t| t.to_lowercase().contains(&needle))
    };
    Group::ALL
        .into_iter()
        .map(|group| {
            let items = specimens
                .0
                .iter()
                .filter(|s| s.group == group && available.contains(&s.id) && matches(s, group));
            (group, items.collect::<Vec<_>>())
        })
        .filter(|(_, items)| !items.is_empty())
        .collect()
}

/// Drawer body: filter, grouped specimens and the open-channel count.
pub fn body(ui: &mut Ui, lab: &mut LabState, data: &LabData, fx: &mut Effects) {
    let filter = egui::TextEdit::singleline(&mut lab.filter)
        .hint_text(dim("filter specimens…"))
        .frame(false);
    ui.add(filter);
    ui.add_space(10.0);
    let full = data.channels.is_full();
    if pill(ui, "Published Orbium · polynomial XY", false, !full).clicked() {
        fx.automata.push(AutomataCommand::RestoreLenia(Box::new(
            models::workbench::project::library::orbium::specimen(),
        )));
        lab.drawer = None;
    }
    for (group, items) in entries(&data.specimens, &data.available(), &lab.filter) {
        section(group.label()).show(ui, |ui| {
            for specimen in items {
                let clicked = ui
                    .horizontal(|ui| {
                        let title = egui::Label::new(plain(&specimen.title).color(INK));
                        let subtitle = egui::Label::new(dim(&specimen.subtitle));
                        let mut response = ui.add(title.sense(Sense::click()))
                            | ui.add(subtitle.sense(Sense::click()));
                        if specimen.id == "lenia" {
                            response |= ui.label(mono("GPU").size(size::LABEL).color(INFO));
                        }
                        response.clicked()
                    })
                    .inner;
                if clicked && !full {
                    fx.automata.push(AutomataCommand::SeedPattern {
                        id: specimen.id.clone(),
                    });
                    lab.drawer = None;
                }
            }
        });
    }
    section("channels").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(mono(format!("{}", data.channels.rows.len())));
            ui.label(dim(format!("of {} open", ChannelIds::CAPACITY)));
            if pill(ui, "clear all", false, !data.channels.rows.is_empty()).clicked() {
                fx.automata.push(AutomataCommand::Clear);
            }
        });
        if full {
            note(ui, FULL_NOTE, true);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::workbench::catalog::specimens::Specimens;

    fn available() -> HashSet<String> {
        ["life:conway", "lenia", "regular:nfa", "swarm:boids"]
            .map(String::from)
            .into()
    }

    #[test]
    fn groups_follow_library_order_and_hide_unavailable_specimens() {
        let specimens = Specimens::embedded();
        let groups = entries(&specimens, &available(), "");
        let names: Vec<_> = groups.iter().map(|(g, _)| g.label()).collect();
        assert_eq!(
            names,
            ["cellular", "machines", "agents"],
            "empty groups are omitted"
        );
        assert_eq!(groups[0].1.len(), 2);
    }

    #[test]
    fn filter_matches_title_subtitle_and_group_case_insensitively() {
        let specimens = Specimens::embedded();
        let only = |f| {
            entries(&specimens, &available(), f)
                .into_iter()
                .flat_map(|(_, s)| s)
                .map(|s| s.id.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(only("GAME"), ["life:conway"]);
        assert_eq!(only("a*b"), ["regular:nfa"]);
        assert_eq!(only("agents"), ["swarm:boids"]);
        assert!(only("zzz").is_empty());
    }
}
