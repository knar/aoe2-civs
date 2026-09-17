use leptos::prelude::*;

#[component]
pub fn Pill(
    name: String,
    pill_id: u64,
    #[prop(into)] on_remove: Callback<()>,
) -> impl IntoView {
    let label = name.clone();
    view! {
        <span class="pill" data-pill-id=pill_id>
            {name}
            <button
                class="pill-remove"
                type="button"
                aria-label=label
                on:click=move |_| on_remove.run(())
            >
                "✕"
            </button>
        </span>
    }
}