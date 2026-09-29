//! DM-michi civ overview: rule-driven "spam & wood units" sheet per civ.
//!
//! Spam (no wood) = food+gold units: every unique food+gold unit the civ
//! holds, plus fixed rule slots (heavy camel, paladin, battle & siege
//! elephants, hand cannons, elephant archers, eagles, steppe lancers) and the
//! Varangian Guard regional unit (a strong non-unique spam pick).
//! Champion appears only for hand-curated civs where it is a real pick.
//! Wood units (usually siege) = every unique unit that does cost wood, plus
//! the siege/defense lines (ram, onager/scorpion lines, bombards) and Arbalest
//! for hand-curated civs. Naval units never appear (land-only profile).
//! Nothing is ever shown as "absent"; quieter sections render an empty muted
//! line instead. Rows show icons rather than text.

use crate::data::OPTIONS;
use crate::model::{Civ, FilterKey, Group};

const fn uk(data_id: u32) -> FilterKey {
    FilterKey {
        group: Group::Unit,
        data_id,
    }
}

const fn tk(data_id: u32) -> FilterKey {
    FilterKey {
        group: Group::Tech,
        data_id,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Section {
    Spam,
    Wood,
}

impl Section {
    pub const fn title(self) -> &'static str {
        match self {
            Section::Spam => "Spam (no wood)",
            Section::Wood => "Wood units (usually siege)",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Upgrade {
    pub key: FilterKey,
    pub name: &'static str,
}

pub struct Slot {
    pub label: &'static str,
    pub section: Section,
    /// Keys that make the primary line available (any one is enough).
    pub keys: &'static [FilterKey],
    /// Shown when the primary line is missing but this cheaper line is not.
    pub fallback: Option<(&'static str, &'static [FilterKey])>,
    /// (`Elite <label>`, key) shown once the elite upgrade key is present.
    pub elite: Option<(&'static str, FilterKey)>,
    /// Researchable upgrades that must all be present for a fully upgraded
    /// unit. Everything universal (all ages, all civs) is omitted.
    pub fu: &'static [Upgrade],
}

const IRON_CASTING: Upgrade = Upgrade {
    key: tk(68),
    name: "Iron Casting",
};
const BLAST_FURNACE: Upgrade = Upgrade {
    key: tk(75),
    name: "Blast Furnace",
};
const BLOODLINES: Upgrade = Upgrade {
    key: tk(435),
    name: "Bloodlines",
};
const HUSBANDRY: Upgrade = Upgrade {
    key: tk(39),
    name: "Husbandry",
};
const SCALE_BARDING: Upgrade = Upgrade {
    key: tk(81),
    name: "Scale Barding Armor",
};
const CHAIN_BARDING: Upgrade = Upgrade {
    key: tk(82),
    name: "Chain Barding Armor",
};
const PLATE_BARDING: Upgrade = Upgrade {
    key: tk(80),
    name: "Plate Barding Armor",
};

const CAV_FU: &[Upgrade] = &[
    IRON_CASTING,
    BLAST_FURNACE,
    BLOODLINES,
    HUSBANDRY,
    SCALE_BARDING,
    CHAIN_BARDING,
    PLATE_BARDING,
];

const ELEPHANT_FU: &[Upgrade] = &[
    IRON_CASTING,
    BLAST_FURNACE,
    SCALE_BARDING,
    CHAIN_BARDING,
    PLATE_BARDING,
];

const LANCER_FU: &[Upgrade] = &[IRON_CASTING, BLAST_FURNACE, BLOODLINES];

const CHAMPION_FU: &[Upgrade] = &[
    IRON_CASTING,
    BLAST_FURNACE,
    Upgrade {
        key: tk(602),
        name: "Arson",
    },
    Upgrade {
        key: tk(215),
        name: "Squires",
    },
    Upgrade {
        key: tk(76),
        name: "Chain Mail Armor",
    },
    Upgrade {
        key: tk(77),
        name: "Plate Mail Armor",
    },
];

const ARCHER_FU: &[Upgrade] = &[
    Upgrade {
        key: tk(201),
        name: "Bracer",
    },
    Upgrade {
        key: tk(212),
        name: "Leather Archer Armor",
    },
    Upgrade {
        key: tk(219),
        name: "Ring Archer Armor",
    },
    Upgrade {
        key: tk(437),
        name: "Thumb Ring",
    },
];

/// Always-shown spam slots: a rule slot is emitted only when the civ actually
/// holds it (absent rows are dropped, never marked ✗).
pub const SPAM_RULES: &[Slot] = &[
    Slot {
        label: "Heavy Camel Rider",
        section: Section::Spam,
        keys: &[uk(330)],
        fallback: None,
        elite: None,
        fu: CAV_FU,
    },
    Slot {
        label: "Paladin",
        section: Section::Spam,
        keys: &[uk(569)],
        fallback: None,
        elite: None,
        fu: CAV_FU,
    },
    Slot {
        label: "Battle Elephant",
        section: Section::Spam,
        keys: &[uk(1132)],
        fallback: None,
        elite: Some(("Elite Battle Elephant", uk(1134))),
        fu: ELEPHANT_FU,
    },
    Slot {
        label: "Siege Elephant",
        section: Section::Spam,
        keys: &[uk(1746)],
        fallback: Some(("Armored Elephant", &[uk(1744)])),
        elite: None,
        fu: &[],
    },
    Slot {
        label: "Hand Cannoneer",
        section: Section::Spam,
        keys: &[uk(5)],
        fallback: None,
        elite: None,
        fu: &[],
    },
    Slot {
        label: "Elite Elephant Archer",
        section: Section::Spam,
        keys: &[uk(875)],
        fallback: None,
        elite: None,
        fu: &[],
    },
    Slot {
        label: "Eagle Warrior",
        section: Section::Spam,
        keys: &[uk(753)],
        fallback: None,
        elite: Some(("Elite Eagle Warrior", uk(752))),
        fu: &[],
    },
    Slot {
        label: "Steppe Lancer",
        section: Section::Spam,
        keys: &[uk(1370)],
        fallback: None,
        elite: Some(("Elite Steppe Lancer", uk(1372))),
        fu: LANCER_FU,
    },
    Slot {
        label: "Varangian Guard",
        section: Section::Spam,
        keys: &[uk(2703)],
        fallback: None,
        elite: Some(("Elite Varangian Guard", uk(2704))),
        fu: &[],
    },
];

/// Always-shown wood-costing siege/defense slots; emitted only when present,
/// exact same drop rule. A missing top line swaps to the base unit's icon.
pub const WOOD_RULES: &[Slot] = &[
    Slot {
        label: "Siege Ram",
        section: Section::Wood,
        keys: &[uk(548)],
        fallback: Some(("Capped Ram", &[uk(422)])),
        elite: None,
        fu: &[],
    },
    Slot {
        label: "Siege Onager",
        section: Section::Wood,
        keys: &[uk(588)],
        fallback: Some(("Onager", &[uk(550)])),
        elite: None,
        fu: &[],
    },
    Slot {
        label: "Heavy Scorpion",
        section: Section::Wood,
        keys: &[uk(542)],
        fallback: None,
        elite: None,
        fu: &[],
    },
    Slot {
        label: "Heavy Rocket Cart",
        section: Section::Wood,
        keys: &[uk(1907)],
        fallback: Some(("Rocket Cart", &[uk(1904)])),
        elite: None,
        fu: &[],
    },
    Slot {
        label: "Bombard Cannon",
        section: Section::Wood,
        keys: &[uk(36)],
        fallback: None,
        elite: None,
        fu: &[],
    },
    Slot {
        label: "Bombard Tower",
        section: Section::Wood,
        keys: &[tk(64)],
        fallback: None,
        elite: None,
        fu: &[],
    },
];

pub const CHAMPION_SLOT: Slot = Slot {
    label: "Champion",
    section: Section::Spam,
    keys: &[uk(567)],
    fallback: Some(("Two-Handed Swordsman", &[uk(473)])),
    elite: None,
    fu: CHAMPION_FU,
};

pub const ARBALEST_SLOT: Slot = Slot {
    label: "Arbalester",
    section: Section::Wood,
    keys: &[uk(492)],
    fallback: Some(("Crossbowman", &[uk(24)])),
    elite: None,
    fu: ARCHER_FU,
};

pub const CAVALIER_SLOT: Slot = Slot {
    label: "Cavalier",
    section: Section::Spam,
    keys: &[uk(283)],
    fallback: None,
    elite: None,
    fu: CAV_FU,
};

/// Last-resort spam pick for civs whose rule slots and unique units stay out:
/// the best food+gold line they can field, in this order.
pub const COVERAGE: &[&Slot] = &[&CAVALIER_SLOT, &CHAMPION_SLOT];

/// Rule lines fully outclassed by a civ-specific unit: showing the generic one
/// is redundant (Imperial Camel Rider > Heavy Camel Rider, Houfnice > Bombard
/// Cannon). Keyed by the rule slot's label, then the replacing unit's id.
const REPLACES: &[(&str, u32)] = &[("Heavy Camel Rider", 207), ("Bombard Cannon", 1709)];

/// Labels whose row counts as a food+gold stable (cavalry) spam line, used to
/// short-circuit the guaranteed Cavalier so there is no double cavalry row.
const CAV_STABLE: &[&str] = &[
    "Cavalier",
    "Paladin",
    "Heavy Camel Rider",
    "Imperial Camel Rider",
    "Battle Elephant",
    "Elite Battle Elephant",
    "Siege Elephant",
    "Armored Elephant",
    "Steppe Lancer",
    "Elite Steppe Lancer",
];

/// Champion is worth a slot for these civs (national bonus carries it). Kept
/// in sync with NOTES by a test.
const SPECIAL_INF: &[&str] = &["Burmese", "Goths", "Japanese", "Teutons", "Vikings"];

/// Arbalest (Arbalester) is worth a slot for these civs (national bonus carries
/// it). Kept in sync with NOTES by a test.
const SPECIAL_RANGE: &[&str] = &[
    "Britons",
    "Ethiopians",
    "Italians",
    "Koreans",
    "Mayans",
    "Muisca",
    "Vietnamese",
];

#[derive(Clone, PartialEq, Debug)]
pub enum Status {
    /// Unit available and every listed upgrade researched.
    Full,
    /// Unit available but these upgrades are missing.
    Partial(Vec<&'static str>),
    /// Only the fallback line is available.
    Base(&'static str),
    /// Not rendered; kept so `evaluate` can signal a missing line.
    Absent,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Row {
    pub label: &'static str,
    pub section: Section,
    pub status: Status,
    /// Icon path under https://aoe2techtree.net/img/ (e.g. "Unit/42").
    pub icon: &'static str,
}

pub struct CivSummary {
    pub name: &'static str,
    pub rows: Vec<Row>,
    pub notes: &'static [&'static str],
    /// Siege Engineers (tk 377) is researched and the civ fields at least one
    /// siege tool, so it shows as a footnote rather than a row.
    pub siege_engineers: bool,
}

fn has_any(civ: &Civ, keys: &[FilterKey]) -> bool {
    keys.iter().any(|key| civ.keys.contains(key))
}

fn evaluate(civ: &Civ, slot: &Slot) -> Status {
    if has_any(civ, slot.keys) {
        let missing: Vec<&'static str> = slot
            .fu
            .iter()
            .filter(|up| !civ.keys.contains(&up.key))
            .map(|up| up.name)
            .collect();
        if missing.is_empty() {
            Status::Full
        } else {
            Status::Partial(missing)
        }
    } else if let Some((label, keys)) = slot.fallback {
        if has_any(civ, keys) {
            Status::Base(label)
        } else {
            Status::Absent
        }
    } else {
        Status::Absent
    }
}

/// Icon path of the option holding `key`; "missing" falls back to a stub file.
fn icon_of(key: FilterKey) -> &'static str {
    OPTIONS
        .iter()
        .find(|option| option.keys.contains(&key))
        .map_or("missing", |option| option.icon)
}

/// The `Elite <base>` option when the civ holds its keys (and it exists).
fn elite_option(civ: &Civ, base: &'static str) -> Option<&'static crate::data::CivOption> {
    let elite = format!("Elite {base}");
    OPTIONS
        .iter()
        .find(|option| option.label == elite)
        .filter(|option| has_any(civ, option.keys))
}

/// Push the slot only if the line (or its fallback) is present; absent lines
/// are dropped entirely (the "no ✗" rule). The row's icon follows the status:
/// the elite unit when the upgrade is held, otherwise the primary line, and
/// the cheaper base unit's icon when only that is available.
fn push_if(civ: &Civ, slot: &Slot, rows: &mut Vec<Row>) {
    if REPLACES
        .iter()
        .any(|(label, id)| *label == slot.label && civ.keys.contains(&uk(*id)))
    {
        return;
    }
    let status = evaluate(civ, slot);
    if let Status::Absent = status {
        return;
    }
    let (label, icon) = match (&status, slot.elite) {
        (Status::Base(_), _) => (slot.label, slot.fallback.map_or("missing", |(_, keys)| icon_of(keys[0]))),
        (_, Some((elabel, key))) if civ.keys.contains(&key) => (elabel, icon_of(key)),
        _ => (slot.label, icon_of(slot.keys[0])),
    };
    rows.push(Row {
        label,
        icon,
        section: slot.section,
        status,
    });
}

/// Every unique food+gold unit the civ holds, upgrading the label/icon to the
/// elite variant when its keys are present.
fn unique_rows(civ: &Civ, section: Section) -> Vec<Row> {
    let mut rows = Vec::new();
    for option in OPTIONS {
        let food_gold = option.food_gold == (section == Section::Spam);
        if option.group == Group::Unit
            && option.unique
            && !option.naval
            && food_gold
            && has_any(civ, option.keys)
        {
            // The elite variant is reached through its base label, so the
            // "Elite X" option itself would otherwise be counted twice.
            if !option.label.starts_with("Elite ") {
                let (label, icon) = elite_option(civ, option.label)
                    .map_or((option.label, option.icon), |e| (e.label, e.icon));
                rows.push(Row {
                    label,
                    icon,
                    section,
                    status: Status::Full,
                });
            }
        }
    }
    rows
}

fn spam_rows(civ: &Civ) -> Vec<Row> {
    let mut rows = unique_rows(civ, Section::Spam);
    for slot in SPAM_RULES {
        push_if(civ, slot, &mut rows);
    }
    if SPECIAL_INF.contains(&civ.name) {
        push_if(civ, &CHAMPION_SLOT, &mut rows);
    }
    if rows.is_empty() {
        for slot in COVERAGE {
            let pick = evaluate(civ, slot);
            if !matches!(pick, Status::Absent) {
                rows.push(Row {
                    label: slot.label,
                    icon: icon_of(slot.keys[0]),
                    section: Section::Spam,
                    status: pick,
                });
                break;
            }
        }
    }
    // Melee backbone guarantee: even when the civ's own picks are all ranged
    // or niche, list a generic food+gold line — Cavalier when the stable
    // allows it, Champion when it doesn't.
    if has_any(civ, CAVALIER_SLOT.keys) {
        if !rows.iter().any(|row| CAV_STABLE.contains(&row.label)) {
            push_if(civ, &CAVALIER_SLOT, &mut rows);
        }
    } else if rows.iter().all(|row| row.label != CHAMPION_SLOT.label) {
        push_if(civ, &CHAMPION_SLOT, &mut rows);
    }
    rows
}

fn tool_rows(civ: &Civ) -> Vec<Row> {
    let mut rows = Vec::new();
    for slot in WOOD_RULES {
        push_if(civ, slot, &mut rows);
    }
    rows
}

fn arbalest_rows(civ: &Civ) -> Vec<Row> {
    if !SPECIAL_RANGE.contains(&civ.name) {
        return Vec::new();
    }
    let mut rows = Vec::new();
    push_if(civ, &ARBALEST_SLOT, &mut rows);
    rows
}

pub fn summary_for(civ: &Civ) -> CivSummary {
    let tools = tool_rows(civ);
    let mut rows = spam_rows(civ);
    rows.extend(tools.iter().cloned());
    rows.extend(arbalest_rows(civ));
    rows.extend(unique_rows(civ, Section::Wood));
    CivSummary {
        name: civ.name,
        rows,
        notes: notes_for(civ.name),
        siege_engineers: civ.keys.contains(&tk(377)) && !tools.is_empty(),
    }
}

/// Notes keyed by civ name in the same order as CIVS. Only bonuses that stay
/// active after everything is researched (permanent unit stats, production
/// costs, farming/trade rates, building stats, wood gathering) are listed —
/// in DM michi unique/imperial techs resolve for free at the start, so their
/// permanent effects count too. Team bonuses are marked. One-time age-up
/// perks are skipped.
const NOTES: &[(&str, &[&str])] = &[
    (
        "Armenians",
        &[
            "Fereters researched: infantry +30 HP",
            "team: infantry +2 line of sight",
        ],
    ),
    ("Aztecs", &["military trains 15% faster"]),
    (
        "Bengalis",
        &[
            "elephants resist -25% bonus damage and conversion",
            "team: trade generates +10% food",
        ],
    ),
    ("Berbers", &["stable units cost -20% in imperial (cheap knights/camels)"]),
    ("Bohemians", &["spearman line deals +25% bonus damage (halbs)"]),
    ("Britons", &["foot archers +2 range in imperial"]),
    ("Bulgarians", &[]),
    ("Burgundians", &["gunpowder units attack +25%"]),
    (
        "Burmese",
        &[
            "infantry up to +3 attack in imperial; elephants +1/+1 armor",
            "infantry +1 attack per relic held (max +4)",
        ],
    ),
    (
        "Byzantines",
        &[
            "buildings +40% HP in imperial",
            "camels, skirmishers & spears cost -25% (trash spam)",
        ],
    ),
    (
        "Celts",
        &[
            "siege weapons attack 25% faster",
            "lumberjacks work +15% (wood famine)",
        ],
    ),
    ("Chinese", &["team: farms provide +10% food"]),
    ("Cumans", &["mounted units move +10% in imperial"]),
    (
        "Danes",
        &[
            "Hamask researched: infantry deal more damage as they lose HP",
            "Northmen's Fury: siege +40% vs buildings; mangonel-line +1 range",
            "team: siege weapons +line of sight",
        ],
    ),
    (
        "Dravidians",
        &[
            "skirmishers & elephant archers attack 25% faster",
            "siege weapons cost -33% wood",
        ],
    ),
    ("Ethiopians", &["foot archers attack 18% faster (arbalests)"]),
    ("Franks", &["mounted units have +20% HP"]),
    (
        "Georgians",
        &[
            "mounted units regenerate 14 HP/min",
            "units take -15% damage when on higher elevation",
        ],
    ),
    (
        "Goths",
        &["infantry cost -30% in imperial; +10 population space"],
    ),
    ("Gurjaras", &["mounted units deal +40% bonus damage in imperial"]),
    (
        "Hindustanis",
        &[
            "camel riders attack 20% faster",
            "gunpowder units get +1/+1 armor",
        ],
    ),
    ("Huns", &[]),
    ("Incas", &["military units cost -30% food in imperial (champions)"]),
    (
        "Italians",
        &["gunpowder units cost -20%", "foot archers get +1/+1 armor"],
    ),
    ("Japanese", &["infantry attack 33% faster (champions)"]),
    (
        "Jurchens",
        &[
            "mounted units & fire lancers attack 25% faster",
            "units take -50% friendly fire damage",
        ],
    ),
    ("Khitans", &["melee attack upgrades doubled (hard-hitting melee)"]),
    (
        "Khmer",
        &[
            "battle elephants move +10% faster",
            "team: scorpions +1 range (heavy scorps)",
        ],
    ),
    (
        "Koreans",
        &["ranged soldiers & infantry cost -50% wood (arbalests/HC/champs)"],
    ),
    ("Lithuanians", &["knight line +1 attack per relic held (max +4)"]),
    ("Magyars", &["scout line costs -15% (hussars cost only food)"]),
    ("Malay", &["battle elephants cost -35% in imperial"]),
    (
        "Malians",
        &["infantry +3 pierce armor in imperial", "buildings cost -15% wood"],
    ),
    (
        "Mapuche",
        &[
            "mounted units gain +3 gold per kill",
            "infantry +15 HP in imperial",
        ],
    ),
    (
        "Mayans",
        &["resources last +15% longer", "foot archers cost -30% in imperial"],
    ),
    (
        "Mongols",
        &[
            "with Drill researched: siege moves 50% faster (fast SO/heavy scorps)",
            "cavalry archers attack 25% faster",
        ],
    ),
    (
        "Muisca",
        &[
            "champions & arbalests get +3 melee armor in imperial",
            "team: natural gold lasts +15% longer",
        ],
    ),
    ("Persians", &["town centers & docks have +100% HP"]),
    ("Poles", &[]),
    (
        "Portuguese",
        &["all units cost -20% gold", "Feitorias trickle resources (wood relief)"],
    ),
    (
        "Romans",
        &[
            "infantry armor upgrades doubled (armored champions)",
            "scorpions cost -50% gold",
        ],
    ),
    (
        "Saracens",
        &["camels have +25% HP", "market trading fee only 5% (fast gold)"]),
    (
        "Saxons",
        &[
            "Shield Wall researched: infantry gain extra armor in large groups",
            "Clerical Recruitment: monks gain +1 range and train 33% faster",
            "towers & castles fire more arrows from the Castle Age",
        ],
    ),
    (
        "Shu",
        &[
            "siege weapons move +15% faster in imperial",
            "lumberjacks also generate food",
        ],
    ),
    (
        "Sicilians",
        &["land units take -40% bonus damage", "farms provide +125% food"]),
    (
        "Slavs",
        &["siege units cost -15%", "farmers work +15% faster"],
    ),
    (
        "Spanish",
        &[
            "gunpowder units attack 18% faster",
            "team: trade generates +25% gold",
        ],
    ),
    ("Tatars", &["units deal +25% damage from higher elevation"]),
    (
        "Teutons",
        &[
            "champions & cavalry get +2 melee armor in imperial",
            "towers garrison +5 (bombard towers)",
            "farms cost -40%",
        ],
    ),
    ("Tupi", &["fallen units refund 15% of their cost (army sustain)"]),
    (
        "Turks",
        &["gunpowder units have +25% HP", "scout line gets +1 pierce armor"],
    ),
    (
        "Varangians",
        &[
            "Vendel Legacy: knight line deals trample damage",
            "Gothikon: Varangian Guards periodically throw axes",
            "shepherding, fishing & hunting also generate gold",
        ],
    ),
    ("Vietnamese", &["archery range units have +20% HP (arbalests)"]),
    ("Vikings", &["infantry have +20% HP (champions)"]),
    ("Wei", &["Hei Guang cavalry & raiders gain +30% HP in imperial"]),
    (
        "Wu",
        &[
            "Jian swordsmen & Hei Guang cavalry +2 attack in imperial",
            "infantry regenerate 30 HP/min",
        ],
    ),
];

pub fn notes_for(name: &str) -> &'static [&'static str] {
    NOTES
        .iter()
        .find(|(civ, _)| *civ == name)
        .map_or(&[], |(_, notes)| *notes)
}

pub fn tech_tree_link(name: &str) -> String {
    format!("https://aoe2techtree.net/#{name}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{CIVS, OPTIONS};
    use std::collections::HashSet;

    fn civ(name: &str) -> &'static Civ {
        CIVS.iter().find(|c| c.name == name).expect("known civ")
    }

    fn row<'a>(summary: &'a CivSummary, label: &str) -> &'a Row {
        summary
            .rows
            .iter()
            .find(|row| row.label == label)
            .unwrap_or_else(|| panic!("no row {label}"))
    }

    fn status(summary: &CivSummary, label: &str) -> Status {
        row(summary, label).status.clone()
    }

    fn partial(names: &[&'static str]) -> Status {
        Status::Partial(names.to_vec())
    }

    fn catalog_keys() -> HashSet<FilterKey> {
        OPTIONS.iter().flat_map(|o| o.keys.iter().copied()).collect()
    }

    fn every_key(catalog: &HashSet<FilterKey>, slot: &Slot) {
        for key in slot
            .keys
            .iter()
            .chain(slot.fallback.map_or(&[] as &[FilterKey], |(_, k)| k))
        {
            assert!(catalog.contains(key), "{} key {key:?} not in catalog", slot.label);
        }
        for up in slot.fu {
            assert!(catalog.contains(&up.key), "{} upgrade {:?} not in catalog", slot.label, up);
        }
    }

    #[test]
    fn every_slot_key_exists_in_the_catalog() {
        let catalog = catalog_keys();
        for slot in SPAM_RULES.iter().chain(WOOD_RULES) {
            every_key(&catalog, slot);
        }
        every_key(&catalog, &CHAMPION_SLOT);
        every_key(&catalog, &ARBALEST_SLOT);
        every_key(&catalog, &CAVALIER_SLOT);
    }

    #[test]
    fn notes_exist_for_every_civ() {
        for civ in CIVS {
            assert!(
                NOTES.iter().any(|(name, _)| *name == civ.name),
                "{} missing from NOTES",
                civ.name
            );
            let _ = notes_for(civ.name);
        }
    }

    #[test]
    fn special_lists_only_contain_civs_that_hold_the_unit() {
        for name in SPECIAL_INF {
            assert!(
                !matches!(evaluate(civ(name), &CHAMPION_SLOT), Status::Absent),
                "{name} has no Champion but is in SPECIAL_INF"
            );
        }
        for name in SPECIAL_RANGE {
            assert!(
                !matches!(evaluate(civ(name), &ARBALEST_SLOT), Status::Absent),
                "{name} has no Arbalest but is in SPECIAL_RANGE"
            );
        }
    }

    #[test]
    fn special_list_civs_are_echoed_in_notes() {
        for name in SPECIAL_INF {
            let notes = notes_for(name);
            assert!(
                notes.iter().any(|n| n.contains("champion") || n.contains("infantry")),
                "{name} SPECIAL_INF note missing infantry/champion mention"
            );
        }
        for name in SPECIAL_RANGE {
            let notes = notes_for(name);
            assert!(
                notes.iter().any(|n| {
                    n.contains("arbalest") || n.contains("foot archer") || n.contains("ranged")
                }),
                "{name} SPECIAL_RANGE note missing arbalest mention"
            );
        }
    }

    #[test]
    fn never_emit_absent_rows() {
        for civ in CIVS {
            let summary = summary_for(civ);
            assert!(
                summary.rows.iter().all(|row| row.status != Status::Absent),
                "{} rows never contain Absent",
                civ.name
            );
        }
    }

    #[test]
    fn every_civ_gets_at_least_one_spam_row() {
        for civ in CIVS {
            let summary = summary_for(civ);
            assert!(
                summary
                    .rows
                    .iter()
                    .any(|row| row.section == Section::Spam),
                "{} has no spam row",
                civ.name
            );
        }
    }

    #[test]
    fn row_order_is_stable_across_sections() {
        let s = summary_for(civ("Turks"));
        let spam: Vec<&str> = s
            .rows
            .iter()
            .filter(|row| row.section == Section::Spam)
            .map(|row| row.label)
            .collect();
        assert_eq!(spam, vec!["Elite Janissary", "Heavy Camel Rider", "Hand Cannoneer"]);
        let tools: Vec<&str> = s
            .rows
            .iter()
            .filter(|row| row.section == Section::Wood)
            .map(|row| row.label)
            .collect();
        assert_eq!(
            tools,
            vec!["Siege Ram", "Heavy Scorpion", "Bombard Cannon", "Bombard Tower"]
        );
    }

    #[test]
    fn turks_have_spam_tools_but_no_champion_or_onager() {
        let s = summary_for(civ("Turks"));
        assert_eq!(status(&s, "Elite Janissary"), Status::Full);
        assert_eq!(status(&s, "Heavy Camel Rider"), Status::Full);
        assert_eq!(status(&s, "Hand Cannoneer"), Status::Full);
        assert_eq!(status(&s, "Bombard Tower"), Status::Full);
        assert_eq!(status(&s, "Heavy Scorpion"), Status::Full);
        assert_no_row(&s, "Champion");
        assert_no_row(&s, "Paladin");
        assert_no_row(&s, "Siege Onager");
    }

    #[test]
    fn koreans_arbalest_special_but_no_champion_or_heavy_camel() {
        let s = summary_for(civ("Koreans"));
        assert_eq!(status(&s, "Arbalester"), Status::Full);
        assert_eq!(status(&s, "Hand Cannoneer"), Status::Full);
        assert_eq!(status(&s, "Elite War Wagon"), Status::Full);
        assert_eq!(status(&s, "Heavy Rocket Cart"), Status::Full);
        assert_no_row(&s, "Champion");
        assert_no_row(&s, "Heavy Camel Rider");
        assert_no_row(&s, "Siege Onager");
        assert_no_row(&s, "Heavy Scorpion");
        assert_eq!(status(&s, "Bombard Tower"), Status::Full);
    }

    #[test]
    fn britons_arbalest_special_misses_thumb_ring() {
        let s = summary_for(civ("Britons"));
        assert_eq!(status(&s, "Arbalester"), partial(&["Thumb Ring"]));
        assert_eq!(status(&s, "Cavalier"), partial(&["Bloodlines"]));
        assert_eq!(status(&s, "Elite Longbowman"), Status::Full);
        assert_no_row(&s, "Champion");
    }

    #[test]
    fn franks_have_paladin_but_miss_bloodlines_and_bbt() {
        let s = summary_for(civ("Franks"));
        assert_eq!(status(&s, "Paladin"), partial(&["Bloodlines"]));
        assert_eq!(status(&s, "Elite Throwing Axeman"), Status::Full);
        assert_no_row(&s, "Cavalier");
        assert_no_row(&s, "Bombard Tower");
    }

#[test]
    fn goths_huskarl_champion_but_no_paladin() {
        let s = summary_for(civ("Goths"));
        assert_eq!(status(&s, "Elite Huskarl"), Status::Full);
        assert_eq!(
            status(&s, "Champion"),
            partial(&["Arson", "Plate Mail Armor"])
        );
        assert_eq!(status(&s, "Hand Cannoneer"), Status::Full);
        assert_no_row(&s, "Paladin");
    }

    #[test]
    fn celts_woad_raider_paladin_and_siege() {
        let s = summary_for(civ("Celts"));
        assert_eq!(status(&s, "Elite Woad Raider"), Status::Full);
        assert_eq!(
            status(&s, "Paladin"),
            partial(&["Bloodlines", "Plate Barding Armor"])
        );
        assert_eq!(status(&s, "Siege Onager"), Status::Full);
        assert_eq!(status(&s, "Heavy Scorpion"), Status::Full);
        assert_no_row(&s, "Bombard Tower");
    }

    #[test]
    fn mongols_elite_mangudai_camel_steppe_and_so() {
        let s = summary_for(civ("Mongols"));
        assert_eq!(status(&s, "Elite Mangudai"), Status::Full);
        assert_no_row(&s, "Mangudai");
        assert_eq!(status(&s, "Heavy Camel Rider"), partial(&["Plate Barding Armor"]));
        assert_eq!(status(&s, "Elite Steppe Lancer"), Status::Full);
        assert_eq!(status(&s, "Siege Onager"), Status::Full);
        assert_no_row(&s, "Paladin");
    }

    #[test]
    fn chinese_get_coverage_cavalier() {
        let s = summary_for(civ("Chinese"));
        assert_eq!(status(&s, "Cavalier"), Status::Full);
        assert_eq!(status(&s, "Elite Chu Ko Nu"), Status::Full);
        assert_eq!(status(&s, "Heavy Rocket Cart"), Status::Full);
        assert_no_row(&s, "Champion");
        assert_no_row(&s, "Arbalester");
    }

    #[test]
    fn dravidians_have_siege_elephant_but_no_siege_ram() {
        let s = summary_for(civ("Dravidians"));
        assert_eq!(status(&s, "Elite Elephant Archer"), Status::Full);
        assert_eq!(status(&s, "Siege Elephant"), Status::Full);
        assert_eq!(
            row(&s, "Siege Elephant").section,
            Section::Spam,
            "siege elephants are food+gold spam, not wood"
        );
        assert_eq!(status(&s, "Elite Urumi Swordsman"), Status::Full);
        assert_eq!(status(&s, "Battle Elephant"), partial(&["Plate Barding Armor"]));
    }

    #[test]
    fn siege_engineers_footnote_only_with_tools() {
        let s = summary_for(civ("Turks"));
        assert!(s.rows.iter().any(|row| row.section == Section::Wood));
        assert_eq!(s.siege_engineers, civ("Turks").keys.contains(&tk(377)));
    }

    #[test]
    fn every_row_carries_a_real_icon() {
        for civ in CIVS {
            for row in summary_for(civ).rows {
                assert_ne!(
                    row.icon, "missing",
                    "{} {row:?} resolved no icon",
                    civ.name
                );
            }
        }
    }

    #[test]
    fn naval_unique_units_are_excluded() {
        let naval_labels: HashSet<&str> = OPTIONS
            .iter()
            .filter(|option| option.naval)
            .map(|option| option.label)
            .collect();
        assert!(
            !naval_labels.contains("Mangudai"),
            "sanity: land unique shouldn't be flagged naval"
        );
        for civ in CIVS {
            for row in summary_for(civ).rows {
                assert!(
                    !naval_labels.contains(row.label),
                    "{} shows naval unit {}",
                    civ.name,
                    row.label
                );
            }
        }
    }

    #[test]
    fn base_lines_show_the_base_units_icon() {
        let britons = summary_for(civ("Britons"));
        assert_eq!(
            row(&britons, "Siege Ram").icon,
            "Unit/63",
            "no siege ram -> capped ram icon"
        );
        assert_eq!(
            row(&britons, "Siege Onager").icon,
            "Unit/101",
            "no siege onager -> onager icon"
        );
    }

    #[test]
    fn notes_cover_highlighted_civs() {
        assert!(notes_for("Turks").iter().any(|n| n.contains("+25% HP")));
        assert!(notes_for("Koreans").iter().any(|n| n.contains("-50% wood")));
        assert!(notes_for("Celts").iter().any(|n| n.contains("25% faster")));
        assert!(notes_for("Vikings").iter().any(|n| n.contains("+20% HP")));
        assert!(notes_for("Portuguese").iter().any(|n| n.contains("-20% gold")));
        assert!(notes_for("Hindustanis").iter().any(|n| n.contains("camel")));
    }

    #[test]
    fn hindustanis_imperial_camel_replaces_heavy_camel() {
        let s = summary_for(civ("Hindustanis"));
        assert!(s.rows.iter().any(|row| row.label.contains("Imperial Camel Rider")));
        assert_no_row(&s, "Heavy Camel Rider");
    }

    #[test]
    fn bohemians_houfnice_replaces_bombard_cannon() {
        let s = summary_for(civ("Bohemians"));
        assert!(s.rows.iter().any(|row| row.label == "Houfnice"));
        assert_no_row(&s, "Bombard Cannon");
    }

    #[test]
    fn sicilians_list_cavalier() {
        let s = summary_for(civ("Sicilians"));
        assert_eq!(status(&s, "Cavalier"), Status::Full);
    }

    #[test]
    fn armenians_list_cavalier_and_infantry_note() {
        let s = summary_for(civ("Armenians"));
        assert!(s.rows.iter().any(|row| row.label == "Cavalier"));
        assert_no_row(&s, "Champion");
        assert!(
            notes_for("Armenians").iter().any(|n| n.contains("+30 HP")),
            "Armenian +30 HP bonus should be noted"
        );
    }

    #[test]
    fn stableless_civs_get_champion() {
        let aztecs = summary_for(civ("Aztecs"));
        assert!(
            aztecs.rows.iter().any(|row| row.label == "Champion"),
            "Aztecs have no stable line so the guaranteed Champion should list"
        );
        let incas = summary_for(civ("Incas"));
        assert_no_row(&incas, "Champion");
    }

    #[test]
    fn varangian_guard_lists_for_every_holder() {
        for name in ["Byzantines", "Danes", "Saxons", "Varangians", "Vikings"] {
            let s = summary_for(civ(name));
            assert!(
                s.rows.iter().any(|row| row.label == "Elite Varangian Guard"),
                "{name} should list the Elite Varangian Guard"
            );
        }
        let franks = summary_for(civ("Franks"));
        assert_no_row(&franks, "Varangian Guard");
        assert_no_row(&franks, "Elite Varangian Guard");
    }

    fn assert_no_row(summary: &CivSummary, label: &str) {
        assert!(
            !summary.rows.iter().any(|row| row.label == label),
            "unexpected row {label}"
        );
    }
}