#!/usr/bin/env python3
"""
Fetch monster data from OSRS Wiki Bucket API and save in canonical format.

Uses the same API approach as osrs-dps-calc:
https://github.com/weirdgloop/osrs-dps-calc/blob/main/scripts/generateMonsters.py

Output format matches osrsreboxed-db style for compatibility with existing tooling.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Optional

WIKI_BASE = "https://oldschool.runescape.wiki"
API_BASE = f"{WIKI_BASE}/api.php"
USER_AGENT = "osrs-sim (https://github.com/your-repo/osrs-sim)"

# Fields to fetch from the wiki's infobox_monster bucket
BUCKET_API_FIELDS = [
    "page_name",
    "page_name_sub",
    "id",
    "name",
    "combat_level",
    "hitpoints",
    "attack_level",
    "strength_level",
    "defence_level",
    "magic_level",
    "ranged_level",
    "attack_bonus",
    "strength_bonus",
    "magic_attack_bonus",
    "magic_damage_bonus",
    "range_attack_bonus",
    "range_strength_bonus",
    "stab_defence_bonus",
    "slash_defence_bonus",
    "crush_defence_bonus",
    "magic_defence_bonus",
    "range_defence_bonus",
    "attack_speed",
    "attack_style",
    "max_hit",
    "size",
    "attribute",
    "slayer_category",
    "slayer_experience",
    "slayer_level",
    "poison_immune",
    "venom_immune",
    "image",
    # New ranged defence fields
    "light_range_defence_bonus",
    "standard_range_defence_bonus",
    "heavy_range_defence_bonus",
    # Elemental weakness
    "elemental_weakness",
    "elemental_weakness_percent",
]

# Monsters to skip (discontinued, unreleased, or otherwise problematic)
MONSTERS_TO_SKIP = [
    "Albatross",
    "Armoured kraken",
    "Bull shark",
    "Butterfly ray",
    "Eagle ray",
    "Frigatebird",
    "Great white shark",
    "Hammerhead shark",
    "Manta ray (Sailing)",
    "Mogre (Sailing)",
    "Narwhal",
    "Orca",
    "Osprey",
    "Pygmy kraken",
    "Spined kraken",
    "Stingray",
    "Tern",
    "Tiger shark",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Fetch monster data from OSRS Wiki Bucket API"
    )
    parser.add_argument(
        "--out-dir",
        default="data/monsters",
        help="Output directory for per-monster JSON files (default: data/monsters)",
    )
    parser.add_argument(
        "--filter",
        choices=("all", "slayer", "boss", "slayer+boss"),
        default="all",
        help="Filter monsters (default: all)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Fetch and process but don't write files",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Print verbose progress",
    )
    return parser.parse_args()


def fetch_monsters_from_wiki() -> list[dict[str, Any]]:
    """Fetch all monsters from the OSRS Wiki Bucket API."""
    monsters: list[dict[str, Any]] = []
    offset = 0
    fields_csv = ",".join(map(repr, BUCKET_API_FIELDS))

    while True:
        print(f"Fetching monsters from wiki (offset={offset})...", file=sys.stderr)

        query_str = (
            f"bucket('infobox_monster')"
            f".select({fields_csv})"
            f".limit(500).offset({offset})"
            f".where(bucket.Not('Category:Discontinued content'))"
            f".orderBy('page_name_sub', 'asc').run()"
        )

        params = {
            "action": "bucket",
            "format": "json",
            "query": query_str,
        }

        url = f"{API_BASE}?{urllib.parse.urlencode(params)}"
        req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})

        with urllib.request.urlopen(req, timeout=30) as response:
            data = json.loads(response.read().decode("utf-8"))

        if "bucket" not in data:
            break

        monsters.extend(data["bucket"])

        if len(data["bucket"]) == 500:
            offset += 500
        else:
            break

    return monsters


def safe_int(value: Any, default: int = 0) -> int:
    """Safely convert a value to int, handling lists and None."""
    if value is None:
        return default
    if isinstance(value, list):
        value = value[0] if value else default
    try:
        return int(value)
    except (TypeError, ValueError):
        return default


def safe_uint(value: Any, default: int = 0) -> int:
    """Safely convert a value to unsigned int, clamping negatives to 0."""
    result = safe_int(value, default)
    return max(0, result)


def parse_max_hit(value: Any) -> int:
    """Parse max_hit which can be int, string like '97 (Melee)', or list of such."""
    if value is None:
        return 0
    if isinstance(value, list):
        # Take the first/highest numeric value from the list
        max_val = 0
        for item in value:
            parsed = parse_max_hit(item)
            if parsed > max_val:
                max_val = parsed
        return max_val
    if isinstance(value, int):
        return value
    if isinstance(value, str):
        # Extract leading numeric portion from strings like "97 (Melee)"
        match = re.match(r"^(\d+)", value.strip())
        if match:
            return int(match.group(1))
    return 0


def safe_str(value: Any, default: str = "") -> str:
    """Safely convert a value to str, handling lists and None."""
    if value is None:
        return default
    if isinstance(value, list):
        value = value[-1] if value else default
    return str(value)


def safe_bool(value: Any, default: bool = False) -> bool:
    """Safely convert a value to bool."""
    if value is None:
        return default
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value.lower() in ("true", "yes", "1")
    return bool(value)


def normalize_attack_style(style: Any) -> list[str]:
    """Normalize attack style to a list of lowercase strings."""
    if style is None or style == "None" or style == "N/A":
        return []
    if isinstance(style, str):
        return [style.lower()]
    if isinstance(style, list):
        return [s.lower() for s in style if s and s not in ("None", "N/A")]
    return []


def normalize_attributes(attrs: Any) -> list[str]:
    """Normalize attributes to a list of lowercase strings."""
    if attrs is None:
        return []
    if isinstance(attrs, str):
        return [attrs.lower()]
    if isinstance(attrs, list):
        return [a.lower() for a in attrs if a]
    return []


def normalize_slayer_category(category: Any) -> list[str]:
    """Normalize slayer category to a list of strings."""
    if category is None:
        return []
    if isinstance(category, str):
        return [category]
    if isinstance(category, list):
        return [c for c in category if c]
    return []


# Regex to strip wiki parser markers
STRIP_MARKER_RE = re.compile(r"['\"`]*UNIQ--[a-zA-Z0-9]+-[0-9A-F]{8}-QINU['\"`]*")


def strip_parser_tags(value: Any) -> Any:
    """Recursively strip wiki parser tags from values."""
    if isinstance(value, dict):
        return {k: strip_parser_tags(v) for k, v in value.items()}
    if isinstance(value, list):
        return [strip_parser_tags(item) for item in value]
    if isinstance(value, str):
        return STRIP_MARKER_RE.sub("", value).strip()
    return value


def transform_monster(wiki_data: dict[str, Any]) -> Optional[dict[str, Any]]:
    """Transform wiki monster data to canonical format."""
    page_name_sub = wiki_data.get("page_name_sub", "")
    page_name = wiki_data.get("page_name", "")

    # Extract version from page_name_sub
    try:
        version = page_name_sub.split("#", 1)[1]
    except IndexError:
        version = ""

    # Skip Challenge Mode variants (handled by calculator UI)
    if "Challenge Mode" in version:
        return None

    # Skip non-main namespace
    if re.match(r"^[A-Za-z]+:", page_name_sub):
        return None

    # Skip specific monsters
    if page_name in MONSTERS_TO_SKIP:
        return None

    # Skip spawn points and non-attackable variants
    if "Spawn point" in version:
        return None
    if "Asleep" in version or "Defeated" in version:
        return None

    # Skip barriers (non-attackable)
    if re.match(r"^(Strong|Weak|Medium|Overcharged) Barrier$", page_name_sub):
        return None

    # Get monster ID
    monster_id = safe_int(wiki_data.get("id"))
    if monster_id == 0:
        return None

    # Get hitpoints - skip if 0 (non-attackable)
    hitpoints = safe_int(wiki_data.get("hitpoints"))
    if hitpoints == 0:
        return None

    # Skip historical/arena variants
    name_lower = page_name.lower()
    if "(historical)" in name_lower:
        return None
    if "(pvm arena)" in name_lower:
        return None
    if "(deadman: apocalypse)" in name_lower:
        return None
    if "(last man standing)" in name_lower:
        return None

    # Build canonical monster object
    slayer_category = normalize_slayer_category(wiki_data.get("slayer_category"))
    attributes = normalize_attributes(wiki_data.get("attribute"))

    # Determine if this is a slayer monster or boss
    is_slayer = bool(slayer_category) or safe_int(wiki_data.get("slayer_experience")) > 0
    is_boss = "boss" in attributes or "Bosses" in slayer_category

    monster = {
        "id": monster_id,
        "name": page_name,
        "version": version if version else None,
        "combat_level": safe_int(wiki_data.get("combat_level")),
        "hitpoints": hitpoints,
        # Combat levels
        "attack_level": safe_int(wiki_data.get("attack_level")),
        "strength_level": safe_int(wiki_data.get("strength_level")),
        "defence_level": safe_int(wiki_data.get("defence_level")),
        "magic_level": safe_int(wiki_data.get("magic_level")),
        "ranged_level": safe_int(wiki_data.get("ranged_level")),
        # Offensive bonuses
        "attack_bonus": safe_int(wiki_data.get("attack_bonus")),
        "strength_bonus": safe_int(wiki_data.get("strength_bonus")),
        "attack_magic": safe_int(wiki_data.get("magic_attack_bonus")),
        "magic_bonus": safe_int(wiki_data.get("magic_damage_bonus")),
        "attack_ranged": safe_int(wiki_data.get("range_attack_bonus")),
        "ranged_bonus": safe_int(wiki_data.get("range_strength_bonus")),
        # Defence bonuses
        "defence_stab": safe_int(wiki_data.get("stab_defence_bonus")),
        "defence_slash": safe_int(wiki_data.get("slash_defence_bonus")),
        "defence_crush": safe_int(wiki_data.get("crush_defence_bonus")),
        "defence_magic": safe_int(wiki_data.get("magic_defence_bonus")),
        "defence_ranged": safe_int(wiki_data.get("range_defence_bonus")),
        # New ranged defence bonuses (light/standard/heavy)
        "defence_ranged_light": safe_int(wiki_data.get("light_range_defence_bonus")),
        "defence_ranged_standard": safe_int(wiki_data.get("standard_range_defence_bonus")),
        "defence_ranged_heavy": safe_int(wiki_data.get("heavy_range_defence_bonus")),
        # Combat properties
        "attack_speed": safe_uint(wiki_data.get("attack_speed")),
        "attack_type": normalize_attack_style(wiki_data.get("attack_style")),
        "max_hit": parse_max_hit(wiki_data.get("max_hit")),
        "size": safe_uint(wiki_data.get("size"), default=1),
        # Attributes and categories
        "attributes": attributes,
        "category": slayer_category,
        # Slayer info
        "slayer_monster": is_slayer,
        "slayer_level": safe_uint(wiki_data.get("slayer_level"), default=1),
        "slayer_xp": float(safe_int(wiki_data.get("slayer_experience"))),
        # Boss flag
        "boss": is_boss,
        # Immunities
        "immune_poison": safe_bool(wiki_data.get("poison_immune")),
        "immune_venom": safe_bool(wiki_data.get("venom_immune")),
        # Elemental weakness
        "elemental_weakness": safe_str(wiki_data.get("elemental_weakness")) or None,
        "elemental_weakness_percent": safe_int(wiki_data.get("elemental_weakness_percent")),
        # Metadata
        "_source": "osrs_wiki",
        "_wiki_page": page_name_sub,
    }

    # Strip any wiki parser markers
    monster = strip_parser_tags(monster)

    return monster


def should_include(monster: dict[str, Any], filter_mode: str) -> bool:
    """Check if monster should be included based on filter."""
    if filter_mode == "all":
        return True
    if filter_mode == "slayer":
        return monster.get("slayer_monster", False)
    if filter_mode == "boss":
        return monster.get("boss", False)
    if filter_mode == "slayer+boss":
        return monster.get("slayer_monster", False) or monster.get("boss", False)
    return True


def main() -> int:
    args = parse_args()
    out_dir = Path(args.out_dir)

    # Fetch from wiki
    wiki_monsters = fetch_monsters_from_wiki()
    print(f"Fetched {len(wiki_monsters)} raw monster entries from wiki", file=sys.stderr)

    # Transform and deduplicate
    monsters_by_id: dict[int, dict[str, Any]] = {}
    skipped = 0
    filtered = 0

    for wiki_data in wiki_monsters:
        monster = transform_monster(wiki_data)
        if monster is None:
            skipped += 1
            continue

        if not should_include(monster, args.filter):
            filtered += 1
            continue

        monster_id = monster["id"]

        # Prefer non-versioned or handle duplicates
        if monster_id in monsters_by_id:
            existing = monsters_by_id[monster_id]
            # Prefer the base version (no version string)
            if monster.get("version") and not existing.get("version"):
                continue
            # If both have versions or neither, prefer the one we already have
            if args.verbose:
                print(
                    f"  Duplicate ID {monster_id}: keeping '{existing['name']}' "
                    f"over '{monster['name']}' (version: {monster.get('version')})",
                    file=sys.stderr,
                )
            continue

        monsters_by_id[monster_id] = monster
        if args.verbose:
            print(f"  Added: {monster['name']} (ID: {monster_id})", file=sys.stderr)

    print(
        f"Processed: {len(monsters_by_id)} unique monsters "
        f"(skipped {skipped}, filtered {filtered})",
        file=sys.stderr,
    )

    if args.dry_run:
        print("Dry run - not writing files", file=sys.stderr)
        # Print sample
        sample_ids = list(monsters_by_id.keys())[:3]
        for mid in sample_ids:
            print(json.dumps(monsters_by_id[mid], indent=2))
        return 0

    # Write individual JSON files
    out_dir.mkdir(parents=True, exist_ok=True)

    for monster_id, monster in monsters_by_id.items():
        filename = f"{monster_id}.json"
        filepath = out_dir / filename
        filepath.write_text(json.dumps(monster, indent=2, sort_keys=True))

    print(f"Wrote {len(monsters_by_id)} monster files to {out_dir}", file=sys.stderr)

    # Also write a consolidated index file
    index_path = out_dir / "_index.json"
    index_data = {
        "count": len(monsters_by_id),
        "ids": sorted(monsters_by_id.keys()),
        "source": "osrs_wiki",
    }
    index_path.write_text(json.dumps(index_data, indent=2))
    print(f"Wrote index to {index_path}", file=sys.stderr)

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        print("\nInterrupted", file=sys.stderr)
        raise SystemExit(130)
    except Exception as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
