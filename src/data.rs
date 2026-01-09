// src/data.rs
//
// Data loading from the data/ folder.
// Structs match the JSON format from osrsbox/wiki.

use crate::model::{
    AttackType, BuildResolved, DefenceBonuses, EquipmentBonuses, PlayerState, Stance,
    TargetResolved,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Item not found: {0}")]
    ItemNotFound(u32),
    #[error("Monster not found: {0}")]
    MonsterNotFound(u32),
}

// =============================================================================
// Item Data Structures (matching data/items/*.json)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemData {
    pub id: u32,
    pub name: String,
    pub equipable: bool,
    pub equipable_weapon: bool,

    #[serde(default)]
    pub equipment: Option<EquipmentData>,

    #[serde(default)]
    pub weapon: Option<WeaponData>,

    // Other fields we don't need for MVP
    #[serde(default)]
    pub members: bool,
    #[serde(default)]
    pub tradeable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentData {
    pub slot: String,

    // Attack bonuses
    #[serde(default)]
    pub attack_stab: i32,
    #[serde(default)]
    pub attack_slash: i32,
    #[serde(default)]
    pub attack_crush: i32,
    #[serde(default)]
    pub attack_magic: i32,
    #[serde(default)]
    pub attack_ranged: i32,

    // Defence bonuses
    #[serde(default)]
    pub defence_stab: i32,
    #[serde(default)]
    pub defence_slash: i32,
    #[serde(default)]
    pub defence_crush: i32,
    #[serde(default)]
    pub defence_magic: i32,
    #[serde(default)]
    pub defence_ranged: i32,

    // Other bonuses
    #[serde(default)]
    pub melee_strength: i32,
    #[serde(default)]
    pub ranged_strength: i32,
    #[serde(default)]
    pub magic_damage: i32,
    #[serde(default)]
    pub prayer: i32,

    // Requirements (optional)
    #[serde(default)]
    pub requirements: Option<HashMap<String, u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponData {
    pub attack_speed: u8,
    pub weapon_type: String,

    #[serde(default)]
    pub stances: Vec<StanceData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StanceData {
    pub attack_style: String,
    pub attack_type: String,
    pub combat_style: String,
    pub experience: String,

    #[serde(default)]
    pub boosts: Option<String>,
}

// =============================================================================
// Monster Data Structures (matching data/monsters/*.json)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterData {
    pub id: u32,
    pub name: String,
    pub hitpoints: u16,

    // Combat levels
    #[serde(default)]
    pub attack_level: u16,
    #[serde(default)]
    pub strength_level: u16,
    pub defence_level: u16,
    #[serde(default)]
    pub magic_level: u16,
    #[serde(default)]
    pub ranged_level: u16,

    // Defence bonuses
    #[serde(default)]
    pub defence_stab: i32,
    #[serde(default)]
    pub defence_slash: i32,
    #[serde(default)]
    pub defence_crush: i32,
    #[serde(default)]
    pub defence_magic: i32,
    #[serde(default)]
    pub defence_ranged: i32,

    // Attributes and categories
    #[serde(default)]
    pub attributes: Vec<String>,
    #[serde(default)]
    pub category: Vec<String>,

    // Slayer info
    #[serde(default)]
    pub slayer_monster: bool,
    #[serde(default)]
    pub slayer_level: u8,

    // Other
    #[serde(default)]
    pub combat_level: u16,
    #[serde(default)]
    pub attack_speed: u8,
    #[serde(default)]
    pub max_hit: u16,
    #[serde(default)]
    pub size: u8,
}

// =============================================================================
// Data Store
// =============================================================================

#[derive(Debug, Clone, Default)]
pub struct DataStore {
    pub items: HashMap<u32, ItemData>,
    pub monsters: HashMap<u32, MonsterData>,
}

impl DataStore {
    /// Create an empty data store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all data from the given data directory.
    pub fn load_from_dir(data_dir: &Path) -> Result<Self, DataError> {
        let mut store = Self::new();
        store.load_items(data_dir)?;
        store.load_monsters(data_dir)?;
        Ok(store)
    }

    /// Load all items from data/items/*.json
    fn load_items(&mut self, data_dir: &Path) -> Result<(), DataError> {
        let items_dir = data_dir.join("items");
        if !items_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(items_dir)? {
            let entry = entry?;
            let path = entry.path();
            // Skip files starting with underscore (like _index.json)
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if filename.starts_with('_') {
                continue;
            }
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let content = fs::read_to_string(&path)?;
                let item: ItemData = serde_json::from_str(&content)?;
                self.items.insert(item.id, item);
            }
        }
        Ok(())
    }

    /// Load all monsters from data/monsters/*.json
    fn load_monsters(&mut self, data_dir: &Path) -> Result<(), DataError> {
        let monsters_dir = data_dir.join("monsters");
        if !monsters_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(monsters_dir)? {
            let entry = entry?;
            let path = entry.path();
            // Skip files starting with underscore (like _index.json)
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if filename.starts_with('_') {
                continue;
            }
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let content = fs::read_to_string(&path)?;
                let monster: MonsterData = serde_json::from_str(&content)?;
                self.monsters.insert(monster.id, monster);
            }
        }
        Ok(())
    }

    /// Get an item by ID.
    pub fn get_item(&self, id: u32) -> Result<&ItemData, DataError> {
        self.items.get(&id).ok_or(DataError::ItemNotFound(id))
    }

    /// Get a monster by ID.
    pub fn get_monster(&self, id: u32) -> Result<&MonsterData, DataError> {
        self.monsters.get(&id).ok_or(DataError::MonsterNotFound(id))
    }
}

