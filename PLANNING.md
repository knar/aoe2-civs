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
  overview.rs       # DM-michi profile: 12 output slots + statuses + curated notes + tests
  components/
    mod.rs
    civ_panel.rs    # civ overview drawer (right side, mobile overlay) + tech-tree link
    combobox.rs     # custom searchable combobox -> on_pick callback
    dnd.rs          # SortableJS per-row: apply to a row element, emit MovePills from event indices
    filter_board.rs # rows + annotation + results (clickable chips) + "+ New filter" button
    filter_row.rs   # one row (OR-group of pills, AND container)
    pill.rs         # draggable pill chip + removal ✕
  styles.css
```

## State model

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Group { Unit, Tech }

/// Identity of one dataset entry. The unit and tech ID spaces overlap, and a
/// single entity can span several IDs (different building slots / civ variants),
/// so an *option* is identified by its label and carries a set of these keys.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct FilterKey { group: Group, data_id: u32 }

#[derive(Clone, PartialEq, Eq, Hash)]
struct Pill { id: u64, keys: &'static [FilterKey], name: String }  // id = drag identity

#[derive(Clone, PartialEq)]
struct Row { id: u64, pills: Vec<Pill> }

// Outer vec = rows (AND); inner vec = pills (OR)
let (rows, set_rows) = signal(Vec::<Row>::new());
```

Planned operations (all "locate by id, delete, then insert"):
- `+ New filter` → open combobox; on pick → append a new row with one pill. Dedupe by the option's key set, never by label.
- Remove pill ✕ → remove from its row; row auto-removes if empty.
- Drag within row → reorder.
- Drag across rows → move pill (source row auto-removes if empty).
- Drag out to blank space → extract pill into its own new row (same model rule: source row replaced/removed if empty).

Matching: `matching_civs(&rows, CIVS) -> Vec<&Civ>` — a civ matches iff **every** row has ≥1 pill where the civ has **any** of that pill's keys (AND over rows, OR within a row). Pure function, unit-tested with a fixture.

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
- **ID collision is real**: unit and tech dataset-ID spaces overlap (`Unit 4 = "Archer"`, `Tech 4 = "Cotton Armors"`; **33** integers appear in both a Unit and a Tech civ list). Within a group, IDs are unambiguous (verified 0 cases of one `(group, id)` mapping to multiple labels). => a `FilterKey { group, data_id }` disambiguates unit-vs-tech; the generator asserts within-group label uniqueness and fails loudly otherwise.
- **One entity, several IDs** (discovered while baking): a unit can appear under multiple dataset IDs — different building slots (e.g. Huskarl in Castle `41` and Barracks `759`) or civ-specific variants (Sicilian Pikeman `1787` vs the generic `358`). Keying options by ID alone both duplicates the label in the picker *and* creates matching gaps (filtering `358` would miss Sicilians). So options are **merged by `(group, label)`** and carry the **set** of dataset IDs; a civ matches if it has *any* of them. 12 such merged options; each has one consistent `name_string_id`.
- **Source of truth for labels**: the per-civ tree `name` can be stale after a tech rename (e.g. node says `Obsidian Arrows`, locale says `Hul'che Javelineers`), so the **locale string wins**. The few English artifacts are normalized: line-break hyphens (`Counter- weights` → `Counterweights`) and abbreviations (`E.` → `Elite`, `Heavy Demo Ship` → `Heavy Demolition Ship`).
- **Catalog**: 248 unit + 199 tech = **447** labelable options; after merging, **435** distinct options, of which **48** are universal (present in every civ — Loom, Archer, Skirmisher, Spearman, Villager, Masonry, …) and excluded as non-differentiating, leaving **387** pickable options (219 units + 168 techs). Buildings (39, also labelable) are excluded from the picker.
- **Civ membership** is compact: 56 civs, ~6,787 `(civ, FilterKey)` references (~121/civ).

### Ingestion decision: bake at build time

A `scripts/generate_data.py` downloads the upstream files once and emits a committed `src/data.rs`:

- `OPTIONS: &[CivOption { label, label_id, group, unique, keys: &[FilterKey] }]` — 387 entries, sorted (units then techs, alphabetical); `keys` holds every dataset ID for the merged entity; `group` is `Unit`/`Tech`; `unique` means the option matches exactly one civ (see below); `label_id` = `name_string_id` kept for future i18n.
- `CIVS: &[Civ { name, keys: &[FilterKey] }]` — Unit+Tech membership only.

No network at `cargo build`/`trunk build`; re-run the script to refresh data.

**i18n later**: language switching does not require runtime-parsing `data.json`. Because every option stores its `name_string_id`, a locale switch is just resolving labels from a different `strings.json` (fetched at runtime or baked per-locale) — all 17 locales already resolve every label.

### Picker filters: hiding civ-specific options

