# aoe2-civs — Planning document

A small client-side Leptos web app: build a nested filter to find Age of Empires II civs by the units/techs they have.

## Goal

A UI where you assemble a filter from AoE2 units & techs as a **list of rows**:
- Each **row** is an OR-group — the civ must have at least one of the pills in that row.
- Rows combine as AND — *every* row's condition must be satisfied.
- So the whole filter `Vec<Vec<Pill>>` = AND over rows, OR within each row.

The civ-matching engine, data ingestion, and persistence are deferred. This doc + repo cover the UI + state model + drag-and-drop only.

## Stack (decided)

| Concern | Choice | Why |
|---|---|---|
| Framework | **Leptos 0.8** (CSR `< `csr` `>` feature) | Fine-grained reactivity, declarative `view!`, full Rust |
| Build/dev server | **Trunk** | `trunk serve`; handles wasm + bundling + live reload |
| Styling | **Plain CSS** (`styles.css`, `class:` attr) | Zero tooling; keep it simple while learning |
| Combobox | **Custom (Leptos)** — input + filtered list + keyboard nav | `leptix-select` was tried and dropped: no search, broken scrollable viewport in the dropdown |
| Drag & drop | **`sortable-js`** (SortableJS bindings) | Mature, battle-tested, native touch + multi-list "group" support — no handrolling the matrix |

Notes on the two "thin spots":
- **Combobox**: `leptix-select` (v0.1.6) shipped a non-scrollable viewport (options clipped by the portal) and no text filter — both fatal for a 300+ option list. No mature searchable combobox exists for Leptos 0.8 yet (`shadcn-leptos-ui` is the closest but drags its own primitives stack; `leptos-autocomplete` targets 0.5/0.6). So `combobox.rs` hand-rolls one: input + case-insensitive substring filter + scrollable list + ArrowUp/Down/Enter/Escape + `on_pick` callback. ~70 lines, full control, plain DOM styling.
- **DnD**: Leptos ecosystem DnD crates (`leptos_dnd`, `leptix-ui` dnd, `taino-leptos-dnd`) are pre-alpha/weak on touch. SortableJS is the mature, touch-capable, multi-list choice without handrolling. `sortable-js` (Ekleog/sortable-js-rs) integrates via Trunk out of the box.

## File layout

```
index.html          # Trunk entry (mounts <main>, loads styles)
Trunk.toml
Cargo.toml
src/
  main.rs           # mount_to_body(App)
  app.rs            # rows signal + board render + "+ add filter" flow
  components/
    mod.rs
    combobox.rs     # custom searchable combobox -> on_pick callback
    dnd.rs          # SortableJS per-row: apply to a row element, emit MovePills from event indices
    filter_board.rs # rows + annotation + "+ New filter" button
    filter_row.rs   # one row (OR-group of pills, AND container)
    pill.rs         # draggable pill chip + removal ✕
  styles.css
```

## State model

```rust
#[derive(Clone, PartialEq, Eq, Hash)]
struct Pill { id: u64, name: String }

#[derive(Clone, PartialEq)]
struct Row { id: u64, pills: Vec<Pill> }

// Outer vec = rows (AND); inner vec = pills (OR)
let (rows, set_rows) = signal(Vec::<Row>::new());
```

Planned operations (all "locate by id, delete, then insert"):
- `+ New filter` → open combobox; on pick → append a new row with one pill.
- Remove pill ✕ → remove from its row; row auto-removes if empty.
- Drag within row → reorder.
- Drag across rows → move pill (source row auto-removes if empty).

## DnD integration approach

SortableJS mutates the DOM; Leptos renders from the `rows` signal. The rule that keeps them from fighting: **one owner of the DOM between drops, and the model is updated from event indices — never by reading the DOM back.**

1. Rows have stable identity: `struct Row { id: u64, pills: Vec<Pill> }`, and the board `<For>` is keyed by `row.id` (not by contents), so reordering does not destroy the row element that owns the Sortable.
2. Each `FilterRow` binds one `Sortable` to its own element via `NodeRef::<html::Div>::new()` + `row_ref.on_load(...)`. `on_load` fires from the element's own build, so there is no "scan the document after render" race and no skip flag. The instance is parked in `StoredValue<Option<Sortable>, LocalStorage>` — `Sortable` is `!Send`/`!Sync`, and `LocalStorage` wraps it in `SendWrapper` tied to the owner, so it is `destroy()`ed when the row is disposed (no leaked global vec).
3. `on_end` reads only `evt.old_index`/`evt.new_index` plus the `data-row-id` of `evt.from`/`evt.to`, then emits a `MovePills` payload. The board applies the move to the model by row id/index (removing a source row if it empties). It never enumerates DOM pills.
4. The model update is deferred one frame via `request_animation_frame`, so SortableJS's callback stack has unwound before any row (and its `Sortable`) is destroyed.
5. `draggable(".pill")` + `filter(".pill-remove")` keep the ✕ button from starting a drag.
6. Cross-row drops need one cleanup: SortableJS physically moves the pill node into the target row, but the target row's keyed `<For>` does not know that node, so Leptos creates a second one — a duplicate. After the model update renders, we remove the orphaned moved node if it is still connected (guarded to `from_row != to_row`; within-row moves reuse Leptos' own node, so they must not be removed).

### Leptos gotchas hit here (worth remembering)

- Write data attributes on HTML elements as `data-foo=...`, **not** `attr:data-foo=...`. The latter renders a literal attribute named `attr:data-foo` (this silently broke all `data-*` lookups, and the combobox highlight, until fixed).
- Keyed `<For>` only calls `children` for *new* keys; for an existing key it reuses the old view without re-invoking it. A child that takes a snapshot prop will therefore go stale — read derived data through a `Memo`/signal instead.

## Milestones (in order)

1. Scaffold Trunk + Leptos hello-world; `trunk serve` renders. ✅
2. Custom searchable combobox + "+ New filter" opens it; a pick appends a row with one pill. ✅
3. Pill removal + AND/OR row rendering. ✅
4. SortableJS: intra-row reorder + cross-row drag. ✅
5. Plain CSS pass + mobile/touch test (`trunk serve --address 0.0.0.0`). ⬜

## Open questions (deferred, not blocking)

- Scope of "new row" button: sub-list vs always
- Result display later (matching civs)
