// src/formulas.rs
//
// Core combat math for melee DPS calculation.
// All formulas use integer math with explicit floors where specified.

use crate::model::{EvalContext, EvalResult};

// =============================================================================
// Core Formula Functions (pure, testable)
// =============================================================================

/// Calculate effective attack level for melee.
/// Formula: floor(floor((base + potion) * prayer) + style_bonus + 8)
pub fn effective_attack_level(
    base_level: i32,
    potion_boost: i32,
    prayer_mult: (i32, i32), // (numerator, denominator)
    style_bonus: i32,
) -> i32 {
    let boosted = base_level + potion_boost;
    let prayed = (boosted * prayer_mult.0) / prayer_mult.1; // floor via integer division
    prayed + style_bonus + 8
}

/// Calculate effective strength level for melee.
/// Same formula structure as attack.
pub fn effective_strength_level(
    base_level: i32,
    potion_boost: i32,
    prayer_mult: (i32, i32),
    style_bonus: i32,
) -> i32 {
    let boosted = base_level + potion_boost;
    let prayed = (boosted * prayer_mult.0) / prayer_mult.1;
    prayed + style_bonus + 8
}

/// Calculate effective defence level (for target's defence roll).
/// For monsters, this is just their defence level + 9 (no style bonus).
pub fn effective_defence_level(defence_level: i32) -> i32 {
    defence_level + 9
}

/// Calculate max attack roll.
/// Formula: effective_attack * (equipment_bonus + 64)
pub fn max_attack_roll(effective_attack: i32, equipment_bonus: i32) -> i32 {
    effective_attack * (equipment_bonus + 64)
}

/// Calculate max defence roll.
/// Formula: effective_defence * (defence_bonus + 64)
pub fn max_defence_roll(effective_defence: i32, defence_bonus: i32) -> i32 {
    effective_defence * (defence_bonus + 64)
}

/// Calculate hit chance (accuracy).
/// Returns a value between 0.0 and 1.0.
///
/// Formula:
///   If A > D: accuracy = 1 - (D + 2) / (2 * (A + 1))
///   Else:     accuracy = A / (2 * (D + 1))
pub fn hit_chance(attack_roll: i32, defence_roll: i32) -> f64 {
    let a = attack_roll as f64;
    let d = defence_roll as f64;

    if attack_roll > defence_roll {
        1.0 - (d + 2.0) / (2.0 * (a + 1.0))
    } else {
        a / (2.0 * (d + 1.0))
    }
}

/// Calculate max hit for melee.
/// Formula: floor(0.5 + effective_strength * (str_bonus + 64) / 640)
///
/// Using integer math: floor((effective * (bonus + 64) + 320) / 640)
pub fn max_hit_melee(effective_strength: i32, str_bonus: i32) -> i32 {
    // The +320 is equivalent to +0.5 when dividing by 640
    (effective_strength * (str_bonus + 64) + 320) / 640
}

/// Calculate DPS (damage per second).
/// Formula: hit_chance * (max_hit / 2) / (interval_ticks * 0.6)
///
/// Average hit when you hit = max_hit / 2 (uniform distribution 0 to max_hit)
pub fn calculate_dps(accuracy: f64, max_hit: i32, interval_ticks: u8) -> f64 {
    if interval_ticks == 0 {
        return 0.0;
    }

    let avg_hit = max_hit as f64 / 2.0;
    let interval_seconds = interval_ticks as f64 * 0.6;

    accuracy * avg_hit / interval_seconds
}

// =============================================================================
// Main Evaluation Function
// =============================================================================

