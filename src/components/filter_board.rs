use crate::components::civ_panel::CivPanel;
use crate::components::combobox::Combobox;
use crate::components::filter_row::FilterRow;
use crate::data::{CivOption, CIVS, OPTIONS};
use crate::model::{
    apply_move, extract_to_new_row, matching_civs, next_id, Civ, ExtractPill, MovePills, Pill, Row,
};
use crate::storage::{load_bool, save_bool};
use leptos::prelude::*;

const SHOW_UNIQUE_UNITS: &str = "aoe2.showUniqueUnits";
const SHOW_UNIQUE_TECHS: &str = "aoe2.showUniqueTechs";
const THEME_KEY: &str = "aoe2.darkMode";

fn system_prefers_dark() -> bool {
    web_sys::window()
        .and_then(|win| win.match_media("(prefers-color-scheme: dark)").ok().flatten())
        .map(|mq| mq.matches())
        .unwrap_or(false)
}

#[component]
pub fn FilterBoard(rows: RwSignal<Vec<Row>>) -> impl IntoView {
    let open_combobox = RwSignal::new(false);

    let show_unique_units = RwSignal::new(load_bool(SHOW_UNIQUE_UNITS, false));
    let show_unique_techs = RwSignal::new(load_bool(SHOW_UNIQUE_TECHS, false));
    Effect::new(move |_| save_bool(SHOW_UNIQUE_UNITS, show_unique_units.get()));
    Effect::new(move |_| save_bool(SHOW_UNIQUE_TECHS, show_unique_techs.get()));

    let dark = RwSignal::new(load_bool(THEME_KEY, system_prefers_dark()));
    Effect::new(move |_| {
        if let Some(html) = document().document_element() {
            if dark.get() {
                let _ = html.set_attribute("data-theme", "dark");
            } else {
                let _ = html.remove_attribute("data-theme");
            }
        }
    });
    Effect::new(move |_| save_bool(THEME_KEY, dark.get()));

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

    let selected = RwSignal::new(None::<Civ>);

    let is_selected = move |civ_name: &str| selected.get().is_some_and(|c| c.name == civ_name);

    let toggle = move |civ: Civ| {
        if is_selected(civ.name) {
            selected.set(None);
        } else {
            selected.set(Some(civ));
        }
    };

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
                <div class="board-annotation-text">
                    <h1>"AoE2 civ filter"</h1>
                    <p>
                        "Add a unit or tech. Drag pills to combine them into one filter, or out to split them."
                    </p>
                </div>
                <button
                    class="theme-toggle"
                    type="button"
                    aria-pressed=move || dark.get()
                    aria-label="Toggle dark mode"
                    title=move || if dark.get() { "Switch to light mode" } else { "Switch to dark mode" }
                    on:click=move |_| dark.update(|v| *v = !*v)
                >
                    <Show
                        when=move || dark.get()
                        fallback=move || {
                            view! {
                                <svg viewBox="0 0 24 24">
                                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                                </svg>
                            }
                        }
                    >
                        <svg viewBox="0 0 24 24">
                            <circle cx="12" cy="12" r="5"></circle>
                            <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"></path>
                        </svg>
                    </Show>
                </button>
            </header>

            <div class="rows">
                <For
                    each=move || rows.get()
                    key=|row| row.id
                    children=move |row| {
                        view! {
                            <div class="row-join" aria-hidden="true">"AND"</div>
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
                    on_cancel=Callback::new(move |()| open_combobox.set(false))
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
                        children=move |civ| {
                            let active = is_selected(civ.name);
                            view! {
                                <li>
                                    <button
                                        class=move || {
                                            if is_selected(civ.name) {
                                                "civ civ-active"
                                            } else {
                                                "civ"
                                            }
                                        }
                                        aria-pressed=active
                                        on:click=move |_| toggle(*civ)
                                    >
                                        {civ.name}
                                    </button>
                                </li>
                            }
                        }
                    />
                </ul>
            </section>

            <CivPanel selected=selected />
        </div>
    }
}
