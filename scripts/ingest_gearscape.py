#!/usr/bin/env python3
"""
Ingest equipment and weapon data from Gearscape API.

Fetches from:
  - https://api.gearscape.net/api/equipment/all
  - https://api.gearscape.net/api/weapon/all

Outputs individual JSON files to data/items/ in OSRSBox-compatible format.

Idempotent: only writes files that have changed.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from typing import Any
from urllib.request import urlopen
from urllib.error import URLError

EQUIPMENT_URL = "https://api.gearscape.net/api/equipment/all"
WEAPON_URL = "https://api.gearscape.net/api/weapon/all"

# Slot name mapping (Gearscape uses same names, but ensure consistency)
SLOT_MAP = {
    "head": "head",
    "cape": "cape",
    "neck": "neck",
    "ammunition": "ammo",
    "body": "body",
    "shield": "shield",
    "legs": "legs",
    "hands": "hands",
    "feet": "feet",
    "ring": "ring",
    "weapon": "weapon",
}


def fetch_json(url: str) -> Any:
    """Fetch JSON from URL."""
    print(f"Fetching {url}...")
    try:
        with urlopen(url, timeout=30) as resp:
            data = json.loads(resp.read().decode("utf-8"))
        return data
    except URLError as e:
        print(f"Error fetching {url}: {e}", file=sys.stderr)
        raise SystemExit(1)


def transform_equipment(item: dict[str, Any], slot: str) -> dict[str, Any]:
    """Transform Gearscape equipment item to OSRSBox-compatible format."""
    # Build requirements dict
    requirements = {}
    req_fields = [
        ("attack_req", "attack"),
        ("strength_req", "strength"),
        ("defence_req", "defence"),
        ("ranged_req", "ranged"),
        ("magic_req", "magic"),
        ("prayer_req", "prayer"),
        ("hitpoints_req", "hitpoints"),
        ("slayer_req", "slayer"),
    ]
    for gs_field, our_field in req_fields:
        val = item.get(gs_field, 1)
        if val and val > 1:
            requirements[our_field] = val

    return {
        "id": item["id"],
        "name": item["name"],
        "equipable": True,
        "equipable_by_player": True,
        "equipable_weapon": False,
        "members": item.get("members", True),
        "tradeable": item.get("tradeable", True),
        "weight": item.get("weight", 0),
        "equipment": {
            "slot": SLOT_MAP.get(slot, slot),
            "attack_stab": item.get("stab_bonus", 0),
            "attack_slash": item.get("slash_bonus", 0),
            "attack_crush": item.get("crush_bonus", 0),
            "attack_magic": item.get("magic_bonus", 0),
            "attack_ranged": item.get("ranged_bonus", 0),
            "defence_stab": item.get("stab_def", 0),
            "defence_slash": item.get("slash_def", 0),
            "defence_crush": item.get("crush_def", 0),
            "defence_magic": item.get("magic_def", 0),
            "defence_ranged": item.get("ranged_def", 0),
            "melee_strength": item.get("melee_str", 0),
            "ranged_strength": item.get("ranged_str", 0),
            "magic_damage": item.get("magic_str", 0),
            "prayer": item.get("prayer_bonus", 0),
            "requirements": requirements if requirements else None,
        },
        # Metadata
        "_source": "gearscape",
        "_gearscape_combat_style": item.get("combat_style"),
    }


def parse_weapon_style(style_str: str) -> dict[str, Any]:
    """
    Parse Gearscape style string like "flick,slash,accurate" into stance dict.
    Format: "combat_style,attack_type,attack_style"
    """
    parts = style_str.split(",")
    if len(parts) != 3:
        return {
            "combat_style": style_str,
            "attack_type": "slash",
            "attack_style": "accurate",
            "experience": "attack",
            "boosts": None,
        }

    combat_style, attack_type, attack_style = parts

    # Map attack_style to experience type
    experience_map = {
        "accurate": "attack",
        "aggressive": "strength",
        "defensive": "defence",
        "controlled": "shared",
        "rapid": "ranged",
        "longrange": "ranged and defence",
    }

    return {
        "combat_style": combat_style,
        "attack_type": attack_type,
        "attack_style": attack_style,
        "experience": experience_map.get(attack_style, "attack"),
        "boosts": None,
    }


def transform_weapon(item: dict[str, Any]) -> dict[str, Any]:
    """Transform Gearscape weapon item to OSRSBox-compatible format."""
    # Build requirements dict
    requirements = {}
    req_fields = [
        ("attack_req", "attack"),
        ("strength_req", "strength"),
        ("defence_req", "defence"),
        ("ranged_req", "ranged"),
        ("magic_req", "magic"),
        ("prayer_req", "prayer"),
        ("hitpoints_req", "hitpoints"),
        ("slayer_req", "slayer"),
    ]
    for gs_field, our_field in req_fields:
        val = item.get(gs_field, 1)
        if val and val > 1:
            requirements[our_field] = val

    # Parse stances
    stances = []
    for style_str in item.get("styles", []):
        stances.append(parse_weapon_style(style_str))

    # Determine weapon type from subcategory
    weapon_type = item.get("subcategory", "unknown")
    # Convert to snake_case format used by OSRSBox
    weapon_type = weapon_type.replace(" ", "_").replace("-", "_").lower()

    return {
        "id": item["id"],
        "name": item["name"],
        "equipable": True,
        "equipable_by_player": True,
        "equipable_weapon": True,
        "members": item.get("members", True),
        "tradeable": item.get("tradeable", True),
        "weight": item.get("weight", 0),
        "equipment": {
            "slot": "weapon",
            "attack_stab": item.get("stab_bonus", 0),
            "attack_slash": item.get("slash_bonus", 0),
            "attack_crush": item.get("crush_bonus", 0),
            "attack_magic": item.get("magic_bonus", 0),
            "attack_ranged": item.get("ranged_bonus", 0),
            "defence_stab": item.get("stab_def", 0),
            "defence_slash": item.get("slash_def", 0),
            "defence_crush": item.get("crush_def", 0),
            "defence_magic": item.get("magic_def", 0),
            "defence_ranged": item.get("ranged_def", 0),
            "melee_strength": item.get("melee_str", 0),
            "ranged_strength": item.get("ranged_str", 0),
            "magic_damage": item.get("magic_str", 0),
            "prayer": item.get("prayer_bonus", 0),
            "requirements": requirements if requirements else None,
        },
        "weapon": {
            "attack_speed": item.get("attack_speed", 4),
            "weapon_type": weapon_type,
            "stances": stances,
        },
        # Metadata
        "_source": "gearscape",
        "_gearscape_combat_style": item.get("combat_style"),
        "_gearscape_two_handed": item.get("two_handed", False),
        "_gearscape_ammunition": item.get("ammunition", []),
    }


def file_hash(path: Path) -> str | None:
    """Get MD5 hash of file contents, or None if file doesn't exist."""
    if not path.exists():
        return None
    return hashlib.md5(path.read_bytes()).hexdigest()


