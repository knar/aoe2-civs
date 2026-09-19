#!/usr/bin/env python3
"""Bake AoE2 civ/unit/tech data from SiegeEngineers/aoe2techtree into src/data.rs.

The upstream dataset is not directly consumable:

  * data.json lists each civ as dataset-ID arrays grouped under keys
    ``Unit`` / ``Tech`` / ``Building``.
  * those dataset IDs are NOT keys in the locale ``strings.json``. The human
    label is reached through each pre-baked tree node's ``name_string_id``:
    civ-array id -> (group, data_id) node in data/trees/*.json -> name_string_id
    -> strings.json label.
  * the unit and tech dataset-ID spaces overlap (e.g. Unit 4 = "Archer" while
    Tech 4 = "Cotton Armors"), so identity is the pair (Group, data_id).

This script downloads the upstream files once and emits a committed ``src/data.rs``
so the normal build needs no network. Re-run it to refresh the data.

Usage:  python3 scripts/generate_data.py
"""

from __future__ import annotations

import json
import re
import sys
import urllib.request
from collections import defaultdict
from pathlib import Path

REPO = "SiegeEngineers/aoe2techtree"
BRANCH = "master"
RAW = f"https://raw.githubusercontent.com/{REPO}/{BRANCH}/data"
TREES_API = f"https://api.github.com/repos/{REPO}/contents/data/trees?ref={BRANCH}"

# node_type -> Group
UNIT_NODE_TYPES = {"Unit", "UnitUpgrade", "UniqueUnit", "RegionalUnit"}
TECH_NODE_TYPES = {"Research"}

OUT = Path(__file__).resolve().parent.parent / "src" / "data.rs"


def fetch_json(url: str):
    req = urllib.request.Request(url, headers={"User-Agent": "aoe2-civs-generator"})
    with urllib.request.urlopen(req) as resp:
        return json.load(resp)


def clean_label(text: str) -> str:
    """Strip markup the upstream labels carry (e.g. ``Elite<br>\\nSteppe Lancer``)."""
    if not text:
        return text
    text = re.sub(r"<br\s*/?>", " ", text, flags=re.IGNORECASE)
    text = re.sub(r"<[^>]+>", "", text)
    text = text.replace("\\n", " ").replace("\n", " ")
    text = re.sub(r"\s+", " ", text).strip()
    # English strings.json line-breaks hyphenated words ("Counter- weights").
    text = re.sub(r"(?<=\w)- (?=\w)", "", text)
    # ...and abbreviates a few names the tree spells out in full.
    text = re.sub(r"\bE\.\s+", "Elite ", text)
    text = text.replace("Heavy Demo Ship", "Heavy Demolition Ship")
    return text


def group_for(node_type: str):
    if node_type in UNIT_NODE_TYPES:
        return "Unit"
    if node_type in TECH_NODE_TYPES:
        return "Tech"
    return None


def icon_dir(node_type: str):
    """img/ subfolder holding a node's picture file (aoe2techtree.net/img/...)."""
    if node_type in UNIT_NODE_TYPES:
        return "Unit"
    if node_type in TECH_NODE_TYPES:
        return "Tech"
    return None


def walk(node, visit):
    if isinstance(node, dict):
        visit(node)
        for value in node.values():
            walk(value, visit)
    elif isinstance(node, list):
        for value in node:
            walk(value, visit)