/// Evaluate DPS for a given context (player + build + target).
pub fn evaluate(ctx: &EvalContext) -> EvalResult {
    let player = ctx.player;
    let build = ctx.build;
    let target = ctx.target;

    // Get stance bonuses for melee
    let (atk_style_bonus, str_style_bonus) = build.stance.melee_bonuses();

    // Calculate effective levels
    let eff_attack = effective_attack_level(
        player.attack as i32,
        player.potion_attack_boost,
        player.prayer_attack_mult,
        atk_style_bonus,
    );

    let eff_strength = effective_strength_level(
        player.strength as i32,
        player.potion_strength_boost,
        player.prayer_strength_mult,
        str_style_bonus,
    );

    let eff_defence = effective_defence_level(target.defence_level as i32);

    // Get equipment bonuses for the attack type
    let attack_bonus = build.bonuses.attack_bonus_for(build.attack_type);
    let defence_bonus = target.defence_bonuses.defence_bonus_for(build.attack_type);

    // Calculate rolls
    let atk_roll = max_attack_roll(eff_attack, attack_bonus);
    let def_roll = max_defence_roll(eff_defence, defence_bonus);

    // Calculate accuracy
    let accuracy = hit_chance(atk_roll, def_roll);

    // Calculate max hit
    let max_hit = max_hit_melee(eff_strength, build.bonuses.melee_strength);

    // Calculate DPS
    let dps = calculate_dps(accuracy, max_hit, build.attack_speed);

    EvalResult {
        dps,
        max_hit: max_hit as u32,
        accuracy,
        attack_roll: atk_roll as u32,
        defence_roll: def_roll as u32,
        interval_ticks: build.attack_speed,
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_attack_level_no_boosts() {
        // 99 attack, no potion, no prayer (1/1), accurate stance (+3)
        let result = effective_attack_level(99, 0, (1, 1), 3);
        // 99 + 0 = 99, * 1/1 = 99, + 3 + 8 = 110
        assert_eq!(result, 110);
    }

    #[test]
    fn test_effective_attack_level_with_piety() {
        // 99 attack, no potion, piety (6/5 = 1.2), accurate stance (+3)
        let result = effective_attack_level(99, 0, (6, 5), 3);
        // 99 * 6/5 = 118 (floor of 118.8), + 3 + 8 = 129
        assert_eq!(result, 129);
    }

    #[test]
    fn test_effective_attack_level_with_super_combat_and_piety() {
        // 99 attack, super combat (+19), piety (6/5), accurate (+3)
        let result = effective_attack_level(99, 19, (6, 5), 3);
        // (99 + 19) * 6/5 = 141 (floor of 141.6), + 3 + 8 = 152
        assert_eq!(result, 152);
    }

    #[test]
    fn test_max_hit_basic() {
        // Effective strength 118, str bonus 0
        let result = max_hit_melee(118, 0);
        // (118 * 64 + 320) / 640 = 7872 / 640 = 12.3 -> 12
        assert_eq!(result, 12);
    }

    #[test]
    fn test_max_hit_with_str_bonus() {
        // Effective strength 118, str bonus 100
        let result = max_hit_melee(118, 100);
        // (118 * 164 + 320) / 640 = 19672 / 640 = 30.7 -> 30
        assert_eq!(result, 30);
    }

    #[test]
    fn test_hit_chance_attacker_advantage() {
        // Attack roll > defence roll
        let result = hit_chance(20000, 10000);
        // 1 - (10002) / (2 * 20001) = 1 - 10002/40002 ≈ 0.75
        assert!(result > 0.74 && result < 0.76);
    }

    #[test]
    fn test_hit_chance_defender_advantage() {
        // Attack roll < defence roll
        let result = hit_chance(10000, 20000);
        // 10000 / (2 * 20001) ≈ 0.25
        assert!(result > 0.24 && result < 0.26);
    }

    #[test]
    fn test_hit_chance_equal_rolls() {
        // Equal rolls - should be exactly 50%
        let result = hit_chance(10000, 10000);
        // 10000 / (2 * 10001) ≈ 0.4999
        assert!(result > 0.49 && result < 0.51);
    }

    #[test]
    fn test_dps_calculation() {
        // 50% accuracy, max hit 30, 4 tick weapon
        let result = calculate_dps(0.5, 30, 4);
        // 0.5 * 15 / 2.4 = 7.5 / 2.4 = 3.125
        assert!((result - 3.125).abs() < 0.001);
    }
}
