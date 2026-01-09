# osrs-sim

Scaffold for an Old School RuneScape DPS simulator and gear optimizer.

## Goals (MVP)

- Melee-only, single-target DPS evaluation.
- Deterministic math kernel with explicit rounding.
- Data-driven effects (placeholder for now).
- CLI entry point (placeholder for now).

## Layout

- `src/lib.rs`: library entry point.
- `src/main.rs`: CLI entry point.
- `src/model.rs`: core types.
- `src/formulas.rs`: pure math helpers.
- `src/effects.rs`: effect definitions and application.
- `src/data.rs`: data loading and normalization.
- `scripts/`: helper scripts for data ingestion.
- `data/`: local artifacts and datasets (see `data/README.md`).
- `fixtures/`: golden inputs/outputs for tests.

## Checks

Run `scripts/check.sh` for format, lint, and tests.

## Status

Scaffold only; no functional math yet.
