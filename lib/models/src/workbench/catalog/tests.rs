//! The embedded specimen file covers every family exactly once with valid, honest metadata.
use super::specimens::*;
use crate::workbench::{catalog::Kind, operations::channels::metrics::metric_for};

#[test]
fn every_family_and_lenia_has_exactly_one_entry() {
    let specimens = Specimens::embedded();
    let mut expected: Vec<String> = Kind::ALL.iter().map(|k| k.id().to_string()).collect();
    expected.push("lenia".into());
    let mut actual: Vec<String> = specimens.0.iter().map(|s| s.id.clone()).collect();
    expected.sort();
    actual.sort();
    assert_eq!(actual, expected);
}

#[test]
fn fields_are_present_and_dois_valid() {
    for s in Specimens::embedded().0 {
        for (name, value) in [
            ("title", &s.title),
            ("subtitle", &s.subtitle),
            ("caption", &s.caption),
            ("explanation", &s.explanation),
            ("citation", &s.citation),
        ] {
            assert!(!value.trim().is_empty(), "{} missing {name}", s.id);
        }
        assert!(valid_doi(&s.doi), "{} has malformed DOI {}", s.id, s.doi);
        assert_eq!(s.doi_url(), format!("https://doi.org/{}", s.doi));
    }
}

#[test]
fn metric_names_match_the_metric_sources() {
    let specimens = Specimens::embedded();
    for kind in Kind::ALL {
        assert_eq!(
            specimens.get(kind.id()).unwrap().metric,
            metric_for(kind).name,
            "{kind:?}"
        );
    }
    assert_eq!(specimens.get("lenia").unwrap().metric, "mass");
    assert_eq!(specimens.get("lenia:orbium").unwrap().id, "lenia");
}

#[test]
fn copy_keeps_contract_disclaimers() {
    let specimens = Specimens::embedded();
    assert!(
        specimens
            .get("lenia")
            .unwrap()
            .explanation
            .contains("not isotropic 3-D Lenia")
    );
    assert!(
        specimens
            .get("turing:replace-a")
            .unwrap()
            .explanation
            .contains("not a universal machine")
    );
}

#[test]
fn doi_syntax_and_unknown_groups() {
    assert!(valid_doi("10.1145/37402.37406"));
    assert!(!valid_doi("10.12/x"), "registrant needs 4–9 digits");
    assert!(!valid_doi("doi:10.1145/1"));
    assert!(!valid_doi("10.1145/"));
    let bad = "[[specimen]]\nid='x'\ntitle='x'\ngroup='planets'\nsubtitle='x'\ncaption='x'\nexplanation='x'\nmetric='x'\ncitation='x'\ndoi='10.1145/1'\n";
    assert!(Specimens::parse(bad).is_err());
}

#[test]
fn figure_numbers_follow_file_order() {
    let specimens = Specimens::embedded();
    assert_eq!(specimens.figure("wolfram:rule30"), Some(1));
    assert_eq!(specimens.figure("missing"), None);
}
