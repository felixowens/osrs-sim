// tests/golden.rs
//
// Golden tests - validated against known DPS calculator outputs

mod common;

use common::{assert_result_matches, eval_from_fixtures, Expected};

/// Voidwaker + Bandos vs TzTok-Jad
/// - 99 all stats, super strength potion, no prayer
/// - Voidwaker (slash, accurate stance)
/// - Bandos chestplate + tassets
/// - Target: TzTok-Jad (480 def, 0 slash def bonus)
#[test]
fn voidwaker_bandos_vs_jad() {
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

    assert_result_matches(&result, &expected, "voidwaker_bandos_vs_jad");
}

/// Voidwaker + Bandos vs TzTok-Jad (no boosts)
/// - 99 all stats, no potions, no prayer
#[test]
fn voidwaker_bandos_vs_jad_no_boosts() {
    let result = eval_from_fixtures(
        "players/maxed_no_boosts.json",
        "builds/voidwaker_bandos_slash_accurate.json",
        "targets/tztok_jad.json",
    );

    let expected = Expected {
        dps: 1.322,
        max_hit: 25,
        accuracy: 0.2531,
    };

    assert_result_matches(&result, &expected, "voidwaker_bandos_vs_jad_no_boosts");
}

/// Voidwaker + Bandos vs TzTok-Jad (aggressive, no boosts)
/// - 99 all stats, no potions, no prayer
/// - Aggressive stance (+3 str, +0 atk)
#[test]
fn voidwaker_bandos_vs_jad_aggressive() {
    let result = eval_from_fixtures(
        "players/maxed_no_boosts.json",
        "builds/voidwaker_bandos_slash_aggressive.json",
        "targets/tztok_jad.json",
    );

    let expected = Expected {
        dps: 1.337,
        max_hit: 26,
        accuracy: 0.2462,
    };

    assert_result_matches(&result, &expected, "voidwaker_bandos_vs_jad_aggressive");
}