def write_if_changed(path: Path, content: str) -> bool:
    """Write content to file only if it differs from existing. Returns True if written."""
    new_hash = hashlib.md5(content.encode("utf-8")).hexdigest()
    if file_hash(path) == new_hash:
        return False
    path.write_text(content)
    return True


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    out_dir = root / "data" / "items"
    out_dir.mkdir(parents=True, exist_ok=True)

    # Track stats
    stats = {
        "equipment_fetched": 0,
        "weapons_fetched": 0,
        "written": 0,
        "unchanged": 0,
        "errors": 0,
    }

    # Fetch equipment
    equipment_data = fetch_json(EQUIPMENT_URL)
    equipment_by_slot = equipment_data.get("equipment", {})

    for slot, items in equipment_by_slot.items():
        for item in items:
            stats["equipment_fetched"] += 1
            try:
                transformed = transform_equipment(item, slot)
                content = json.dumps(transformed, indent=2, sort_keys=True)
                path = out_dir / f"{item['id']}.json"
                if write_if_changed(path, content):
                    stats["written"] += 1
                else:
                    stats["unchanged"] += 1
            except Exception as e:
                print(f"Error transforming equipment {item.get('id')}: {e}", file=sys.stderr)
                stats["errors"] += 1

    # Fetch weapons
    weapon_data = fetch_json(WEAPON_URL)
    weapons = weapon_data.get("weapons", [])

    for item in weapons:
        stats["weapons_fetched"] += 1
        try:
            transformed = transform_weapon(item)
            content = json.dumps(transformed, indent=2, sort_keys=True)
            path = out_dir / f"{item['id']}.json"
            if write_if_changed(path, content):
                stats["written"] += 1
            else:
                stats["unchanged"] += 1
        except Exception as e:
            print(f"Error transforming weapon {item.get('id')}: {e}", file=sys.stderr)
            stats["errors"] += 1

    # Report
    print()
    print("=== Gearscape Ingestion Complete ===")
    print(f"Equipment fetched: {stats['equipment_fetched']}")
    print(f"Weapons fetched:   {stats['weapons_fetched']}")
    print(f"Files written:     {stats['written']}")
    print(f"Files unchanged:   {stats['unchanged']}")
    if stats["errors"]:
        print(f"Errors:            {stats['errors']}")

    return 1 if stats["errors"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
