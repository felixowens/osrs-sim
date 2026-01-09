# Scripts

## ingest_items.py

Filter OSRSBox-style item JSON by a whitelist of item IDs and save locally.

### Usage

```bash
python3 scripts/ingest_items.py \
  --whitelist data/item-ids.json \
  --base-url https://raw.githubusercontent.com/0xNeffarion/osrsreboxed-db/refs/heads/master/docs/items-json \
  --out-dir data/items
```

Whitelist formats:
- JSON array of item IDs, e.g. `[4151, 11865]`.
- JSON object with an `ids` array.
- Newline-delimited text (comments allowed with `#`).

Output format:
- One JSON file per item, named `<item_id>.json` in the output directory.

## ingest_items_default.py

Wrapper that uses repo-local defaults (no args).

### Setup

Edit `data/ingest.json` and set `base_url` if you want a different source.

### Usage

```bash
python3 scripts/ingest_items_default.py
```
