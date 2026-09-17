use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Group {
    Unit,
    Tech,
}

/// Identity of one dataset entry. The unit and tech ID spaces overlap, and a
/// single entity can span several IDs (different building slots / civ variants),
/// so an option is identified by its label and carries a set of these keys.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FilterKey {
    pub group: Group,
    pub data_id: u32,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Pill {
    pub id: u64,
    pub keys: &'static [FilterKey],
    pub name: String,
}

/// A civ and the units/techs it has, as filter keys.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Civ {
    pub name: &'static str,
    pub keys: &'static [FilterKey],
}

#[derive(Clone, PartialEq)]
pub struct Row {
    pub id: u64,
    pub pills: Vec<Pill>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MovePills {
    pub from_row: u64,
    pub old_index: usize,
    pub to_row: u64,
    pub new_index: usize,
}

pub fn apply_move(rows: &mut Vec<Row>, m: MovePills) {
    let Some(from_pos) = rows.iter().position(|row| row.id == m.from_row) else {
        return;
    };
    let Some(pill) = rows[from_pos].pills.get(m.old_index).cloned() else {
        return;
    };
    rows[from_pos].pills.remove(m.old_index);
    let empty_from = rows[from_pos].pills.is_empty();

    if m.from_row == m.to_row {
        let target = m.new_index.min(rows[from_pos].pills.len());
        rows[from_pos].pills.insert(target, pill);
    } else if let Some(to_pos) = rows.iter().position(|row| row.id == m.to_row) {
        let target = m.new_index.min(rows[to_pos].pills.len());
        rows[to_pos].pills.insert(target, pill);
    } else {
        rows[from_pos].pills.insert(m.old_index, pill);
    }

    if empty_from && m.from_row != m.to_row {
        rows.retain(|row| row.id != m.from_row);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ExtractPill {
    pub from_row: u64,
    pub old_index: usize,
}

pub fn extract_to_new_row(rows: &mut Vec<Row>, e: ExtractPill) -> Option<u64> {
    let from_pos = rows.iter().position(|row| row.id == e.from_row)?;
    let pill = rows[from_pos].pills.get(e.old_index).cloned()?;
    rows[from_pos].pills.remove(e.old_index);

    let new_id = next_id();
    let new_row = Row {
        id: new_id,
        pills: vec![pill],
    };
    if rows[from_pos].pills.is_empty() {
        rows[from_pos] = new_row;
    } else {
        rows.insert(from_pos + 1, new_row);
    }
    Some(new_id)
}

/// Civs matching the filter: AND over rows, OR within each row. An empty filter
/// matches every civ.
pub fn matching_civs<'a>(rows: &[Row], civs: &'a [Civ]) -> Vec<&'a Civ> {
    civs.iter()
        .filter(|civ| {
            rows.iter().all(|row| {
                row.pills
                    .iter()
                    .any(|pill| pill.keys.iter().any(|key| civ.keys.contains(key)))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(items: Vec<FilterKey>) -> &'static [FilterKey] {
        Box::leak(items.into_boxed_slice())
    }

    fn pill(id: u64, name: &str) -> Pill {
        Pill {
            id,
            keys: keys(vec![FilterKey {
                group: Group::Unit,
                data_id: id as u32,
            }]),
            name: name.to_string(),
        }
    }

    fn row(id: u64, pills: Vec<Pill>) -> Row {
        Row { id, pills }
    }

    #[test]
    fn same_row_reorder_forward() {
        let mut rows = vec![row(1, vec![pill(10, "a"), pill(11, "b"), pill(12, "c")])];
        apply_move(
            &mut rows,
            MovePills {
                from_row: 1,
                old_index: 0,
                to_row: 1,
                new_index: 2,
            },
        );
        assert_eq!(
            rows[0].pills.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![11, 12, 10]
        );
    }

    #[test]
    fn same_row_reorder_backward() {
        let mut rows = vec![row(1, vec![pill(10, "a"), pill(11, "b"), pill(12, "c")])];
        apply_move(
            &mut rows,
            MovePills {
                from_row: 1,
                old_index: 2,
                to_row: 1,
                new_index: 0,
            },
        );
        assert_eq!(
            rows[0].pills.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![12, 10, 11]
        );
    }

    #[test]
    fn cross_row_move_inserts_and_keeps_source() {
        let mut rows = vec![
            row(1, vec![pill(10, "a"), pill(11, "b")]),
            row(2, vec![pill(12, "c")]),
        ];
        apply_move(
            &mut rows,
            MovePills {
                from_row: 1,
                old_index: 0,
                to_row: 2,
                new_index: 1,
            },
        );
        assert_eq!(
            rows[0].pills.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![11]
        );
        assert_eq!(
            rows[1].pills.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![12, 10]
        );
    }

    #[test]
    fn cross_row_move_removes_emptied_source_row() {
        let mut rows = vec![row(1, vec![pill(10, "a")]), row(2, vec![pill(12, "c")])];
        apply_move(
            &mut rows,
            MovePills {
                from_row: 1,
                old_index: 0,
                to_row: 2,
                new_index: 1,
            },
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, 2);
        assert_eq!(
            rows[0].pills.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![12, 10]
        );
    }

    #[test]
    fn cross_row_move_to_end_clamps() {
        let mut rows = vec![row(1, vec![pill(10, "a")]), row(2, vec![pill(12, "c")])];
        apply_move(
            &mut rows,
            MovePills {
                from_row: 1,
                old_index: 0,
                to_row: 2,
                new_index: 5,
            },
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].pills.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![12, 10]
        );
    }

    #[test]
    fn extract_splits_a_two_pill_row_into_two_rows() {
        let mut rows = vec![row(1, vec![pill(10, "a"), pill(11, "b")])];
        let new_id = extract_to_new_row(
            &mut rows,
            ExtractPill {
                from_row: 1,
                old_index: 1,
            },
        )
        .expect("extract should create a row");
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].pills.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![10]
        );
        assert_eq!(rows[1].id, new_id);
        assert_eq!(
            rows[1].pills.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![11]
        );
    }

    #[test]
    fn extract_from_first_index_keeps_order() {
        let mut rows = vec![row(1, vec![pill(10, "a"), pill(11, "b")])];
        extract_to_new_row(
            &mut rows,
            ExtractPill {
                from_row: 1,
                old_index: 0,
            },
        );
        assert_eq!(
            rows[0].pills.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![11]
        );
        assert_eq!(
            rows[1].pills.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![10]
        );
    }

    #[test]
    fn extract_empty_source_row_is_replaced_in_place() {
        let mut rows = vec![row(1, vec![pill(10, "a")]), row(2, vec![pill(12, "c")])];
        let new_id = extract_to_new_row(
            &mut rows,
            ExtractPill {
                from_row: 1,
                old_index: 0,
            },
        )
        .expect("extract should create a row");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, new_id);
        assert_eq!(
            rows[0].pills.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![10]
        );
        assert_eq!(rows[1].id, 2);
    }