// =============================================================================
// Resolution: Input -> Resolved structs
// =============================================================================

impl ItemData {
    /// Get equipment bonuses from this item.
    pub fn get_bonuses(&self) -> EquipmentBonuses {
        match &self.equipment {
            Some(eq) => EquipmentBonuses {
                attack_stab: eq.attack_stab,
                attack_slash: eq.attack_slash,
                attack_crush: eq.attack_crush,
                attack_magic: eq.attack_magic,
                attack_ranged: eq.attack_ranged,
                defence_stab: eq.defence_stab,
                defence_slash: eq.defence_slash,
                defence_crush: eq.defence_crush,
                defence_magic: eq.defence_magic,
                defence_ranged: eq.defence_ranged,
                melee_strength: eq.melee_strength,
                ranged_strength: eq.ranged_strength,
                magic_damage: eq.magic_damage,
                prayer: eq.prayer,
            },
            None => EquipmentBonuses::default(),
        }
    }

    /// Get attack speed (only for weapons).
    pub fn get_attack_speed(&self) -> u8 {
        self.weapon.as_ref().map(|w| w.attack_speed).unwrap_or(4)
    }
}

impl MonsterData {
    /// Convert to resolved target.
    pub fn to_target_resolved(&self) -> TargetResolved {
        TargetResolved {
            name: self.name.clone(),
            hitpoints: self.hitpoints,
            defence_level: self.defence_level,
            defence_bonuses: DefenceBonuses {
                stab: self.defence_stab,
                slash: self.defence_slash,
                crush: self.defence_crush,
                magic: self.defence_magic,
                ranged: self.defence_ranged,
            },
            attributes: self.attributes.clone(),
        }
    }
}

impl EquipmentBonuses {
    /// Add another set of bonuses to this one.
    pub fn add(&mut self, other: &EquipmentBonuses) {
        self.attack_stab += other.attack_stab;
        self.attack_slash += other.attack_slash;
        self.attack_crush += other.attack_crush;
        self.attack_magic += other.attack_magic;
        self.attack_ranged += other.attack_ranged;
        self.defence_stab += other.defence_stab;
        self.defence_slash += other.defence_slash;
        self.defence_crush += other.defence_crush;
        self.defence_magic += other.defence_magic;
        self.defence_ranged += other.defence_ranged;
        self.melee_strength += other.melee_strength;
        self.ranged_strength += other.ranged_strength;
        self.magic_damage += other.magic_damage;
        self.prayer += other.prayer;
    }

