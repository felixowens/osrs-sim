# Scripts

## ingest_entities.py

Filter OSRSBox-style item/monster JSON by a whitelist of IDs and save locally.

### Usage

```bash
python3 scripts/ingest_entities.py \
  --whitelist data/item-ids.json \
  --base-url https://raw.githubusercontent.com/0xNeffarion/osrsreboxed-db/refs/heads/master/docs/items-json \
  --out-dir data/items
```

Monster usage:

```bash
python3 scripts/ingest_entities.py \
  --whitelist data/monster-ids.json \
  --base-url https://raw.githubusercontent.com/0xNeffarion/osrsreboxed-db/refs/heads/master/docs/monsters-json \
  --out-dir data/monsters \
  --kind monster
```

Whitelist formats:
- JSON array of IDs, e.g. `[4151, 11865]`.
- JSON object with an `ids` array.
- Newline-delimited text (comments allowed with `#`).

Output format:
- One JSON file per ID, named `<id>.json` in the output directory.

## ingest_all.py

Run item and monster ingestion back-to-back using `data/ingest.json`.

### Usage

```bash
python3 scripts/ingest_all.py
```

Reads:
- Items: `base_url`, `out_dir`, `whitelist`, `allow_missing`.
- Monsters: `monsters_base_url`, `monsters_out_dir`, `monsters_whitelist`.

If `monsters_base_url` is omitted and `base_url` ends in `items-json`, it derives
the monsters URL by swapping to `monsters-json`.

## ingest_items.py

Wrapper for item ingestion; delegates to `ingest_entities.py` and forces `--kind item`.

## ingest_items_default.py

Wrapper that uses repo-local defaults (no args).

### Setup

Edit `data/ingest.json` and set `base_url` (and optional `whitelist`/`kind`)
if you want a different source.

### Usage

```bash
python3 scripts/ingest_items_default.py
```
