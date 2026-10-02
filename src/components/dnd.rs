use crate::model::{ExtractPill, MovePills};
use js_sys::Reflect;
use leptos::prelude::*;
use sortable_js::{Event, Options, Sortable};
use wasm_bindgen::JsValue;

pub const ROW_ATTR: &str = "data-row-id";
const PILL_SELECTOR: &str = ".pill";
const PILL_FILTER: &str = ".pill-remove";

pub fn apply(
    elt: &web_sys::Element,
    on_move: Callback<MovePills>,
    on_extract: Callback<ExtractPill>,
) -> Sortable {
    let mut opts = Options::new();
    opts.group("filters")
        .animation_ms(150.)
        .draggable(PILL_SELECTOR)
        .filter(PILL_FILTER)
        .ghost_class("sortable-ghost")
        .chosen_class("sortable-chosen")
        .drag_class("sortable-drag")
        .on_end(move |evt: Event| {
            // Draggable-only indices, not raw child indices: the row also contains
            // static "or" joiner spans, which SortableJS would otherwise count.
            let (Some(old_index), Some(new_index)) =
                (evt.old_draggable_index, evt.new_draggable_index)
            else {
                return;
            };
            let (Some(from_row), Some(to_row)) = (attr_id(&evt.from), attr_id(&evt.to)) else {
                return;
            };

            if dropped_outside(&evt) {
                let extract = ExtractPill {
                    from_row,
                    old_index,
                };
                let moved = evt.item.clone();
                request_animation_frame(move || {
                    on_extract.run(extract);
                    request_animation_frame(move || {
                        if moved.is_connected() {
                            moved.remove();
                        }
                    });
                });
                return;
            }

            let moved = evt.item.clone();
            let cross_row = from_row != to_row;
            request_animation_frame(move || {
                on_move.run(MovePills {
                    from_row,
                    old_index,
                    to_row,
                    new_index,
                });
                if cross_row {
                    request_animation_frame(move || {
                        if moved.is_connected() {
                            moved.remove();
                        }
                    });
                }
            });
        });
    opts.apply(elt)
}

fn dropped_outside(evt: &Event) -> bool {
    let Some((x, y)) = pointer_coords(evt) else {
        return false;
    };
    let Some(mut el) = document().element_from_point(x as f32, y as f32) else {
        return false;
    };
    loop {
        if el.has_attribute(ROW_ATTR) {
            return false;
        }
        match el.parent_element() {
            Some(parent) => el = parent,
            None => return true,
        }
    }
}

fn pointer_coords(evt: &Event) -> Option<(f64, f64)> {
    let original = Reflect::get(&evt.raw_event, &JsValue::from_str("originalEvent")).ok()?;
    if original.is_null() || original.is_undefined() {
        return None;
    }
    let x = Reflect::get(&original, &JsValue::from_str("clientX"))
        .ok()
        .and_then(|v| v.as_f64())?;
    let y = Reflect::get(&original, &JsValue::from_str("clientY"))
        .ok()
        .and_then(|v| v.as_f64())?;
    Some((x, y))
}

fn attr_id(el: &web_sys::HtmlElement) -> Option<u64> {
    el.get_attribute(ROW_ATTR)?.parse().ok()
}
