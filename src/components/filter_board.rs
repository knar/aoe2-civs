use crate::components::combobox::Combobox;
use crate::components::filter_row::FilterRow;
use crate::data::{CivOption, CIVS, OPTIONS};
use crate::model::{
    apply_move, extract_to_new_row, matching_civs, next_id, ExtractPill, MovePills, Pill, Row,
};
use crate::storage::{load_bool, save_bool};
use leptos::prelude::*;

const SHOW_UNIQUE_UNITS: &str = "aoe2.showUniqueUnits";
const SHOW_UNIQUE_TECHS: &str = "aoe2.showUniqueTechs";

#[component]
pub fn FilterBoard(rows: RwSignal<Vec<Row>>) -> impl IntoView {
    let open_combobox = RwSignal::new(false);

    let show_unique_units = RwSignal::new(load_bool(SHOW_UNIQUE_UNITS, false));
    let show_unique_techs = RwSignal::new(load_bool(SHOW_UNIQUE_TECHS, false));
    Effect::new(move |_| save_bool(SHOW_UNIQUE_UNITS, show_unique_units.get()));
    Effect::new(move |_| save_bool(SHOW_UNIQUE_TECHS, show_unique_techs.get()));

    let on_new_filter = move |_| open_combobox.set(true);

    let on_pick = move |opt: CivOption| {
        rows.update(|all| {
            let already_picked = all
                .iter()
                .flat_map(|row| row.pills.iter())
                .any(|pill| pill.keys == opt.keys);
            if already_picked {
                return;
            }
            all.push(Row {
                id: next_id(),
                pills: vec![Pill {
                    id: next_id(),
                    keys: opt.keys,
                    name: opt.label.to_string(),
                }],
            });
        });
        open_combobox.set(false);
    };

    let options = OPTIONS.to_vec();
    let matches = Memo::new(move |_| matching_civs(&rows.get(), CIVS));

    let on_remove_pill = Callback::new(move |id: u64| {
        rows.update(|all| {
            if let Some(pos) = all
                .iter()
                .position(|row| row.pills.iter().any(|pill| pill.id == id))
            {
                all[pos].pills.retain(|pill| pill.id != id);
                if all[pos].pills.is_empty() {
                    all.remove(pos);
                }
            }
        });
    });

    let on_move = Callback::new(move |m: MovePills| {
        rows.update(|all| apply_move(all, m));
    });

    let on_extract = Callback::new(move |e: ExtractPill| {
        rows.update(|all| {
            extract_to_new_row(all, e);
        });
    });

    view! {
        <div class="filter-board">
            <header class="board-annotation">
                <h1>"AoE2 civ filter"</h1>
                <p>"Every row is required (AND). Pick a unit or tech per row."</p>
            </header>

            <div class="rows">
                <For
                    each=move || rows.get()
                    key=|row| row.id
                    children=move |row| {
                        view! {
                            <FilterRow
                                rows=rows
                                row_id=row.id
                                on_remove_pill=on_remove_pill
                                on_move=on_move
                                on_extract=on_extract
                            />
                        }
                    }
                />
            </div>

            <Show when=move || open_combobox.get()>
                <Combobox
                    options=options.clone()
                    on_pick=on_pick
                    show_unique_units=show_unique_units
                    show_unique_techs=show_unique_techs
                />
            </Show>

            <button
                class="new-filter"
                disabled=move || open_combobox.get()
                on:click=on_new_filter
            >
                "+ New filter"
            </button>

            <section class="results">
                <h2 class="results-count">
                    {move || {
                        let n = matches.get().len();
                        if n == 1 {
                            "1 civ matches".to_string()
                        } else {
                            format!("{n} civs match")
                        }
                    }}
                </h2>
                <ul class="civ-list">
                    <For
                        each=move || matches.get()
                        key=|civ| civ.name
                        children=|civ| view! { <li class="civ">{civ.name}</li> }
                    />
                </ul>
            </section>
        </div>
    }
}
