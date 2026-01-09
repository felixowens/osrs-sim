// src/model.rs
//
// Core types for the OSRS DPS simulator.
// Split into:
//   - Input structs (for JSON parsing from fixtures)
//   - Internal/resolved structs (for the engine)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bump this when you make a breaking change to input formats.
pub const SCHEMA_VERSION_V1: u32 = 1;

// =============================================================================
// Input Structs (JSON parsing)
// =============================================================================

// -----------------------------
// Player Input
// -----------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerInput {
    pub schema_version: u32,
    pub skills: SkillsInput,

    /// Map of prayer_key -> enabled (e.g. "piety": true).
    #[serde(default)]
    pub prayers: HashMap<String, bool>,

    /// Optional boosts; omitted means "no boost".
    #[serde(default)]
    pub boosts: BoostsInput,

    /// Context flags (task/wilderness/etc).
    #[serde(default)]
    pub flags: FlagsInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillsInput {
    pub attack: u8,
    pub strength: u8,
    pub defence: u8,
    pub ranged: u8,
    pub magic: u8,
    pub prayer: u8,
    pub hitpoints: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoostsInput {
    #[serde(default)]
    pub melee: Option<BoostSpec>,
    #[serde(default)]
    pub ranged: Option<BoostSpec>,
    #[serde(default)]
    pub magic: Option<BoostSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoostSpec {
    /// JSON uses field name "type"; Rust uses "kind".
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlagsInput {
    #[serde(default)]
    pub on_slayer_task: bool,
    #[serde(default)]
    pub in_wilderness: bool,
}

// -----------------------------
// Build Input
// -----------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildInput {
    pub schema_version: u32,
    pub equipment: EquipmentInput,
    pub style: StyleInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentInput {
    /// Required for MVP.
    pub weapon: u32,

    #[serde(default)]
    pub head: Option<u32>,
    #[serde(default)]
    pub cape: Option<u32>,
    #[serde(default)]
    pub neck: Option<u32>,
    #[serde(default)]
    pub ammo: Option<u32>,
    #[serde(default)]
    pub body: Option<u32>,
    #[serde(default)]
    pub shield: Option<u32>,
    #[serde(default)]
    pub legs: Option<u32>,
    #[serde(default)]
    pub hands: Option<u32>,
    #[serde(default)]
    pub feet: Option<u32>,
    #[serde(default)]
    pub ring: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleInput {
    pub combat: Combat,
    pub attack_type: AttackType,
    pub stance: Stance,

    /// Off by default.
    #[serde(default)]
    pub special_attack: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Combat {
    Melee,
    Ranged,
    Magic,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackType {
    Stab,
    Slash,
    Crush,
    Ranged,
    Magic,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stance {
    Accurate,
    Aggressive,
    Defensive,
    Controlled,
    Rapid,
    Longrange,
}

// -----------------------------
// Target Input
// -----------------------------

/// Matches "oneOf": either {monster_id, overrides?} or {custom}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TargetInput {
    ById(TargetByIdInput),
    Custom(TargetCustomInput),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetByIdInput {
    pub schema_version: u32,
    pub monster_id: u32,

    #[serde(default)]
    pub overrides: Option<TargetOverrides>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetCustomInput {
    pub schema_version: u32,
    pub custom: CustomTarget,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetOverrides {
    #[serde(default)]
    pub attributes_add: Vec<String>,
    #[serde(default)]
    pub attributes_remove: Vec<String>,

    #[serde(default)]
    pub attack_level: Option<u16>,
    #[serde(default)]
    pub strength_level: Option<u16>,
    #[serde(default)]
    pub defence_level: Option<u16>,
    #[serde(default)]
    pub magic_level: Option<u16>,
    #[serde(default)]
    pub ranged_level: Option<u16>,
    #[serde(default)]
    pub hitpoints_level: Option<u16>,

    #[serde(default)]
    pub defence_bonuses: Option<DefenceBonusesPartial>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefenceBonusesPartial {
    #[serde(default)]
    pub stab: Option<i32>,
    #[serde(default)]
    pub slash: Option<i32>,
    #[serde(default)]
    pub crush: Option<i32>,
    #[serde(default)]
    pub magic: Option<i32>,
    #[serde(default)]
    pub ranged: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomTarget {
    pub name: String,
    pub levels: CustomTargetLevels,
    pub defence_bonuses: DefenceBonuses,
    #[serde(default)]
    pub attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomTargetLevels {
    pub defence: u16,
    pub hitpoints: u16,

    #[serde(default)]
    pub attack: Option<u16>,
    #[serde(default)]
    pub strength: Option<u16>,
    #[serde(default)]
    pub magic: Option<u16>,
    #[serde(default)]
    pub ranged: Option<u16>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefenceBonuses {
    pub stab: i32,
    pub slash: i32,
    pub crush: i32,
    pub magic: i32,
    pub ranged: i32,
}

// =============================================================================
// Internal/Resolved Structs (for the engine)
// =============================================================================

/// Resolved player state with all levels and active effects computed.
#[derive(Debug, Clone)]
pub struct PlayerState {
    /// Base skill levels (1-99)
    pub attack: u8,
    pub strength: u8,
    pub defence: u8,
    pub ranged: u8,
    pub magic: u8,
    pub prayer: u8,
    pub hitpoints: u8,

    /// Active prayer multipliers (num/den for exact math)
    pub prayer_attack_mult: (i32, i32),
    pub prayer_strength_mult: (i32, i32),
    pub prayer_defence_mult: (i32, i32),

    /// Potion boosts (flat + percentage already computed to flat)
    pub potion_attack_boost: i32,
    pub potion_strength_boost: i32,
    pub potion_defence_boost: i32,

    /// Context flags
    pub on_slayer_task: bool,
    pub in_wilderness: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            attack: 99,
            strength: 99,
            defence: 99,
            ranged: 99,
            magic: 99,
            prayer: 99,
            hitpoints: 99,
            prayer_attack_mult: (1, 1),
            prayer_strength_mult: (1, 1),
            prayer_defence_mult: (1, 1),
            potion_attack_boost: 0,
            potion_strength_boost: 0,
            potion_defence_boost: 0,
            on_slayer_task: false,
            in_wilderness: false,
        }
    }
}

/// Aggregated equipment bonuses (sum of all equipped items).
#[derive(Debug, Clone, Copy, Default)]
pub struct EquipmentBonuses {
    // Attack bonuses
    pub attack_stab: i32,
    pub attack_slash: i32,
    pub attack_crush: i32,
    pub attack_magic: i32,
    pub attack_ranged: i32,

    // Defence bonuses
    pub defence_stab: i32,
    pub defence_slash: i32,
    pub defence_crush: i32,
    pub defence_magic: i32,
    pub defence_ranged: i32,

    // Other bonuses
    pub melee_strength: i32,
    pub ranged_strength: i32,
    pub magic_damage: i32, // percentage points
    pub prayer: i32,
}

/// Resolved build with aggregated stats.
#[derive(Debug, Clone)]
pub struct BuildResolved {
    pub bonuses: EquipmentBonuses,
    pub attack_speed: u8, // in game ticks
    pub combat: Combat,
    pub attack_type: AttackType,
    pub stance: Stance,
}

impl Default for BuildResolved {
    fn default() -> Self {
        Self {
            bonuses: EquipmentBonuses::default(),
            attack_speed: 4,
            combat: Combat::Melee,
            attack_type: AttackType::Slash,
            stance: Stance::Accurate,
        }
    }
}

/// Resolved target (monster) with all stats.
#[derive(Debug, Clone)]
pub struct TargetResolved {
    pub name: String,
    pub hitpoints: u16,
    pub defence_level: u16,
    pub defence_bonuses: DefenceBonuses,
    pub attributes: Vec<String>,
}

impl Default for TargetResolved {
    fn default() -> Self {
        Self {
            name: "Dummy".to_string(),
            hitpoints: 100,
            defence_level: 1,
            defence_bonuses: DefenceBonuses::default(),
            attributes: vec![],
        }
    }
}

/// Context passed to the evaluation engine.
pub struct EvalContext<'a> {
    pub player: &'a PlayerState,
    pub build: &'a BuildResolved,
    pub target: &'a TargetResolved,
}

/// Result of DPS evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub dps: f64,
    pub max_hit: u32,
    pub accuracy: f64,
    pub attack_roll: u32,
    pub defence_roll: u32,
    pub interval_ticks: u8,
}

impl Default for EvalResult {
    fn default() -> Self {
        Self {
            dps: 0.0,
            max_hit: 0,
            accuracy: 0.0,
            attack_roll: 0,
            defence_roll: 0,
            interval_ticks: 4,
        }
    }
}

// =============================================================================
// Validation Helpers
// =============================================================================

impl PlayerInput {
    pub fn validate_basic(&self) -> Result<(), String> {
        if self.schema_version != SCHEMA_VERSION_V1 {
            return Err(format!("player.schema_version must be {SCHEMA_VERSION_V1}"));
        }
        Ok(())
    }
}

impl BuildInput {
    pub fn validate_basic(&self) -> Result<(), String> {
        if self.schema_version != SCHEMA_VERSION_V1 {
            return Err(format!("build.schema_version must be {SCHEMA_VERSION_V1}"));
        }
        Ok(())
    }
}

impl TargetInput {
    pub fn validate_basic(&self) -> Result<(), String> {
        match self {
            TargetInput::ById(t) => {
                if t.schema_version != SCHEMA_VERSION_V1 {
                    return Err(format!("target.schema_version must be {SCHEMA_VERSION_V1}"));
                }
            }
            TargetInput::Custom(t) => {
                if t.schema_version != SCHEMA_VERSION_V1 {
                    return Err(format!("target.schema_version must be {SCHEMA_VERSION_V1}"));
                }
            }
        }
        Ok(())
    }
}

// =============================================================================
// Stance Bonus Helpers
// =============================================================================

impl Stance {
    /// Returns (attack_bonus, strength_bonus) for melee stances.
    pub fn melee_bonuses(self) -> (i32, i32) {
        match self {
            Stance::Accurate => (3, 0),
            Stance::Aggressive => (0, 3),
            Stance::Defensive => (0, 0),
            Stance::Controlled => (1, 1),
            // Ranged/magic stances - not applicable for melee
            Stance::Rapid | Stance::Longrange => (0, 0),
        }
    }
}
