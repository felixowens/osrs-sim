#!/usr/bin/env python3
"""Fetch whitelisted OSRSBox-style item JSON and save locally."""

from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, List, Optional, Tuple


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Ingest whitelisted item IDs from an OSRSBox-style JSON source",
    )
    parser.add_argument(
        "--whitelist",
        required=True,
        help="Path to whitelist (JSON array/object or newline-delimited text)",
    )
    parser.add_argument(
        "--base-url",
        required=True,
        help="Base URL for item JSON files (ending with items-json)",
    )
    parser.add_argument(
        "--out-dir",
        default="data/items",
        help="Output directory for per-item JSON files (default: data/items)",
    )
    parser.add_argument(
        "--allow-missing",
        action="store_true",
        help="Allow missing item IDs instead of failing",
    )
    return parser.parse_args()


def load_whitelist(path: Path) -> List[int]:
    if path.suffix.lower() == ".json":
        data = json.loads(path.read_text())
        if isinstance(data, dict):
            if "ids" in data:
                ids = data["ids"]
            else:
                raise ValueError("Expected whitelist JSON to contain an 'ids' array")
        else:
            ids = data
        return normalize_ids(ids)

    ids: List[str] = []
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        ids.append(line)
    return normalize_ids(ids)


def normalize_ids(raw: Any) -> List[int]:
    if not isinstance(raw, list):
        raise ValueError("Whitelist must be a JSON array or newline-delimited text")
    ids: List[int] = []
    for value in raw:
        if isinstance(value, bool):
            raise ValueError("Whitelist entries must be integers or strings")
        if isinstance(value, int):
            ids.append(int(value))
            continue
        if isinstance(value, str):
            value = value.strip()
            if not value:
                continue
            ids.append(int(value, 10))
            continue
        raise ValueError("Whitelist entries must be integers or strings")
    return ids


def normalize_base_url(base_url: str) -> str:
    return base_url.rstrip("/")


def fetch_item(base_url: str, item_id: int) -> Tuple[Optional[dict], Optional[str]]:
    url = f"{base_url}/{item_id}.json"
    try:
        with urllib.request.urlopen(url, timeout=15) as response:
            payload = response.read().decode("utf-8")
    except urllib.error.HTTPError as exc:
        return None, f"HTTP {exc.code} for {url}"
    except urllib.error.URLError as exc:
        return None, f"URL error for {url}: {exc.reason}"

    try:
        data = json.loads(payload)
    except json.JSONDecodeError as exc:
        return None, f"Invalid JSON for {url}: {exc}"

    if not isinstance(data, dict):
        return None, f"Unexpected JSON structure for {url}"

    return data, None


def main() -> int:
    args = parse_args()
    whitelist_path = Path(args.whitelist)
    out_dir = Path(args.out_dir)
    base_url = normalize_base_url(args.base_url)

    whitelist = load_whitelist(whitelist_path)
    if not whitelist:
        raise ValueError("Whitelist is empty")

    out_dir.mkdir(parents=True, exist_ok=True)

    missing_ids: List[int] = []
    warnings: List[str] = []

    for item_id in sorted(set(whitelist)):
        item, warning = fetch_item(base_url, item_id)
        if warning:
            warnings.append(warning)
        if item is None:
            missing_ids.append(item_id)
            continue

        item_id_in_data = item.get("id", item.get("item_id"))
        if item_id_in_data is not None:
            try:
                if int(item_id_in_data) != item_id:
                    warnings.append(
                        f"Item ID mismatch for {item_id}: payload has {item_id_in_data}"
                    )
            except (TypeError, ValueError):
                warnings.append(
                    f"Item ID mismatch for {item_id}: payload has {item_id_in_data}"
                )

        filename = f"{item_id}.json"
        (out_dir / filename).write_text(json.dumps(item, indent=2, sort_keys=True))

    if warnings:
        for warning in warnings:
            print(f"warning: {warning}", file=sys.stderr)

    message = f"Wrote {len(whitelist) - len(missing_ids)} items to {out_dir}"
    if missing_ids:
        message += f" (missing {len(missing_ids)} IDs)"
    print(message, file=sys.stderr)

    if missing_ids and not args.allow_missing:
        return 1
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:  # noqa: BLE001 - CLI entry point
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
