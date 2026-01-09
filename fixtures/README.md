# Fixtures

Reusable test fixtures for the OSRS DPS simulator.

## Directory Structure

```
fixtures/
├── players/     # Player configurations (stats, prayers, potions)
├── builds/      # Equipment setups (gear, combat style)
└── targets/     # Monster/target definitions
```

## Adding a Golden Test

1. Add/reuse fixtures in `players/`, `builds/`, `targets/`
2. Add test to `tests/golden.rs`:

```rust
#[test]
fn descriptive_test_name() {
    let result = eval_from_fixtures(
        "players/maxed_super_str.json",
        "builds/voidwaker_bandos_slash_accurate.json",
        "targets/tztok_jad.json",
    );

    let expected = Expected {
        dps: 1.585,
        max_hit: 30,
        accuracy: 0.2531,
    };

    assert_result_matches(&result, &expected, "descriptive_test_name");
}
```

## Available Fixtures

### Players

- `maxed_no_boosts.json` - 99 stats, no prayers/potions
- `maxed_super_str.json` - 99 stats, super strength potion
- `maxed_piety_super_combat.json` - 99 stats, Piety + Super Combat

### Builds

- `voidwaker_bandos_slash_accurate.json` - Voidwaker + Bandos, slash/accurate

### Targets

- `dummy.json` - Training dummy (1 def, 0 bonuses)
- `tztok_jad.json` - TzTok-Jad

## Running Tests

```bash
cargo test --test golden
```
