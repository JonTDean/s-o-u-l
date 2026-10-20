//! Pure formatting helpers for inspector sections.
use engine_core::automata::channel::metrics::MetricId;
use models::workbench::{
    catalog::Kind,
    experiment::config::Settings,
    session::{Model, Session},
};

/// Order parameters keep two decimals; counts are integers.
pub fn format_metric(value: f64, id: &MetricId) -> String {
    if id.name == "polarisation" || id.name == "mass" {
        format!("{value:.2}")
    } else {
        format!("{value:.0}")
    }
}

/// Size, boundary and backend for the `grid` margin section.
pub fn arena_label(model: &Model, kind: Kind) -> String {
    match model {
        Model::Grid(g) if matches!(kind, Kind::Rule30 | Kind::Rule110) => {
            format!("{} cells · 64-row history · CPU", g.grid.size.x)
        }
        Model::Grid(g) => format!(
            "{} × {} · {} · CPU",
            g.grid.size.x,
            g.grid.size.y,
            match g.boundary {
                simulation_kernel::core::boundary::Boundary::Dead => "dead edges",
                simulation_kernel::core::boundary::Boundary::Periodic => "periodic",
            }
        ),
        Model::Boids(_) => "64 × 64 · wraps · CPU".into(),
        Model::Ants(_) => "64 × 64 · clamped · CPU".into(),
        Model::Symbolic(_) => "tape · CPU".into(),
    }
}

/// Whether a session's rule, seed pattern or population departs from the catalog
/// specimen, so its authored title, caption and footnote no longer describe it.
pub fn customised(session: &Session) -> bool {
    let (s, d) = (&session.settings, Settings::for_kind(session.kind));
    s.program != d.program
        || s.elementary != d.elementary
        || s.birth != d.birth
        || s.survival != d.survival
        || s.agent_count != d.agent_count
        || s.pattern != d.pattern
        || s.world != d.world
}

/// Input hint and whether it is a warning (over the 256-symbol bound).
pub fn input_note(len: usize) -> (String, bool) {
    if len > 256 {
        (format!("{len}/256: too long, Reset is refused"), true)
    } else {
        (format!("applies on reset · {len}/256"), false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::workbench::{catalog::Kind, session::Session};

    #[test]
    fn metric_values_format_by_kind() {
        let order = MetricId {
            name: "polarisation",
            label: "order",
            unit: "0–1",
        };
        let pop = MetricId {
            name: "population",
            label: "alive",
            unit: "cells",
        };
        assert_eq!(format_metric(0.912_3, &order), "0.91");
        assert_eq!(format_metric(212.0, &pop), "212");
    }

    #[test]
    fn arenas_describe_size_and_edges() {
        let label = |kind| arena_label(&Session::new(kind).model, kind);
        assert_eq!(label(Kind::Life), "64 × 64 · dead edges · CPU");
        assert_eq!(label(Kind::Rule30), "64 cells · 64-row history · CPU");
        assert_eq!(label(Kind::Boids), "64 × 64 · wraps · CPU");
    }

    #[test]
    fn edited_rules_and_populations_are_customised() {
        let mut session = Session::new(Kind::Rule30);
        assert!(!customised(&session));
        session.settings.rate_hz = 60.0;
        session.settings.colour = engine_core::automata::channel::SWATCHES[1];
        assert!(
            !customised(&session),
            "pace and colour do not change the specimen"
        );
        session.settings.elementary = 90;
        assert!(customised(&session));
        let mut ants = Session::new(Kind::Ants);
        ants.settings.agent_count = 8;
        assert!(customised(&ants), "captions state the population");
    }

    #[test]
    fn input_note_warns_beyond_the_bound() {
        assert_eq!(
            input_note(6),
            ("applies on reset · 6/256".to_string(), false)
        );
        assert!(input_note(300).1);
    }
}
