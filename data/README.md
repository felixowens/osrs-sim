# Data

Local artifacts and datasets will live here.

Planned sources: OSRSBox updated fork

- <https://github.com/0xNeffarion/osrsreboxed-db/tree/master/docs/items-json>
- <https://github.com/0xNeffarion/osrsreboxed-db/tree/master/docs/monsters-json>

Defaults:

- `item-ids.json`: whitelist for ingestion.
- `monster-ids.json`: whitelist for monster ingestion (create as needed).
- `ingest.json`: base URL/output settings for `scripts/ingest_items_default.py` and
  `scripts/ingest_all.py` (optional `whitelist`, `kind`, and `monsters_*` overrides).