def main() -> int:
    print(f"fetching {RAW}/data.json ...")
    data = fetch_json(f"{RAW}/data.json")
    print(f"fetching {RAW}/locales/en/strings.json ...")
    strings = fetch_json(f"{RAW}/locales/en/strings.json")

    print("listing data/trees ...")
    tree_entries = fetch_json(TREES_API)
    tree_names = sorted(e["name"] for e in tree_entries if e["name"].endswith(".json"))
    print(f"fetching {len(tree_names)} civ trees ...")

    # (group, data_id) -> set of (label_id, label)
    seen: dict[tuple[str, int], set[tuple[int, str]]] = defaultdict(set)
    # (group, data_id) -> (img subfolder, picture_index)
    pics: dict[tuple[str, int], tuple[str, int]] = {}
    # data_ids trained from a dock (building_id 45) -> the unit is navy
    naval_ids: set[tuple[str, int]] = set()

    for i, name in enumerate(tree_names, 1):
        tree = fetch_json(f"{RAW}/trees/{name}")
        if i % 10 == 0 or i == len(tree_names):
            print(f"  [{i}/{len(tree_names)}] {name}")

        def visit(n, tree=tree):
            nid, node_type = n.get("id"), n.get("node_type")
            if not (isinstance(nid, str) and node_type):
                return
            match = re.match(r"^(Unit|Tech|Building)_(\d+)_", nid)
            if not match:
                return
            group = group_for(node_type)
            if group is None:
                return
            data_id = int(match.group(2))
            if group == "Unit" and str(n.get("building_id")) == "45":
                naval_ids.add((group, data_id))
            icon_dir_name = icon_dir(node_type)
            if icon_dir_name is not None and (group, data_id) not in pics:
                pic = re.fullmatch(r"(\d+)", str(n.get("picture_index") or ""))
                if pic:
                    pics[(group, data_id)] = (icon_dir_name, int(pic.group(1)))
            sid = n.get("name_string_id")
            label = clean_label(strings.get(str(sid))) if sid is not None else None
            if not label:
                label = clean_label(n.get("name"))
            if not label:
                raise SystemExit(f"no label for {group} {data_id} in {name}")
            seen[(group, data_id)].add((sid or 0, label))

        walk(tree, visit)

    # within-group label uniqueness (the invariant that makes FilterKey deterministic)
    labels: dict[tuple[str, int], tuple[int, str]] = {}
    conflicts = []
    for key, entries in seen.items():
        if len(entries) > 1:
            conflicts.append((key, sorted(entries)))
        else:
            labels[key] = next(iter(entries))
    if conflicts:
        for key, entries in conflicts[:20]:
            print(f"CONFLICT {key}: {entries}", file=sys.stderr)
        raise SystemExit(f"{len(conflicts)} (group, id) pairs map to multiple labels")

    civs: list[tuple[str, list[tuple[str, int]]]] = []
    for civ_name, groups in data["civs"].items():
        members: list[tuple[str, int]] = []
        for group in ("Unit", "Tech"):
            for data_id in groups.get(group, []):
                if (group, data_id) not in labels:
                    raise SystemExit(f"{civ_name}: {group} {data_id} has no label")
                members.append((group, data_id))
        civs.append((civ_name, members))

    if not civs:
        raise SystemExit("no civs found in data.json")
    total = len(civs)

    # One entity can appear under several dataset IDs (different building slots,
    # or civ-specific variants like generic Pikeman 358 vs Sicilian 1787). Merge
    # by (group, label) so the picker lists each entity once and a civ matches if
    # it has any of the entity's IDs.
    merged: dict[tuple[str, str], dict] = {}
    for _, members in civs:
        for key in members:
            group, data_id = key
            label_id, label = labels[key]
            entry = merged.setdefault((group, label), {"ids": set(), "label_ids": set()})
            entry["ids"].add(data_id)
            entry["label_ids"].add(label_id)
    for (group, label), entry in merged.items():
        if len(entry["label_ids"]) != 1:
            raise SystemExit(
                f"{group} {label!r}: inconsistent name_string_id "
                f"{sorted(entry['label_ids'])}"
            )

    member_sets = [set(members) for _, members in civs]

    unit_stats = data["data"]["Unit"]

    def unit_food_gold(ids: set[int]) -> bool:
        """True when none of the unit's variants costs wood (food+gold / gold-only).

        Feeds the DM-michi overview: wood-free units are the spammable ones.
        """
        for data_id in sorted(ids):
            stats = unit_stats.get(str(data_id))
            if not stats:
                return False
            cost = stats.get("Cost") or {}
            if cost.get("Wood", 0) > 0:
                return False
        return True

    def coverage(group: str, ids: set[int]) -> int:
        return sum(1 for members in member_sets if any((group, i) in members for i in ids))

    covered = {mk: coverage(mk[0], entry["ids"]) for mk, entry in merged.items()}

    def merged_icon(group: str, ids: set[int]) -> str:
        """Icon of the option's lowest dataset ID (civ variants all share pics)."""
        for data_id in sorted(ids):
            pic = pics.get((group, data_id))
            if pic:
                return f"{pic[0]}/{pic[1]}"
        return "missing"

    # options present in every civ cannot differentiate -> excluded from the picker
    universal = {mk for mk, count in covered.items() if count == total}
    options = [mk for mk in merged if mk not in universal]
    options.sort(key=lambda mk: (0 if mk[0] == "Unit" else 1, mk[1].casefold()))

    def rust_str(value: str) -> str:
        return value.replace("\\", "\\\\").replace('"', '\\"')

    lines: list[str] = []
    add = lines.append
    add("// @generated by scripts/generate_data.py -- do not edit by hand.")
    add(f"// Source: https://github.com/{REPO} (MIT).")
    add("")
    add("use crate::model::{Civ, FilterKey, Group};")
    add("")
    add("#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]")
    add("pub struct CivOption {")
    add("    pub label: &'static str,")
    add("    pub label_id: u32,")
    add("    pub group: Group,")
    add("    /// Matches exactly one civ (unique unit or civ-exclusive tech).")
    add("    pub unique: bool,")
    add("    /// Unit costs no wood (food+gold / gold-only); false for techs.")
    add("    pub food_gold: bool,")
    add("    /// Unit is trained from a dock (navy); false for techs.")
    add("    pub naval: bool,")
    add("    /// Icon path under https://aoe2techtree.net/img/ (e.g. \"Unit/42\").")
    add("    pub icon: &'static str,")
    add("    pub keys: &'static [FilterKey],")
    add("}")
    add("")
    add("const fn uk(data_id: u32) -> FilterKey {")
    add("    FilterKey { group: Group::Unit, data_id }")
    add("}")
    add("")
    add("const fn tk(data_id: u32) -> FilterKey {")
    add("    FilterKey { group: Group::Tech, data_id }")
    add("}")
    add("")
    add("pub const OPTIONS: &[CivOption] = &[")
    for group, label in options:
        entry = merged[(group, label)]
        label_id = next(iter(entry["label_ids"]))
        fn = "uk" if group == "Unit" else "tk"
        key_list = ", ".join(f"{fn}({i})" for i in sorted(entry["ids"]))
        unique = "true" if covered[(group, label)] == 1 else "false"
        food_gold = unit_food_gold(entry["ids"]) if group == "Unit" else False
        naval = group == "Unit" and any((group, i) in naval_ids for i in entry["ids"])
        icon = merged_icon(group, entry["ids"])
        add(
            f'    CivOption {{ label: "{rust_str(label)}", '
            f"label_id: {label_id}, group: Group::{group}, unique: {unique}, "
            f"food_gold: {str(food_gold).lower()}, naval: {str(naval).lower()}, "
            f"icon: \"{icon}\", "
            f"keys: &[{key_list}] }},"
        )
    add("];")
    add("")
    add("pub const CIVS: &[Civ] = &[")
    for civ_name, members in sorted(civs, key=lambda c: c[0].casefold()):
        add(f'    Civ {{ name: "{rust_str(civ_name)}", keys: &[')
        # wrap to ~96 columns
        current = "        "
        for group, data_id in members:
            piece = f'{"uk" if group == "Unit" else "tk"}({data_id}), '
            if len(current) + len(piece) > 96:
                add(current.rstrip())
                current = "        "
            current += piece
        if current.strip():
            add(current.rstrip().rstrip(",") + ",")
        add("    ] },")
    add("];")
    add("")

    OUT.write_text("\n".join(lines))

    unit_count = sum(1 for g, _ in options if g == "Unit")
    tech_count = len(options) - unit_count
    multi = sum(1 for entry in merged.values() if len(entry["ids"]) > 1)
    unique = sum(1 for mk in options if covered[mk] == 1)
    print()
    print(f"wrote {OUT}")
    print(f"  civs:            {total}")
    print(f"  options:         {len(options)} ({unit_count} units, {tech_count} techs)")
    print(f"  unique (1-civ):  {unique}")
    print(f"  merged (multi-ID options): {multi}")
    print(f"  universal (out): {len(universal)}")
    print(f"  civ references:  {sum(len(m) for _, m in civs)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
