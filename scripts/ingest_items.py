#!/usr/bin/env python3
"""Backward-compatible wrapper for ingest_entities.py (items)."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    cmd = [
        sys.executable,
        str(root / "scripts" / "ingest_entities.py"),
    ]
    cmd.extend(sys.argv[1:])
    cmd.extend(["--kind", "item"])
    return subprocess.call(cmd)


if __name__ == "__main__":
    raise SystemExit(main())