Most of the catalog is noise for multi-civ filtering: **262 of 387 options (68%) match exactly one civ** — every `UniqueUnit` plus every civ-exclusive tech, and nothing else. The generator flags these as `unique` (defined as *matches exactly one civ*, so it stays correct if upstream flags drift).

The picker has two independent toggle chips, **"Unique units"** / **"Unique techs"**, both **off by default** (so the list opens at 125 options); turning one on reveals that category. Toggles live in `FilterBoard` (so they survive the combobox unmounting) and persist to `localStorage` under `aoe2.showUniqueUnits` / `aoe2.showUniqueTechs` via `src/storage.rs`. Hiding is picker-only — already-placed pills keep matching.

## Milestones (in order)

1. Scaffold Trunk + Leptos hello-world; `trunk serve` renders. ✅
2. Custom searchable combobox + "+ New filter" opens it; a pick appends a row with one pill. ✅
3. Pill removal + AND/OR row rendering. ✅
4. SortableJS: intra-row reorder + cross-row drag + drag-out-to-new-row. ✅
5. Plain CSS pass + mobile/touch test (`trunk serve --address 0.0.0.0`). ✅

### Part 2 — real data + matching

6. **Generator + baked data.** `scripts/generate_data.py` emits `src/data.rs` (`OPTIONS`, `CIVS`); strip markup; merge options by label; exclude buildings + universal options; assert within-group label uniqueness and per-option `name_string_id` consistency. A native test pins catalog counts (387 options / 56 civs) and rejects any unlabeled entry. ✅
7. **Model.** `Group`, `FilterKey`, `Pill.keys`; `matching_civs(rows, CIVS)` (AND over rows, OR within; any key per pill) + fixture unit tests (empty filter => all civs; single row OR; multi-row AND; multi-key merge; no match; unit-vs-tech id collision). ✅
8. **UI.** Combobox emits the picked `Option`; board dedupes by key set and builds the pill from it; results panel shows match count + civ names, reactive via `Memo`. CSS for the results area. ✅
9. **Pass.** `cargo test` (22 passing), wasm + `trunk build`, `cargo clippy` (clean), mobile check (picker + results), update PLANNING wrap-up. ✅
10. **Hide civ-specific options.** Bake `unique` (= 1-civ) onto options; two `localStorage`-persisted toggles in the picker (default off); tests pin 262 unique / 147 unit / 115 tech and assert `unique <=> matches one civ`. ✅

### Part 3 — DM michi civ overview

11. **Civ overview panel.** Clicking a civ chip in the results opens a right-side drawer (`components/civ_panel.rs`) with a concise, always-on DM-michi profile:
    - **12 unit/tech slots** (7 core army, 5 siege & defense) each with a deterministic status — `✓` full, `⊘ missing: <upgrades>` (only non-universal, differentiating techs are listed), `▽ <fallback> only` (e.g. Cavalier when no Paladin), `✗` absent. Computed in `src/overview.rs::summary_for(&Civ)` straight from the civ's `CIVS` keys — no extra data.
    - **"Bonuses that matter"** — one to two hand-curated notes per civ, filtered to effects that stay active *after everything is already researched* (post-imp DM): permanent power-unit stats, production-cost/training perks for those units, farming/trade rates, building stats, wood gathering. One-time/pre-age research perks (free/faster techs, earlier ages, eco) are deliberately omitted. Team bonuses flagged. Source: field-guide (`amateurakhbar/aoe2-field-guide` `data/aoe2_data.json`) DE game strings, spot-checked (Burgundians/Turks gunpowder, Mongols Drill).
    - Footer link → `https://aoe2techtree.net/#<CivName>`.
    - Persisted UI decisions: no concise/full toggle (always concise); single "DM michi" profile (slots/notes table is the extension point for more modes).
12. **Tests** (`overview.rs`): every slot/FU/fallback key must exist in the `OPTIONS` catalog; notes exist for all 56 civs; pinned civ expectations match known data facts (Turks BBT ✓ no onager but heavy scorpion; Koreans BBT ✓ no onager/heavy scorpion + champion lacking Blast Furnace; Britons arbalest missing Thumb Ring; Franks paladin missing Bloodlines; Goths champion missing Plate Mail + Arson, Paladin → Cavalier only; Celts Siege Onager ✓ no BBT; Chinese BBT ✓ + heavy scorpion, no onager). ✅ (36 tests total, clippy clean)
13. **Spam & tools redesign.** Replaced the fixed 12-slot grid with a rule-driven, per-civ "good options" sheet — no more ✗/noise rows:
    - **Spam (food & gold)**: every *unique* unit the civ holds whose training cost has no wood (baked at generation time as `food_gold`, from upstream `data.json` `data.Unit[...].Cost`), shown as `Elite X` when the elite line is held, plus always-shown rule slots — Heavy Camel Rider, Paladin, Battle Elephant, Hand Cannoneer, Elite Elephant Archer, Eagle Warrior, Steppe Lancer — each only when present. A coverage fallback guarantees ≥1 spam row (Cavalier → Champion → Arbalest; e.g. Chinese). Champion/Arbalest otherwise appear **only** for hand-curated `SPECIAL_INF`/`SPECIAL_RANGE` civs, kept in sync with NOTES by a test.
    - **Siege tools**: Siege Ram / Siege Elephant / Siege Onager / Heavy Scorpion (with their cheaper fallbacks, e.g. ▽ Capped Ram, ▽ Armored Elephant) + Bombard Cannon + Bombard Tower, only when present; Siege Engineers researched shows as a footnote when the civ actually has a siege line.
    - Absence is quiet — absent lines are dropped, whole empty sections render a muted italic line. `Status::Absent` is never emitted into the panel. ✅ (43 tests, clippy clean; `food_gold` pinned in `data_tests.rs`)
