# aoe2-civs — Planning document

A small client-side Leptos web app: build a nested filter to find Age of Empires II civs by the units/techs they have.

## Goal

A UI where you assemble a filter from AoE2 units & techs as a **list of rows**:
- Each **row** is an OR-group — the civ must have at least one of the pills in that row.
- Rows combine as AND — *every* row's condition must be satisfied.
- So the whole filter `Vec<Vec<Pill>>` = AND over rows, OR within each row.

Persistence stays deferred; the civ-matching engine and real-data ingestion are planned below ("Real data & matching").

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
scripts/
  generate_data.py  # one-off/dev: bake upstream data into src/data.rs (committed output)
src/
  main.rs           # mount_to_body(App)
  app.rs            # rows signal + board render + "+ add filter" flow
  data.rs           # GENERATED: OPTIONS catalog + CIVS membership (do not hand-edit)
  model.rs          # Pill/Row/FilterKey + apply_move/extract + matching_civs
  components/
    mod.rs
    combobox.rs     # custom searchable combobox -> on_pick callback
    dnd.rs          # SortableJS per-row: apply to a row element, emit MovePills from event indices
    filter_board.rs # rows + annotation + results + "+ New filter" button
    filter_row.rs   # one row (OR-group of pills, AND container)
    pill.rs         # draggable pill chip + removal ✕
  styles.css
```

## State model

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Group { Unit, Tech }

/// Identity of a filterable thing, unique across groups (see "id collision" below).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct FilterKey { group: Group, data_id: u32 }

#[derive(Clone, PartialEq, Eq, Hash)]
struct Pill { id: u64, key: FilterKey, name: String }  // id = app drag identity

#[derive(Clone, PartialEq)]
struct Row { id: u64, pills: Vec<Pill> }

// Outer vec = rows (AND); inner vec = pills (OR)
let (rows, set_rows) = signal(Vec::<Row>::new());
```

Planned operations (all "locate by id, delete, then insert"):
- `+ New filter` → open combobox; on pick → append a new row with one pill. Dedupe by `FilterKey`, never by label.
- Remove pill ✕ → remove from its row; row auto-removes if empty.
- Drag within row → reorder.
- Drag across rows → move pill (source row auto-removes if empty).
- Drag out to blank space → extract pill into its own new row (same model rule: source row replaced/removed if empty).

Matching: `matching_civs(&rows, CIVS) -> Vec<&'static str>` — a civ matches iff **every** row has ≥1 pill whose `FilterKey` is in that civ's set (AND over rows, OR within a row). Pure function, unit-tested with a fixture.

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

## Real data & matching

Source: [`SiegeEngineers/aoe2techtree`](https://github.com/SiegeEngineers/aoe2techtree) (MIT), files under `data/`:

- `data.json` — top-level `civs`: each civ → `{ Unit: [ids], Tech: [ids], Building: [ids], ... }` (dataset IDs). Also master `data.{Building,Tech,Unit,unit_upgrades}` with stats.
- `data/trees/<CIV>.json` — pre-baked per-civ tree layouts; each node carries `id` (`Unit_<dsid>_<bldg>` / `Tech_<dsid>_<bldg>`), `name` (plain English), `name_string_id`, `node_type`, `use_type`.
- `data/locales/<lang>/strings.json` — maps `name_string_id` → localized label (17 locales, all with full coverage).

### Findings (verified against upstream)

- **Do the locale files tie IDs to labels?** Yes, but *indirectly*: the civ-array dataset IDs (and the `LanguageNameId` stat fields) are **not** keys in `strings.json`. The link is **`name_string_id`**, which only lives on the per-civ `trees/*.json` nodes. Chain: civ array id → `(group, dsid)` node in trees → `name_string_id` → `strings.json` label.
- **ID collision is real**: unit and tech dataset-ID spaces overlap (`Unit 4 = "Archer"`, `Tech 4 = "Cotton Armors"`; **33** integers appear in both a Unit and a Tech civ list). Within a group, IDs are unambiguous (verified 0 cases of one `(group, id)` mapping to multiple labels). => every identity is `FilterKey { group, data_id }`; the generator asserts within-group label uniqueness and fails loudly otherwise.
- **Catalog**: 238 unit + 192 tech = **430** labelable options; 49 of those are universal (present in every civ — Loom, Archer, Skirmisher, Masonry, …) and are excluded as non-differentiating, leaving **~381** pickable options. Buildings (39, also labelable) are excluded from the picker.
- **Civ membership** is compact: 53 civs, ~6,410 `(civ, FilterKey)` references (~121/civ).
- **Label cleanup**: strip `<br>` / `\n` (e.g. `"Elite<br>\nSteppe Lancer"` → `"Elite Steppe Lancer"`).

### Ingestion decision: bake at build time

A `scripts/generate_data.py` downloads the upstream files once and emits a committed `src/data.rs`:

- `OPTIONS: &[Option { group, data_id, label, label_id }]` — ~381 entries, sorted; `label_id` = `name_string_id` kept for future i18n.
- `CIVS: &[Civ { name, keys: &[FilterKey] }]` — Unit+Tech membership only.

No network at `cargo build`/`trunk build`; re-run the script to refresh data.

**i18n later**: language switching does not require runtime-parsing `data.json`. Because every option stores its `name_string_id`, a locale switch is just resolving labels from a different `strings.json` (fetched at runtime or baked per-locale) — all 17 locales already resolve every label.

## Milestones (in order)

1. Scaffold Trunk + Leptos hello-world; `trunk serve` renders. ✅
2. Custom searchable combobox + "+ New filter" opens it; a pick appends a row with one pill. ✅
3. Pill removal + AND/OR row rendering. ✅
4. SortableJS: intra-row reorder + cross-row drag + drag-out-to-new-row. ✅
5. Plain CSS pass + mobile/touch test (`trunk serve --address 0.0.0.0`). ✅

### Part 2 — real data + matching

6. **Generator + baked data.** `scripts/generate_data.py` emits `src/data.rs` (`OPTIONS`, `CIVS`); strip `<br>`/`\n`; exclude buildings + universal options; assert within-group label uniqueness. A native test pins catalog counts (~381 options / 53 civs) and rejects any unlabeled entry.
7. **Model.** `Group`, `FilterKey`, `Pill.key`; `matching_civs(rows, CIVS)` (AND over rows, OR within) + fixture unit tests (empty filter => all civs; single row OR; multi-row AND; no match).
8. **UI.** Combobox emits `(FilterKey, label)`; board dedupes by `FilterKey`; results panel shows match count + civ names, reactive via `Memo`. CSS for the results area.
9. **Pass.** `cargo test`, wasm build, mobile check (picker + results), update PLANNING wrap-up.

## Open questions (deferred, not blocking)

- Scope of "new row" button: sub-list vs always
- i18n / locale switcher (data model already keeps `label_id`; needs a `strings.json` fetch or per-locale bake)
- Persistence (shareable filter via URL or `localStorage`)
- Whether to expose the excluded universal options behind a toggle