    /// Get attack bonus for a specific attack type.
    pub fn attack_bonus_for(&self, attack_type: AttackType) -> i32 {
        match attack_type {
            AttackType::Stab => self.attack_stab,
            AttackType::Slash => self.attack_slash,
            AttackType::Crush => self.attack_crush,
            AttackType::Magic => self.attack_magic,
            AttackType::Ranged => self.attack_ranged,
        }
    }
}

impl DefenceBonuses {
    /// Get defence bonus for a specific attack type.
    pub fn defence_bonus_for(&self, attack_type: AttackType) -> i32 {
        match attack_type {
            AttackType::Stab => self.stab,
            AttackType::Slash => self.slash,
            AttackType::Crush => self.crush,
            AttackType::Magic => self.magic,
            AttackType::Ranged => self.ranged,
        }
    }
}

// =============================================================================
// Resolver: Build equipment list -> BuildResolved
// =============================================================================

pub struct Resolver<'a> {
    pub store: &'a DataStore,
}

impl<'a> Resolver<'a> {
    pub fn new(store: &'a DataStore) -> Self {
        Self { store }
    }

    /// Resolve equipment IDs into aggregated bonuses.
    pub fn resolve_equipment(
        &self,
        equipment_ids: &[Option<u32>],
        weapon_id: u32,
        stance: Stance,
        attack_type: AttackType,
    ) -> Result<BuildResolved, DataError> {
        let mut bonuses = EquipmentBonuses::default();

        // Add weapon bonuses
        let weapon = self.store.get_item(weapon_id)?;
        bonuses.add(&weapon.get_bonuses());
        let attack_speed = weapon.get_attack_speed();

        // Add other equipment bonuses
        for id in equipment_ids.iter().flatten() {
            let item = self.store.get_item(*id)?;
            bonuses.add(&item.get_bonuses());
        }

        Ok(BuildResolved {
            bonuses,
            attack_speed,
            combat: crate::model::Combat::Melee,
            attack_type,
            stance,
        })
    }

    /// Resolve a monster ID into target stats.
    pub fn resolve_monster(&self, monster_id: u32) -> Result<TargetResolved, DataError> {
        let monster = self.store.get_monster(monster_id)?;
        Ok(monster.to_target_resolved())
    }
}

// =============================================================================
// Prayer/Potion Resolution Helpers
// =============================================================================

/// Get prayer multipliers for a given prayer name.
/// Returns (attack_mult, strength_mult, defence_mult) as (num, den) pairs.
pub fn get_prayer_multipliers(prayer: &str) -> ((i32, i32), (i32, i32), (i32, i32)) {
    match prayer.to_lowercase().as_str() {
        // Piety: 20% attack, 23% strength, 25% defence
        "piety" => ((6, 5), (123, 100), (5, 4)),
        // Chivalry: 15% attack, 18% strength, 20% defence
        "chivalry" => ((23, 20), (59, 50), (6, 5)),
        // Ultimate Strength: 15% strength
        "ultimate_strength" | "ultimate strength" => ((1, 1), (23, 20), (1, 1)),
        // Incredible Reflexes: 15% attack
        "incredible_reflexes" | "incredible reflexes" => ((23, 20), (1, 1), (1, 1)),
        // Steel Skin: 15% defence
        "steel_skin" | "steel skin" => ((1, 1), (1, 1), (23, 20)),
        // No prayer or unknown
        _ => ((1, 1), (1, 1), (1, 1)),
    }
}

