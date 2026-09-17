use crate::components::combobox::Combobox;
use crate::components::filter_row::FilterRow;
use crate::data::OPTIONS;
use crate::model::{apply_move, extract_to_new_row, next_id, ExtractPill, MovePills, Pill, Row};
use leptos::prelude::*;

#[component]
pub fn FilterBoard(rows: RwSignal<Vec<Row>>) -> impl IntoView {
    let open_combobox = RwSignal::new(false);

    let on_new_filter = move |_| open_combobox.set(true);

    let on_pick = move |name: String| {
        rows.update(|all| {
            if all
                .iter()
                .any(|row| row.pills.iter().any(|pill| pill.name == name))
            {
                return;
            }
            all.push(Row {
                id: next_id(),
                pills: vec![Pill {
                    id: next_id(),
                    name,
                }],
            });
        });
        open_combobox.set(false);
    };

    let options = OPTIONS.iter().map(|s| s.to_string()).collect::<Vec<_>>();

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
                <Combobox options=options.clone() on_pick=on_pick />
            </Show>

            <button
                class="new-filter"
                disabled=move || open_combobox.get()
                on:click=on_new_filter
            >
                "+ New filter"
            </button>
        </div>
    }
}