use crate::components::dnd;
use crate::components::pill::Pill as PillChip;
use crate::model::{ExtractPill, MovePills, Row};
use leptos::html;
use leptos::prelude::*;
use sortable_js::Sortable;
use wasm_bindgen::JsCast;

#[component]
pub fn FilterRow(
    rows: RwSignal<Vec<Row>>,
    row_id: u64,
    on_remove_pill: Callback<u64>,
    on_move: Callback<MovePills>,
    on_extract: Callback<ExtractPill>,
) -> impl IntoView {
    let pills = Memo::new(move |_| {
        rows.with(|all| {
            all.iter()
                .find(|row| row.id == row_id)
                .map(|row| row.pills.clone())
                .unwrap_or_default()
        })
    });

    let row_ref = NodeRef::<html::Div>::new();
    let sortable = StoredValue::<Option<Sortable>, LocalStorage>::new_local(None);

    row_ref.on_load(move |el| {
        let el: web_sys::HtmlElement = el.unchecked_into();
        sortable.set_value(Some(dnd::apply(&el, on_move, on_extract)));
    });

    view! {
        <div class="filter-row" node_ref=row_ref data-row-id=row_id>
            <For
                each=move || pills.get()
                key=|pill| pill.id
                children=move |pill| {
                    view! {
                        <PillChip
                            name=pill.name.clone()
                            pill_id=pill.id
                            on_remove=move |()| on_remove_pill.run(pill.id)
                        />
                    }
                }
            />
        </div>
    }
}