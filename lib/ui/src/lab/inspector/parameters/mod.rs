//! `parameters` margin section: experiment settings drafts applied through `ConfigureSession`.
use crate::{
    lab::Effects,
    theme::widgets::{margin_section, note, pill},
};
use bevy_egui::egui::Ui;
use engine_core::automata::channel::ChannelId;
use models::workbench::{
    experiment::config::Settings, operations::channels::ConfigureSession, session::Session,
};
use std::collections::HashMap;
mod authored;
mod fields;
mod world;

/// An edited copy of a session's settings and the settings it was taken from.
#[derive(Clone, Debug)]
pub struct Draft {
    original: Settings,
    edited: Settings,
    editor: authored::Editor,
}

/// The draft for `id`, replaced whenever the session's settings changed elsewhere.
pub fn draft<'a>(
    drafts: &'a mut HashMap<ChannelId, Draft>,
    id: ChannelId,
    current: &Settings,
) -> &'a mut Settings {
    let entry = drafts.entry(id).or_insert_with(|| Draft {
        original: current.clone(),
        edited: current.clone(),
        editor: Default::default(),
    });
    if entry.original != *current {
        // The transport and colour swatches change pace and colour directly; keep any
        // pending rule edits and follow them. Anything else replaces the draft.
        let mut paced = entry.original.clone();
        (paced.rate_hz, paced.colour) = (current.rate_hz, current.colour);
        if paced == *current {
            (entry.edited.rate_hz, entry.edited.colour) = (current.rate_hz, current.colour);
            entry.original = current.clone();
        } else {
            *entry = Draft {
                original: current.clone(),
                edited: current.clone(),
                editor: Default::default(),
            };
        }
    }
    &mut entry.edited
}

/// Buttons under the parameter fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Rate and rule parameters take effect immediately.
    ApplyLive,
    /// Apply and reset (seed and population changes need this).
    ApplyReset,
    /// Restore the family defaults into the draft only.
    Defaults,
}

/// The backend request for an action; `Defaults` only edits the draft.
pub fn request(id: ChannelId, settings: &Settings, action: Action) -> Option<ConfigureSession> {
    let reset = match action {
        Action::ApplyLive => false,
        Action::ApplyReset => true,
        Action::Defaults => return None,
    };
    Some(ConfigureSession {
        id,
        settings: Box::new(settings.clone()),
        reset,
    })
}

/// Draw the section for one CPU session.
pub fn show(
    ui: &mut Ui,
    session: &Session,
    drafts: &mut HashMap<ChannelId, Draft>,
    fx: &mut Effects,
) {
    let id = session.channel;
    margin_section(ui, "parameters", false, |ui| {
        draft(drafts, id, &session.settings);
        let entry = drafts.get_mut(&id).unwrap();
        let edited = &mut entry.edited;
        fields::draw(ui, session.kind, edited);
        if session.kind == models::workbench::catalog::Kind::Life {
            authored::draw(ui, edited, &mut entry.editor);
        }
        note(
            ui,
            "Rate and rule apply live; seed and population need a reset.",
            false,
        );
        ui.horizontal_wrapped(|ui| {
            for (label, action) in [
                ("Apply live", Action::ApplyLive),
                ("Apply and reset", Action::ApplyReset),
                ("Defaults", Action::Defaults),
            ] {
                if pill(ui, label, action == Action::ApplyLive, true).clicked() {
                    match request(id, edited, action) {
                        Some(request) => fx.configure.push(request),
                        None => *edited = Settings::for_kind(session.kind),
                    }
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::workbench::catalog::Kind;

    #[test]
    fn drafts_survive_frames_and_follow_external_changes() {
        let mut drafts = HashMap::new();
        let id = ChannelId(2);
        let mut current = Settings::for_kind(Kind::Life);
        draft(&mut drafts, id, &current).birth = 0b100_1000; // B36, not the B3 default
        assert_eq!(
            draft(&mut drafts, id, &current).birth,
            0b100_1000,
            "edits persist"
        );
        current.rate_hz = 30.0; // applied elsewhere, e.g. the transport
        current.colour = engine_core::automata::channel::SWATCHES[2];
        let merged = draft(&mut drafts, id, &current).clone();
        assert_eq!(
            merged.birth, 0b100_1000,
            "transport rate and colour keep pending edits"
        );
        assert_eq!(
            (merged.rate_hz, merged.colour),
            (current.rate_hz, current.colour)
        );
        current.elementary = 90; // a rule applied elsewhere replaces the draft
        assert_eq!(*draft(&mut drafts, id, &current), current);
    }

    #[test]
    fn actions_become_configure_requests() {
        let id = ChannelId(5);
        let settings = Settings::for_kind(Kind::Boids);
        let live = request(id, &settings, Action::ApplyLive).unwrap();
        assert_eq!((live.id, live.reset), (id, false));
        assert!(request(id, &settings, Action::ApplyReset).unwrap().reset);
        assert!(request(id, &settings, Action::Defaults).is_none());
    }
}
