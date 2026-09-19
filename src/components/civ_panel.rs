use crate::model::Civ;
use crate::overview::{summary_for, tech_tree_link, Row, Section, Status};
use leptos::prelude::*;

const ICON_BASE: &str = "https://aoe2techtree.net/img/";

#[component]
pub fn CivPanel(selected: RwSignal<Option<Civ>>) -> impl IntoView {
    let is_open = move || selected.get().is_some();
    let close = move |_| selected.set(None);

    let panel_class = move || {
        if is_open() {
            "civ-panel open"
        } else {
            "civ-panel"
        }
    };

    let backdrop = move || {
        if is_open() {
            "civ-panel-backdrop open"
        } else {
            "civ-panel-backdrop"
        }
    };

    view! {
        <>
            <div class=backdrop on:click=close aria-hidden="true"></div>
            <aside class=panel_class>
                <Show when=is_open>
                    {move || {
                        let civ = selected.get().expect("drawer only renders while open");
                        let sum = summary_for(&civ);
                        let name = sum.name;
                        let notes = sum.notes;
                        let se = sum.siege_engineers;
                        let spam = rows_in(&sum, Section::Spam);
                        let wood = rows_in(&sum, Section::Wood);
                        let spam_empty = spam.is_empty();
                        let wood_empty = wood.is_empty();
                        let link = tech_tree_link(name);
                        view! {
                            <header class="civ-panel-head">
                                <h2>{name}</h2>
                                <span class="civ-panel-tag">"DM michi"</span>
                                <button class="civ-panel-close" on:click=close aria-label="Close">
                                    "✕"
                                </button>
                            </header>

                            <section class="civ-panel-section">
                                <h3>{Section::Spam.title()}</h3>
                                <Show when=move || spam_empty>
                                    <p class="civ-panel-none">"no food & gold spam options"</p>
                                </Show>
                                <ul class="civ-panel-rows">
                                    <For
                                        each=move || spam.clone()
                                        key=|row| row.label
                                        children=|row: Row| row_element(row)
                                    />
                                </ul>
                            </section>

                            <section class="civ-panel-section">
                                <h3>{Section::Wood.title()}</h3>
                                <Show when=move || wood_empty>
                                    <p class="civ-panel-none">"no wood units worth fielding"</p>
                                </Show>
                                <ul class="civ-panel-rows">
                                    <For
                                        each=move || wood.clone()
                                        key=|row| row.label
                                        children=|row: Row| row_element(row)
                                    />
                                </ul>
                                <Show when=move || se>
                                    <p class="civ-panel-footnote">
                                        "Siege Engineers researched"
                                    </p>
                                </Show>
                            </section>

                            <section class="civ-panel-notes">
                                <h3>"Bonuses that matter"</h3>
                                <Show
                                    when=move || !notes.is_empty()
                                    fallback=move || {
                                        view! { <p class="civ-panel-none">"nothing notable for the spam game"</p> }
                                    }
                                >
                                    <ul>
                                        <For
                                            each=move || notes.to_vec()
                                            key=|note| *note
                                            children=|note: &'static str| view! { <li>{note}</li> }
                                        />
                                    </ul>
                                </Show>
                            </section>

                            <footer class="civ-panel-foot">
                                <a
                                    href=link
                                    target="_blank"
                                    rel="noopener noreferrer"
                                >
                                    "Full tech tree ↗"
                                </a>
                            </footer>
                        }
                    }}
                </Show>
            </aside>
        </>
    }
}

fn rows_in(summary: &crate::overview::CivSummary, section: Section) -> Vec<Row> {
    summary
        .rows
        .iter()
        .filter(|row| row.section == section)
        .cloned()
        .collect()
}

fn row_element(row: Row) -> impl IntoView {
    let src = format!("{ICON_BASE}{}.png", row.icon);
    let class = match &row.status {
        Status::Full => "civ-panel-icon s-full",
        Status::Partial(_) => "civ-panel-icon s-part",
        Status::Base(_) => "civ-panel-icon s-base",
        Status::Absent => unreachable!("absent rows are never emitted into the panel"),
    };
    let title = match &row.status {
        Status::Full => row.label.to_string(),
        Status::Partial(missing) => format!("{} — missing: {}", row.label, missing.join(", ")),
        Status::Base(label) => label.to_string(),
        Status::Absent => unreachable!("absent rows are never emitted into the panel"),
    };
    view! {
        <li class="civ-panel-row">
            <img
                class=class
                src=src
                alt=row.label
                title=title
                loading="lazy"
                width="48"
                height="48"
                onerror="this.src='https://aoe2techtree.net/img/missing.png'"
            />
        </li>
    }
}