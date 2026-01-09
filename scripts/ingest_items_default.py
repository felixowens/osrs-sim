#!/usr/bin/env python3
"""Run ingest_entities.py with repo-local defaults."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict


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


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    config_path = root / "data" / "ingest.json"

    config = load_config(config_path)
    whitelist_value = config.get("whitelist", "data/item-ids.json")
    if not isinstance(whitelist_value, str) or not whitelist_value.strip():
        print("error: ingest.json has invalid 'whitelist'", file=sys.stderr)
        return 1
    whitelist = resolve_path(root, whitelist_value)

    base_url = config.get("base_url")
    if not isinstance(base_url, str) or not base_url.strip():
        print("error: ingest.json missing 'base_url'", file=sys.stderr)
        return 1

    out_value = config.get("out_dir", "data/items")
    if not isinstance(out_value, str) or not out_value.strip():
        print("error: ingest.json has invalid 'out_dir'", file=sys.stderr)
        return 1

    out_dir = resolve_path(root, out_value)

    cmd = [
        sys.executable,
        str(root / "scripts" / "ingest_entities.py"),
        "--whitelist",
        str(whitelist),
        "--base-url",
        base_url,
        "--out-dir",
        str(out_dir),
    ]

    kind_value = config.get("kind")
    if isinstance(kind_value, str) and kind_value.strip():
        cmd.extend(["--kind", kind_value])

    if config.get("allow_missing") is True:
        cmd.append("--allow-missing")

    return subprocess.call(cmd)


if __name__ == "__main__":
    raise SystemExit(main())
