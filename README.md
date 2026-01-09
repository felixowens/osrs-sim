# osrs-sim

An Old School RuneScape DPS simulator and gear optimizer, inspired by [Gearscape's Best Setup calculator](https://gearscape.net/calculators/best/).

## Project Goal

Build a deterministic, extensible combat math engine that can:

1. Calculate DPS for any player/equipment/target combination
2. Find optimal gear setups via intelligent search (future)
3. Support multi-phase boss encounters (future)

The architecture follows a three-layer design:

- **Data Layer**: Items, monsters, prayers, potions from OSRS Wiki/OSRSBox
- **Rules Engine**: Data-driven effects system for special items (Slayer helm, Salve, Void, etc.)
- **Optimizer**: Gear combination search with pruning strategies (future)

## Current Status

**Implemented:**

- Melee DPS calculation with all core formulas (effective levels, max hit, accuracy, DPS)
- CLI with `eval`, `validate-data`, `item-info`, `monster-info` commands
- Data loading from ingested OSRS Wiki JSON
- Player state resolution (prayers, potions, stances)
- Golden tests for formula verification

**Not yet implemented:**

- Ranged and magic combat styles
- Data-driven effects system (Slayer helm, Salve amulet, Void, etc.)
- Gear optimizer / best-in-slot finder
- Multi-phase boss support

## Usage

```bash
# Evaluate DPS for a setup
cargo run -- eval \
  --player fixtures/players/maxed_piety_super_combat.json \
  --build fixtures/builds/voidwaker_bandos_slash_aggressive.json \
  --target fixtures/targets/tztok_jad.json

# With detailed breakdown
cargo run -- eval --player ... --build ... --target ... --explain

# Output as JSON
cargo run -- eval --player ... --build ... --target ... --json

# Inspect item data
cargo run -- item-info 27690  # Voidwaker

# Inspect monster data
cargo run -- monster-info 3127  # TzTok-Jad
```

## Layout

- `src/lib.rs`: library entry point
- `src/main.rs`: CLI (clap-based)
- `src/model.rs`: core types (player, build, target, results)
- `src/formulas.rs`: pure combat math functions
- `src/effects.rs`: effect definitions and application (placeholder)
- `src/data.rs`: data loading and resolution
- `scripts/`: Python scripts for data ingestion from OSRS Wiki
- `data/`: ingested items and monsters (see `data/README.md`)
- `fixtures/`: test inputs (players, builds, targets)

## Development

```bash
# Format, lint, and test
./scripts/check.sh

# Run tests only
cargo test

# Build release
cargo build --release
```

## References

- [Bitterkoekje's DPS Calculator](https://docs.google.com/spreadsheets/d/1wzy1VxNWEAAc0FQyDAdpiFggAfn5U6RGPp2CisAHZW8) - gold standard for formula verification
- [OSRS Wiki DPS Calculator](https://github.com/weirdgloop/osrs-dps-calc) - open source reference
- [OSRSBox](https://www.osrsbox.com/) - item and monster data source
