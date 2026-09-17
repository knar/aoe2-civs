use crate::data::CivOption;
use leptos::{ev, prelude::*};
use wasm_bindgen::JsCast;

const INPUT_ID: &str = "combobox-input";

#[component]
pub fn Combobox(
    options: Vec<CivOption>,
    #[prop(into)] on_pick: Callback<CivOption>,
) -> impl IntoView {
    let options = StoredValue::new(options);
    let query = RwSignal::new(String::new());
    let open = RwSignal::new(true);
    let highlight = RwSignal::new(0usize);

    let filtered = Memo::new(move |_| {
        let q = query.get().trim().to_lowercase();
        options.with_value(|opts| {
            if q.is_empty() {
                opts.clone()
            } else {
                opts.iter()
                    .filter(|o| o.label.to_lowercase().contains(&q))
                    .copied()
                    .collect()
            }
        })
    });

    Effect::new(move |_| {
        if open.get() {
            if let Some(el) = document().get_element_by_id(INPUT_ID) {
                if let Some(input) = el.dyn_ref::<web_sys::HtmlInputElement>() {
                    let _ = input.focus();
                }
            }
        }
    });

    Effect::new(move |_| {
        if open.get() {
            if let Some(el) = document()
                .get_element_by_id(&format!("combobox-item-{}", highlight.get()))
            {
                let options = web_sys::ScrollIntoViewOptions::new();
                options.set_block(web_sys::ScrollLogicalPosition::Nearest);
                el.scroll_into_view_with_scroll_into_view_options(&options);
            }
        }
    });

    let on_input = move |ev: ev::Event| {
        query.set(event_target_value(&ev));
        highlight.set(0);
        open.set(true);
    };

    let on_keydown = move |ev: ev::KeyboardEvent| {
        let list = filtered.get();
        if list.is_empty() {
            if ev.key() == "Escape" {
                open.set(false);
            }
            return;
        }
        match ev.key().as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                let next = highlight.get().saturating_add(1).min(list.len() - 1);
                highlight.set(next);
            }
            "ArrowUp" => {
                ev.prevent_default();
                let prev = highlight.get().saturating_sub(1);
                highlight.set(prev);
            }
            "Enter" => {
                ev.prevent_default();
                if let Some(opt) = list.get(highlight.get()) {
                    on_pick.run(*opt);
                }
            }
            "Escape" => open.set(false),
            _ => {}
        }
    };

    view! {
        <div class="combobox">
            <input
                id=INPUT_ID
                class="combobox-input"
                type="text"
                placeholder="Pick a unit or tech"
                prop:value=move || query.get()
                on:input=on_input
                on:keydown=on_keydown
            />
            <Show when=move || open.get() && !filtered.get().is_empty()>
                <ul class="combobox-list">
                    <For
                        each=move || {
                            filtered.get().into_iter().enumerate().collect::<Vec<_>>()
                        }
                        key=|(_, opt)| *opt
                        children=move |(i, opt)| {
                            view! {
                                <li
                                    class="combobox-item"
                                    id=format!("combobox-item-{}", i)
                                    data-highlighted=move || i == highlight.get()
                                    on:mouseenter=move |_| highlight.set(i)
                                    on:click=move |_| on_pick.run(opt)
                                >
                                    {opt.label}
                                </li>
                            }
                        }
                    />
                </ul>
            </Show>
        </div>
    }
}
