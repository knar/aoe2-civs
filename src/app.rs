use crate::components::filter_board::FilterBoard;
use crate::model::Row;
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let rows = RwSignal::new(Vec::<Row>::new());
    view! { <FilterBoard rows /> }
}