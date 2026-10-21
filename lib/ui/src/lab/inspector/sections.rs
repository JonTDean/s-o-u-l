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
    s.energy != d.energy
        || s.program != d.program
        || s.elementary != d.elementary
        || s.birth != d.birth
        || s.survival != d.survival
        || s.agent_count != d.agent_count
        || s.pattern != d.pattern
        || s.world != d.world
}

/// Folded "About this specimen" summary: figure, first author's surname and year.
pub fn about_summary(figure: u32, citation: &str) -> String {
    let fallback = format!("Fig. {figure}");
    let surname = citation
        .split_once(',')
        .and_then(|(authors, _)| authors.split_whitespace().last());
    let year = citation.rmatch_indices('(').find_map(|(i, _)| {
        let digits = citation[i + 1..]
            .strip_suffix(')')
            .unwrap_or(&citation[i + 1..]);
        let digits = digits.split(')').next()?;
        (digits.len() == 4 && digits.bytes().all(|b| b.is_ascii_digit())).then_some(digits)
    });
    match (surname, year) {
        (Some(surname), Some(year)) => format!("{fallback} · {surname} {year}"),
        _ => fallback,
    }
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
    fn about_summary_names_first_author_and_year() {
        let chan = "B. W.-C. Chan, \"Lenia: Biology of Artificial Life\", Complex Systems 28(3), 251–286 (2019).";
        assert_eq!(about_summary(5, chan), "Fig. 5 · Chan 2019");
        let hardy = "J. Hardy, Y. Pomeau, O. de Pazzis, \"Time evolution\", J. Math. Phys. 14(12), 1746–1759 (1973).";
        assert_eq!(about_summary(4, hardy), "Fig. 4 · Hardy 1973");
    }

    #[test]
    fn about_summary_falls_back_to_figure() {
        assert_eq!(about_summary(3, "Unpublished notes"), "Fig. 3");
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