    const ARCHER: FilterKey = FilterKey {
        group: Group::Unit,
        data_id: 4,
    };
    const SKIRM: FilterKey = FilterKey {
        group: Group::Unit,
        data_id: 7,
    };
    const LOOM: FilterKey = FilterKey {
        group: Group::Tech,
        data_id: 22,
    };

    fn civ_fixture() -> &'static [Civ] {
        static CIVS: &[Civ] = &[
            Civ {
                name: "A",
                keys: &[ARCHER, LOOM],
            },
            Civ {
                name: "B",
                keys: &[SKIRM],
            },
            Civ {
                name: "C",
                keys: &[ARCHER, SKIRM],
            },
        ];
        CIVS
    }

    fn keyed_row(id: u64, item_keys: &[FilterKey]) -> Row {
        Row {
            id,
            pills: item_keys
                .iter()
                .map(|&key| Pill {
                    id: next_id(),
                    keys: keys(vec![key]),
                    name: String::new(),
                })
                .collect(),
        }
    }

    fn names<'a>(civs: &[&'a Civ]) -> Vec<&'a str> {
        civs.iter().map(|c| c.name).collect()
    }

    #[test]
    fn empty_filter_matches_all_civs() {
        let matched = matching_civs(&[], civ_fixture());
        assert_eq!(names(&matched), vec!["A", "B", "C"]);
    }

    #[test]
    fn single_pill_matches_only_civs_that_have_it() {
        let rows = [keyed_row(1, &[LOOM])];
        let matched = matching_civs(&rows, civ_fixture());
        assert_eq!(names(&matched), vec!["A"]);
    }

    #[test]
    fn row_is_or_across_pills() {
        let rows = [keyed_row(1, &[ARCHER, SKIRM])];
        let matched = matching_civs(&rows, civ_fixture());
        assert_eq!(names(&matched), vec!["A", "B", "C"]);
    }

    #[test]
    fn rows_are_and() {
        let rows = [keyed_row(1, &[ARCHER]), keyed_row(2, &[SKIRM])];
        let matched = matching_civs(&rows, civ_fixture());
        assert_eq!(names(&matched), vec!["C"]);
    }

    #[test]
    fn no_civ_matches_contradictory_rows() {
        let rows = [keyed_row(1, &[LOOM]), keyed_row(2, &[SKIRM])];
        let matched = matching_civs(&rows, civ_fixture());
        assert!(matched.is_empty());
    }

    #[test]
    fn unit_and_tech_ids_do_not_collide() {
        let tech = FilterKey {
            group: Group::Tech,
            data_id: 4,
        };
        assert_ne!(ARCHER, tech);
        let rows = [keyed_row(1, &[tech])];
        assert!(matching_civs(&rows, civ_fixture()).is_empty());
    }

    #[test]
    fn pill_matches_civ_with_any_of_its_keys() {
        const GENERIC: FilterKey = FilterKey {
            group: Group::Unit,
            data_id: 358,
        };
        const VARIANT: FilterKey = FilterKey {
            group: Group::Unit,
            data_id: 1787,
        };
        static CIVS: &[Civ] = &[
            Civ {
                name: "Generic",
                keys: &[GENERIC],
            },
            Civ {
                name: "Variant",
                keys: &[VARIANT],
            },
            Civ {
                name: "Neither",
                keys: &[],
            },
        ];
        let row = Row {
            id: 1,
            pills: vec![Pill {
                id: next_id(),
                keys: keys(vec![GENERIC, VARIANT]),
                name: "Pikeman".to_string(),
            }],
        };
        let matched = matching_civs(&[row], CIVS);
        assert_eq!(names(&matched), vec!["Generic", "Variant"]);
    }
}