14. **Icon rows + cost-based split (no wood vs wood).** Rows went icon-first and the two sections are now strictly cost-split:
    - Icons: `generate_data.py` now bakes `icon` (`Unit/<picture_index>` / `Tech/<...>`, e.g. `Unit/42` Mangudai) from each tree node's `picture_index`; served from `https://aoe2techtree.net/img/<icon>.png` (48×48, `image-rendering: pixelated`). The panel shows `<img>` rows only; tooltips carry the name / `missing: …` list / `only Capped Ram available`.
    - **Spam (no wood)** = food+gold rows: unique food+gold units (elite-pivoted), rule slots, Champion special, coverage (now Cavalier → Champion only, since Arbalest moved out), **plus Siege Elephant** (food+gold ⇒ spam, fallback Armored Elephant). **Wood units (usually siege)** = Siege Ram / Siege Onager / Heavy Scorpion / Bombard Cannon / Bombard Tower with **fallback shown by swapping to the base unit's icon** (Capped Ram icon, Onager icon, Scorpion icon — dashed border + tooltip instead of "▽ X only" text), the Arbalest `SPECIAL_RANGE` slot (upstream now labels it "Arbalester"), and **every wood-costing unique unit** with elite pivot (Mangudai, Chu Ko Nu, Longboat, Thirisadai, Turtle Ship…). Appearance mirrors the spam side: absent lines dropped, empty sections get a muted line, partial upgrades dim the icon (`opacity: .55`).
    - Verified: all 102 distinct row icons HEAD-checked 200 from aoe2techtree.net; every row carries a real (non-`missing`) icon in a test. ✅ (44 tests, clippy clean, `trunk build` ok; icon format pinned in `data_tests.rs`)
    - **Naval units dropped entirely** (land-only profile): the generator now bakes `naval` from each tree node's `building_id == 45` (dock) — Longboat, Turtle Ship, Caravel, Thirisadai, Dromon etc. never appear in any section; a test asserts no summary row resolves from a `naval` option.
    - **Fallback = base icon, pinned**: base-only civs render the base unit's icon (Capped Ram `Unit/63`, Onager `Unit/101`, Scorpion `Unit/80`) — the lower-tier state is visible in the icon itself, not just the tooltip. ✅ (46 tests, clippy clean)
    - **Refinements**: tooltip for base-only rows is just the base unit's name (`Onager`, not `Siege Onager — only Onager available`); Scorpion-only civs **omit** the row entirely (no fallback on Heavy Scorpion); added the Three-Kingdoms **Heavy Rocket Cart** line (`uk(1907)` → Rocket Cart fallback `uk(1904)`, Chinese/Jurchens/Khitans/Koreans, `Unit/460` icon, verified live). ✅ (46 tests, clippy clean)
    - **Replaced-outclassed lines**: Hindustanis skip Heavy Camel Rider (Imperial Camel Rider `uk(207)` supersedes it) and Bohemians skip Bombard Cannon (Houfnice `uk(1709)` supersedes it) — a `REPLACES` table in `overview.rs` suppresses the generic slot when the civ holds the replacing unit.
    - **Melee backbone guarantee**: every civ always lists a food+gold melee line — generic **Cavalier** when it has the stable line and no premium cavalry row is already shown (Sicilians, Goths, Vikings, Koreans…), else **Champion** when the civ completely lacks a stable (Aztecs). Champion/Arbalest specials and coverage stay untouched.
    - **Armenian note**: "Fereters researched: infantry +30 HP" + team infantry +2 LOS (previously blank → muted "nothing notable" line). ✅ (51 tests, clippy clean)

## Open questions (deferred, not blocking)

- Scope of "new row" button: sub-list vs always
- i18n / locale switcher (data model already keeps `label_id`; needs a `strings.json` fetch or per-locale bake)
- Persist the *filter itself* (shareable via URL or `localStorage`), not just the picker toggles
- Whether to expose the excluded universal options behind a toggle
- Category tabs (All/Units/Techs) in the picker — `group` already supports it