/// Get attack boost for a given potion type and base level.
pub fn get_potion_attack_boost(potion: &str, base_level: u8) -> i32 {
    let level = base_level as i32;
    match potion.to_lowercase().as_str() {
        // Super combat boosts all three: +5 + 15%
        "super_combat" | "super combat" => 5 + (level * 15) / 100,
        // Super attack: +5 + 15%
        "super_attack" | "super attack" => 5 + (level * 15) / 100,
        // Attack potion: +3 + 10%
        "attack" => 3 + (level * 10) / 100,
        // Super strength does NOT boost attack
        _ => 0,
    }
}

/// Get strength boost for a given potion type and base level.
pub fn get_potion_strength_boost(potion: &str, base_level: u8) -> i32 {
    let level = base_level as i32;
    match potion.to_lowercase().as_str() {
        // Super combat boosts all three: +5 + 15%
        "super_combat" | "super combat" => 5 + (level * 15) / 100,
        // Super strength: +5 + 15%
        "super_strength" | "super strength" => 5 + (level * 15) / 100,
        // Strength potion: +3 + 10%
        "strength" => 3 + (level * 10) / 100,
        // Super attack does NOT boost strength
        _ => 0,
    }
}

/// Get defence boost for a given potion type and base level.
pub fn get_potion_defence_boost(potion: &str, base_level: u8) -> i32 {
    let level = base_level as i32;
    match potion.to_lowercase().as_str() {
        // Super combat boosts all three: +5 + 15%
        "super_combat" | "super combat" => 5 + (level * 15) / 100,
        // Super defence: +5 + 15%
        "super_defence" | "super defence" => 5 + (level * 15) / 100,
        // Defence potion: +3 + 10%
        "defence" => 3 + (level * 10) / 100,
        _ => 0,
    }
}

/// Resolve player input into PlayerState.
pub fn resolve_player(
    skills: &crate::model::SkillsInput,
    prayers: &HashMap<String, bool>,
    boosts: &crate::model::BoostsInput,
    flags: &crate::model::FlagsInput,
) -> PlayerState {
    // Find active prayer multipliers (use first active prayer for MVP)
    let mut prayer_atk = (1, 1);
    let mut prayer_str = (1, 1);
    let mut prayer_def = (1, 1);

    for (prayer_name, active) in prayers {
        if *active {
            let (atk, str, def) = get_prayer_multipliers(prayer_name);
            // Take the best multipliers (simplified - in reality prayers don't stack)
            if atk.0 * prayer_atk.1 > prayer_atk.0 * atk.1 {
                prayer_atk = atk;
            }
            if str.0 * prayer_str.1 > prayer_str.0 * str.1 {
                prayer_str = str;
            }
            if def.0 * prayer_def.1 > prayer_def.0 * def.1 {
                prayer_def = def;
            }
        }
    }

    // Compute potion boosts
    let potion_attack = boosts
        .melee
        .as_ref()
        .map(|b| get_potion_attack_boost(&b.kind, skills.attack))
        .unwrap_or(0);
    let potion_strength = boosts
        .melee
        .as_ref()
        .map(|b| get_potion_strength_boost(&b.kind, skills.strength))
        .unwrap_or(0);
    let potion_defence = boosts
        .melee
        .as_ref()
        .map(|b| get_potion_defence_boost(&b.kind, skills.defence))
        .unwrap_or(0);

    PlayerState {
        attack: skills.attack,
        strength: skills.strength,
        defence: skills.defence,
        ranged: skills.ranged,
        magic: skills.magic,
        prayer: skills.prayer,
        hitpoints: skills.hitpoints,
        prayer_attack_mult: prayer_atk,
        prayer_strength_mult: prayer_str,
        prayer_defence_mult: prayer_def,
        potion_attack_boost: potion_attack,
        potion_strength_boost: potion_strength,
        potion_defence_boost: potion_defence,
        on_slayer_task: flags.on_slayer_task,
        in_wilderness: flags.in_wilderness,
    }
}
