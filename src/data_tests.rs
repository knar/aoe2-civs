//! Invariants of the generated catalog (`src/data.rs`).
//!
//! These guard the bake: if upstream changes shape or the generator regresses,
//! the counts and structural expectations here fail loudly.

use crate::data::{CIVS, OPTIONS};
use crate::model::Group;
use std::collections::HashSet;

#[test]
fn catalog_counts_are_pinned() {
    assert_eq!(CIVS.len(), 53, "civ count changed");
    assert_eq!(OPTIONS.len(), 369, "option count changed");

    let units = OPTIONS
        .iter()
        .filter(|o| o.keys[0].group == Group::Unit)
        .count();
    let techs = OPTIONS.len() - units;
    assert_eq!(units, 209, "unit option count changed");
    assert_eq!(techs, 160, "tech option count changed");

    let unique_units = OPTIONS
        .iter()
        .filter(|o| o.unique && o.group == Group::Unit)
        .count();
    let unique_techs = OPTIONS
        .iter()
        .filter(|o| o.unique && o.group == Group::Tech)
        .count();
    assert_eq!(unique_units, 143, "unique unit option count changed");
    assert_eq!(unique_techs, 109, "unique tech option count changed");
}

#[test]
fn unique_flag_means_matches_exactly_one_civ() {
    for option in OPTIONS {
        let matching = CIVS
            .iter()
            .filter(|civ| option.keys.iter().any(|key| civ.keys.contains(key)))
            .count();
        assert_eq!(
            option.unique,
            matching == 1,
            "{}: unique={} but matches {matching} civs",
            option.label,
            option.unique
        );
    }
}

#[test]
fn option_group_matches_its_keys() {
    for option in OPTIONS {
        assert!(
            option.keys.iter().all(|key| key.group == option.group),
            "{}: group {:?} disagrees with its keys",
            option.label,
            option.group
        );
    }
}

#[test]
fn food_gold_classifies_wood_free_units() {
    fn option(label: &str) -> &crate::data::CivOption {
        OPTIONS
            .iter()
            .find(|o| o.label == label)
            .unwrap_or_else(|| panic!("no option {label}"))
    }
    assert!(option("Siege Elephant").food_gold);
    assert!(option("Armored Elephant").food_gold);
    assert!(option("Huskarl").food_gold);
    assert!(option("War Elephant").food_gold);
    assert!(option("Steppe Lancer").food_gold);
    assert!(!option("Siege Ram").food_gold);
    assert!(!option("Mangudai").food_gold);
    assert!(!option("Longbowman").food_gold);
    assert!(!option("Chu Ko Nu").food_gold);
    assert!(!option("Fire Lancer").food_gold);
    assert!(!option("Camel Archer").food_gold);
    assert!(option("Champion").food_gold);
    assert!(option("Hand Cannoneer").food_gold);
}

#[test]
fn every_option_is_well_formed() {
    for option in OPTIONS {
        assert!(!option.label.is_empty(), "empty label");
        assert_eq!(option.label, option.label.trim(), "untrimmed label");
        assert!(!option.keys.is_empty(), "{} has no keys", option.label);
        let group = option.keys[0].group;
        assert!(
            option.keys.iter().all(|key| key.group == group),
            "{} mixes unit and tech keys",
            option.label
        );
        assert!(
            option.icon == "missing"
                || (option.icon.starts_with("Unit/") || option.icon.starts_with("Tech/"))
                    && option.icon[5..].parse::<u32>().is_ok(),
            "{} has an ill-formed icon {:?}",
            option.label,
            option.icon
        );
    }
}

#[test]
fn option_labels_are_unique() {
    let mut seen = HashSet::new();
    for option in OPTIONS {
        assert!(seen.insert(option.label), "duplicate label {:?}", option.label);
    }
}

#[test]
fn dataset_keys_belong_to_one_option_only() {
    let mut seen = HashSet::new();
    for option in OPTIONS {
        for key in option.keys {
            assert!(
                seen.insert(*key),
                "key {key:?} appears in more than one option"
            );
        }
    }
}

#[test]
fn no_option_is_universal() {
    let total = CIVS.len();
    for option in OPTIONS {
        let matching = CIVS
            .iter()
            .filter(|civ| option.keys.iter().any(|key| civ.keys.contains(key)))
            .count();
        assert!(
            matching < total,
            "{} is universal ({matching}/{total}) and should have been excluded",
            option.label
        );
    }
}

#[test]
fn every_option_matches_at_least_one_civ() {
    for option in OPTIONS {
        let matching = CIVS
            .iter()
            .any(|civ| option.keys.iter().any(|key| civ.keys.contains(key)));
        assert!(matching, "{} matches no civ", option.label);
    }
}

#[test]
fn civ_names_are_unique() {
    let mut seen = HashSet::new();
    for civ in CIVS {
        assert!(seen.insert(civ.name), "duplicate civ {:?}", civ.name);
        assert!(!civ.keys.is_empty(), "{} has no keys", civ.name);
    }
}
