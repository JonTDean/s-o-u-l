use super::{
    Rule, Transition,
    composition::{Blend, Layer, Program},
};
use crate::workbench::{
    catalog::Kind,
    session::{Model, Session},
};
fn field(s: &Session) -> Vec<u8> {
    let Model::Grid(g) = &s.model else {
        panic!("grid")
    };
    g.bytes()
}
#[test]
fn ordered_patterns_validate_and_cover_all_neighborhoods() {
    let life = Rule::life(8, 12);
    life.validate().unwrap();
    for n in 0u16..512 {
        let count = (n & !16).count_ones();
        assert_eq!(life.evaluate(n), count == 3 || (n & 16 != 0 && count == 2));
        assert_eq!(Rule::copy_west().evaluate(n), n & 8 != 0);
    }
    let mut rule = Rule::copy_west();
    rule.transitions.insert(
        0,
        Transition {
            mask: 0,
            pattern: 0,
            alive: false,
        },
    );
    assert!(!rule.evaluate(511), "first match wins");
    rule.version = 2;
    assert!(rule.validate().is_err());
    rule.version = 1;
    rule.transitions[0].pattern = 512;
    assert!(rule.validate().is_err());
    rule.transitions.clear();
    rule.name = "\n".into();
    assert!(rule.validate().is_err());
}
#[test]
fn directional_program_and_composition_follow_snapshot_semantics() {
    let mut s = Session::new(Kind::Life);
    let mut settings = s.settings.clone();
    settings.world.width = 8;
    settings.world.height = 8;
    settings.world.seed = Some(vec![0; 64]);
    settings.world.seed.as_mut().unwrap()[27] = 255;
    settings.program = Some(Program {
        primary: Rule::copy_west(),
        secondary: None,
    });
    s.configure(settings.clone(), true).unwrap();
    s.step();
    assert_eq!(field(&s).iter().position(|v| *v != 0), Some(28));
    assert_eq!(field(&s).iter().filter(|v| **v != 0).count(), 1);
    for blend in [Blend::Union, Blend::Intersection, Blend::ExclusiveOr] {
        settings.program.as_mut().unwrap().secondary = Some(Layer {
            rule: Rule::copy_west(),
            blend,
        });
        s.configure(settings.clone(), true).unwrap();
        s.step();
        assert_eq!(field(&s)[28] != 0, blend != Blend::ExclusiveOr);
    }
    let before = serde_json::to_value(&s).unwrap();
    settings.program.as_mut().unwrap().primary.version = 42;
    assert!(s.configure(settings, false).is_err());
    assert_eq!(serde_json::to_value(&s).unwrap(), before);
}
#[test]
fn authored_life_matches_existing_rule_and_continues_after_serialization() {
    let mut reference = Session::new(Kind::Life);
    let mut custom = Session::new(Kind::Life);
    custom.settings.program = Some(Program {
        primary: Rule::life(8, 12),
        secondary: None,
    });
    for _ in 0..12 {
        reference.step();
        custom.step();
        assert_eq!(field(&reference), field(&custom));
    }
    let mut restored: Session =
        serde_json::from_value(serde_json::to_value(&custom).unwrap()).unwrap();
    for _ in 0..10 {
        restored.step();
        custom.step();
        assert_eq!(field(&restored), field(&custom));
    }
}
#[test]
fn portable_rules_reject_bad_files_and_never_replace_existing_exports() {
    use super::storage;
    let program = Program {
        primary: Rule::copy_west(),
        secondary: None,
    };
    let text = serde_json::to_string(&program).unwrap();
    assert_eq!(storage::parse(&text).unwrap(), program);
    assert!(storage::parse(&text.replace("\"version\":1", "\"version\":99")).is_err());
    assert!(storage::parse(&" ".repeat(storage::MAX_BYTES + 1)).is_err());
    assert!(storage::parse("{\"primary\":null}").is_err());
    let root = std::env::temp_dir().join(format!("soul-rule-files-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("rule.json");
    let _ = std::fs::remove_file(&path);
    storage::write(&path, &program).unwrap();
    assert_eq!(storage::read(&path).unwrap(), program);
    let mut changed = program.clone();
    changed.primary.default_alive = true;
    assert!(storage::write(&path, &changed).is_err());
    assert_eq!(storage::read(&path).unwrap(), program);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn invalid_masks_lengths_and_foreign_channels_are_rejected() {
    let mut rule = Rule::copy_west();
    rule.transitions[0].mask = 512;
    assert!(rule.validate().is_err());
    rule.transitions = vec![
        Transition {
            mask: 0,
            pattern: 0,
            alive: false
        };
        513
    ];
    assert!(rule.validate().is_err());
    rule.transitions.clear();
    rule.default_alive = true;
    assert!(rule.validate().is_ok());
    assert!(rule.evaluate(0));
    let mut s = Session::new(Kind::Rule30);
    let mut settings = s.settings.clone();
    settings.program = Some(Program {
        primary: rule,
        secondary: None,
    });
    assert!(s.configure(settings, true).is_err());
    for blend in [Blend::Union, Blend::Intersection, Blend::ExclusiveOr] {
        for left in [false, true] {
            for right in [false, true] {
                let expected = match blend {
                    Blend::Union => left || right,
                    Blend::Intersection => left && right,
                    Blend::ExclusiveOr => left != right,
                };
                assert_eq!(blend.combine(left, right), expected);
            }
        }
    }
}
