#!/usr/bin/env python3
"""Run item + monster ingestion with repo-local defaults."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, Optional


def load_config(path: Path) -> Dict[str, Any]:
    try:
        return json.loads(path.read_text())
    except FileNotFoundError:
        print(
            f"error: missing config at {path}. "
            "Create it from data/ingest.json template.",
            file=sys.stderr,
        )
        raise SystemExit(1)
    except json.JSONDecodeError as exc:
        print(f"error: invalid JSON in {path}: {exc}", file=sys.stderr)
        raise SystemExit(1)


def resolve_path(root: Path, value: str) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    return (root / path).resolve()


def normalize_base_url(base_url: str) -> str:
    return base_url.rstrip("/")


def derive_monsters_base_url(items_base_url: str) -> Optional[str]:
    suffix = "/items-json"
    if items_base_url.endswith(suffix):
        return items_base_url[: -len("items-json")] + "monsters-json"
    if items_base_url.endswith("items-json"):
        return items_base_url[: -len("items-json")] + "monsters-json"
    return None


def build_cmd(
    root: Path,
    whitelist: Path,
    base_url: str,
    out_dir: Path,
    kind: str,
    allow_missing: bool,
) -> list[str]:
    cmd = [
        sys.executable,
        str(root / "scripts" / "ingest_entities.py"),
        "--whitelist",
        str(whitelist),
        "--base-url",
        base_url,
        "--out-dir",
        str(out_dir),
        "--kind",
        kind,
    ]
    if allow_missing:
        cmd.append("--allow-missing")
    return cmd


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    config_path = root / "data" / "ingest.json"
    config = load_config(config_path)

    base_url = config.get("base_url")
    if not isinstance(base_url, str) or not base_url.strip():
        print("error: ingest.json missing 'base_url'", file=sys.stderr)
        return 1
    base_url = normalize_base_url(base_url)

    items_whitelist_value = config.get("whitelist", "data/item-ids.json")
    if not isinstance(items_whitelist_value, str) or not items_whitelist_value.strip():
        print("error: ingest.json has invalid 'whitelist'", file=sys.stderr)
        return 1
    items_whitelist = resolve_path(root, items_whitelist_value)

    items_out_value = config.get("out_dir", "data/items")
    if not isinstance(items_out_value, str) or not items_out_value.strip():
        print("error: ingest.json has invalid 'out_dir'", file=sys.stderr)
        return 1
    items_out_dir = resolve_path(root, items_out_value)

    monsters_base_url = config.get("monsters_base_url")
    if isinstance(monsters_base_url, str) and monsters_base_url.strip():
        monsters_base_url = normalize_base_url(monsters_base_url)
    else:
        monsters_base_url = derive_monsters_base_url(base_url)
        if monsters_base_url is None:
            print(
                "error: set ingest.json 'monsters_base_url' "
                "or use an items base_url ending in items-json",
                file=sys.stderr,
            )
            return 1

    monsters_whitelist_value = config.get("monsters_whitelist", "data/monster-ids.json")
    if (
        not isinstance(monsters_whitelist_value, str)
        or not monsters_whitelist_value.strip()
    ):
        print("error: ingest.json has invalid 'monsters_whitelist'", file=sys.stderr)
        return 1
    monsters_whitelist = resolve_path(root, monsters_whitelist_value)

    monsters_out_value = config.get("monsters_out_dir", "data/monsters")
    if not isinstance(monsters_out_value, str) or not monsters_out_value.strip():
        print("error: ingest.json has invalid 'monsters_out_dir'", file=sys.stderr)
        return 1
    monsters_out_dir = resolve_path(root, monsters_out_value)

    allow_missing = config.get("allow_missing") is True

    items_cmd = build_cmd(
        root,
        items_whitelist,
        base_url,
        items_out_dir,
        "item",
        allow_missing,
    )
    monsters_cmd = build_cmd(
        root,
        monsters_whitelist,
        monsters_base_url,
        monsters_out_dir,
        "monster",
        allow_missing,
    )

    result = subprocess.call(items_cmd)
    if result != 0:
        return result
    return subprocess.call(monsters_cmd)


if __name__ == "__main__":
    raise SystemExit(main())
