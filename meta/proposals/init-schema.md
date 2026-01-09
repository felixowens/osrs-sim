### **ChatGPT**

do you already have a JSON schema in mind for player/build/target

---

### **ChatGPT**

Yep — I’d use a small, “input-only” schema that references game data by IDs, plus an escape hatch for overrides. The goal is:

inputs stay compact (IDs, toggles)

engine stays deterministic

you can extend without breaking old files (via schema\_version + optional fields)

Below is what I’d ship for an extensible MVP.

Design rules

All items/monsters are referenced by numeric IDs (from your data pack).

Inputs are split into player, build, target.

Include schema\_version everywhere.

Allow overrides for target and (later) special-casing without hardcoding.

Player JSON (MVP + extensible)
Example

\`\`\`json
jsonCopy code{
  "schema\_version": 1,
  "skills": {
    "attack": 99,
    "strength": 99,
    "defence": 99,
    "ranged": 99,
    "magic": 99,
    "prayer": 99,
    "hitpoints": 99
  },
  "prayers": {
    "piety": true
  },
  "boosts": {
    "melee": { "type": "super\_combat" }
  },
  "flags": {
    "on\_slayer\_task": true,
    "in\_wilderness": false
  }
}
\`\`\`

JSON Schema (Draft 2020-12)

\`\`\`json
jsonCopy code{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "<https://example.local/schemas/player.v1.json>",
  "type": "object",
  "additionalProperties": false,
  "required": ["schema\_version", "skills"],
  "properties": {
    "schema\_version": { "type": "integer", "const": 1 },
    "skills": {
      "type": "object",
      "additionalProperties": false,
      "required": ["attack", "strength", "defence", "ranged", "magic", "prayer", "hitpoints"],
      "properties": {
        "attack": { "type": "integer", "minimum": 1, "maximum": 99 },
        "strength": { "type": "integer", "minimum": 1, "maximum": 99 },
        "defence": { "type": "integer", "minimum": 1, "maximum": 99 },
        "ranged": { "type": "integer", "minimum": 1, "maximum": 99 },
        "magic": { "type": "integer", "minimum": 1, "maximum": 99 },
        "prayer": { "type": "integer", "minimum": 1, "maximum": 99 },
        "hitpoints": { "type": "integer", "minimum": 10, "maximum": 99 }
      }
    },
    "prayers": {
      "type": "object",
      "additionalProperties": { "type": "boolean" },
      "description": "Map of prayer\_key -> enabled. Keys are your canonical prayer ids (e.g. piety, rigour)."
    },
    "boosts": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "melee": {
          "type": "object",
          "additionalProperties": false,
          "required": ["type"],
          "properties": { "type": { "type": "string" } }
        },
        "ranged": {
          "type": "object",
          "additionalProperties": false,
          "required": ["type"],
          "properties": { "type": { "type": "string" } }
        },
        "magic": {
          "type": "object",
          "additionalProperties": false,
          "required": ["type"],
          "properties": { "type": { "type": "string" } }
        }
      }
    },
    "flags": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "on\_slayer\_task": { "type": "boolean" },
        "in\_wilderness": { "type": "boolean" }
      }
    }
  }
}
\`\`\`

Why this shape is nice in Rust: skills is a fixed struct; prayers can be HashMap<PrayerKey,bool>; boosts can be enums.

Build JSON
Example

\`\`\`json
jsonCopy code{
  "schema\_version": 1,
  "equipment": {
    "head": 11865,
    "cape": 6570,
    "neck": 6585,
    "weapon": 4151,
    "body": 4720,
    "legs": 4722,
    "hands": 7462,
    "feet": 3105,
    "ring": 6737,
    "shield": 12825
  },
  "style": {
    "combat": "melee",
    "attack\_type": "slash",
    "stance": "aggressive"
  }
}
\`\`\`

JSON Schema

\`\`\`json
jsonCopy code{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "<https://example.local/schemas/build.v1.json>",
  "type": "object",
  "additionalProperties": false,
  "required": ["schema\_version", "equipment", "style"],
  "properties": {
    "schema\_version": { "type": "integer", "const": 1 },
    "equipment": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "head": { "type": "integer" },
        "cape": { "type": "integer" },
        "neck": { "type": "integer" },
        "ammo": { "type": "integer" },
        "weapon": { "type": "integer" },
        "body": { "type": "integer" },
        "shield": { "type": "integer" },
        "legs": { "type": "integer" },
        "hands": { "type": "integer" },
        "feet": { "type": "integer" },
        "ring": { "type": "integer" }
      },
      "required": ["weapon"]
    },
    "style": {
      "type": "object",
      "additionalProperties": false,
      "required": ["combat", "attack\_type", "stance"],
      "properties": {
        "combat": { "type": "string", "enum": ["melee", "ranged", "magic"] },
        "attack\_type": { "type": "string", "enum": ["stab", "slash", "crush", "ranged", "magic"] },
        "stance": { "type": "string", "enum": ["accurate", "aggressive", "defensive", "controlled", "rapid", "longrange"] },
        "special\_attack": { "type": "boolean", "default": false }
      }
    }
  }
}
\`\`\`

Notes:

stance exists even if some weapons don’t support all stances — validation can happen at “resolve time”.

You can later add spell (magic), ammo\_mode, salve\_mode, etc., without breaking schema v1.

Target JSON
Example 1 (common): reference monster id

\`\`\`json
jsonCopy code{
  "schema\_version": 1,
  "monster\_id": 415,
  "overrides": null
}
\`\`\`

Example 2: override a few things (good for “phases” or special variants)

\`\`\`json
jsonCopy code{
  "schema\_version": 1,
  "monster\_id": 415,
  "overrides": {
    "attributes\_add": ["undead"],
    "defence\_level": 200,
    "defence\_bonuses": { "slash": 50 }
  }
}
\`\`\`

Example 3 (escape hatch): fully custom target

\`\`\`json
jsonCopy code{
  "schema\_version": 1,
  "custom": {
    "name": "Dummy",
    "levels": { "hitpoints": 100, "defence": 1 },
    "defence\_bonuses": { "stab": 0, "slash": 0, "crush": 0, "magic": 0, "ranged": 0 },
    "attributes": []
  }
}
\`\`\`

JSON Schema

\`\`\`json
jsonCopy code{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "<https://example.local/schemas/target.v1.json>",
  "type": "object",
  "additionalProperties": false,
  "required": ["schema\_version"],
  "properties": {
    "schema\_version": { "type": "integer", "const": 1 },

    "monster\_id": { "type": "integer" },

    "overrides": {
      "type": ["object", "null"],
      "additionalProperties": false,
      "properties": {
        "attributes\_add": { "type": "array", "items": { "type": "string" } },
        "attributes\_remove": { "type": "array", "items": { "type": "string" } },

        "attack\_level": { "type": "integer", "minimum": 1 },
        "strength\_level": { "type": "integer", "minimum": 1 },
        "defence\_level": { "type": "integer", "minimum": 1 },
        "magic\_level": { "type": "integer", "minimum": 1 },
        "ranged\_level": { "type": "integer", "minimum": 1 },
        "hitpoints\_level": { "type": "integer", "minimum": 1 },

        "defence\_bonuses": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "stab": { "type": "integer" },
            "slash": { "type": "integer" },
            "crush": { "type": "integer" },
            "magic": { "type": "integer" },
            "ranged": { "type": "integer" }
          }
        }
      }
    },

    "custom": {
      "type": "object",
      "additionalProperties": false,
      "required": ["name", "levels", "defence\_bonuses", "attributes"],
      "properties": {
        "name": { "type": "string" },
        "levels": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "attack": { "type": "integer", "minimum": 1 },
            "strength": { "type": "integer", "minimum": 1 },
            "defence": { "type": "integer", "minimum": 1 },
            "magic": { "type": "integer", "minimum": 1 },
            "ranged": { "type": "integer", "minimum": 1 },
            "hitpoints": { "type": "integer", "minimum": 1 }
          },
          "required": ["defence", "hitpoints"]
        },
        "defence\_bonuses": {
          "type": "object",
          "additionalProperties": false,
          "required": ["stab", "slash", "crush", "magic", "ranged"],
          "properties": {
            "stab": { "type": "integer" },
            "slash": { "type": "integer" },
            "crush": { "type": "integer" },
            "magic": { "type": "integer" },
            "ranged": { "type": "integer" }
          }
        },
        "attributes": { "type": "array", "items": { "type": "string" } }
      }
    }
  },
  "oneOf": [
    { "required": ["monster\_id"] },
    { "required": ["custom"] }
  ]
}
\`\`\`

The one extra file I’d add: “run config”
Not required, but very ergonomic for CLI:

\`\`\`json
jsonCopy code{
  "schema\_version": 1,
  "data\_version": "2026-01-01",
  "player": "fixtures/player.json",
  "build": "fixtures/build.json",
  "target": "fixtures/target.json",
  "output": { "format": "json", "explain": true }
}
\`\`\`

This lets you run batches and keep outputs reproducible.

If you want, I can translate these schemas into Rust structs + serde attributes (including enums and deny\_unknown\_fields) so you can paste them straight into model.rs.

---

### **ChatGPT**

sure

---

### **ChatGPT**

Here’s a clean, idiomatic model.rs you can drop in and start using with serde\_json. It matches the Player/Build/Target shapes we discussed, uses deny\_unknown\_fields everywhere, and keeps room to grow without boxing you in.

\`\`\`rust
rustCopy code// src/model.rs
# ![allow(clippy::struct\_excessive\_bools)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bump this when you make a breaking change to input formats.
pub const SCHEMA\_VERSION\_V1: u32 = 1;

// -----------------------------
// Player
// -----------------------------

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct PlayerInput {
    pub schema\_version: u32,
    pub skills: SkillsInput,

    /// Map of prayer\_key -> enabled (e.g. "piety": true).
    /// Keep as String for MVP; you can later switch to a PrayerKey newtype/enum.
    #[serde(default)]
    pub prayers: HashMap<String, bool>,

    /// Optional boosts; omitted means "no boost".
    #[serde(default)]
    pub boosts: BoostsInput,

    /// Context flags (task/wilderness/etc).
    #[serde(default)]
    pub flags: FlagsInput,
}

# [derive(Debug, Clone, Default, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct BoostsInput {
    #[serde(default)]
    pub melee: Option<BoostSpec>,
    #[serde(default)]
    pub ranged: Option<BoostSpec>,
    #[serde(default)]
    pub magic: Option<BoostSpec>,
}

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct BoostSpec {
    /// JSON uses field name "type"; Rust uses "kind".
    #[serde(rename = "type")]
    pub kind: String,
}

# [derive(Debug, Clone, Default, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct FlagsInput {
    #[serde(default)]
    pub on\_slayer\_task: bool,
    #[serde(default)]
    pub in\_wilderness: bool,
}

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct SkillsInput {
    pub attack: u16,
    pub strength: u16,
    pub defence: u16,
    pub ranged: u16,
    pub magic: u16,
    pub prayer: u16,
    pub hitpoints: u16,
}

// -----------------------------
// Build
// -----------------------------

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct BuildInput {
    pub schema\_version: u32,
    pub equipment: EquipmentInput,
    pub style: StyleInput,
}

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
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

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct StyleInput {
    pub combat: Combat,
    pub attack\_type: AttackType,
    pub stance: Stance,

    /// Off by default.
    #[serde(default)]
    pub special\_attack: bool,
}

# [derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
# [serde(rename\_all = "snake\_case")]
pub enum Combat {
    Melee,
    Ranged,
    Magic,
}

# [derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
# [serde(rename\_all = "snake\_case")]
pub enum AttackType {
    Stab,
    Slash,
    Crush,
    Ranged,
    Magic,
}

# [derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
# [serde(rename\_all = "snake\_case")]
pub enum Stance {
    Accurate,
    Aggressive,
    Defensive,
    Controlled,
    Rapid,
    Longrange,
}

// -----------------------------
// Target
// -----------------------------

/// Matches the "oneOf": either {monster\_id, overrides?} or {custom}.
# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(untagged)]
pub enum TargetInput {
    ById(TargetByIdInput),
    Custom(TargetCustomInput),
}

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct TargetByIdInput {
    pub schema\_version: u32,
    pub monster\_id: u32,

    #[serde(default)]
    pub overrides: Option<TargetOverrides>,
}

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct TargetCustomInput {
    pub schema\_version: u32,
    pub custom: CustomTarget,
}

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct TargetOverrides {
    #[serde(default)]
    pub attributes\_add: Vec<String>,
    #[serde(default)]
    pub attributes\_remove: Vec<String>,

    #[serde(default)]
    pub attack\_level: Option<u16>,
    #[serde(default)]
    pub strength\_level: Option<u16>,
    #[serde(default)]
    pub defence\_level: Option<u16>,
    #[serde(default)]
    pub magic\_level: Option<u16>,
    #[serde(default)]
    pub ranged\_level: Option<u16>,
    #[serde(default)]
    pub hitpoints\_level: Option<u16>,

    #[serde(default)]
    pub defence\_bonuses: Option<DefenceBonusesPartial>,
}

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct DefenceBonusesPartial {
    #[serde(default)]
    pub stab: Option<i16>,
    #[serde(default)]
    pub slash: Option<i16>,
    #[serde(default)]
    pub crush: Option<i16>,
    #[serde(default)]
    pub magic: Option<i16>,
    #[serde(default)]
    pub ranged: Option<i16>,
}

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct CustomTarget {
    pub name: String,
    pub levels: CustomTargetLevels,
    pub defence\_bonuses: DefenceBonuses,
    #[serde(default)]
    pub attributes: Vec<String>,
}

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct CustomTargetLevels {
    /// Required
    pub defence: u16,
    /// Required
    pub hitpoints: u16,

    /// Optional (MVP: often unused)
    #[serde(default)]
    pub attack: Option<u16>,
    #[serde(default)]
    pub strength: Option<u16>,
    #[serde(default)]
    pub magic: Option<u16>,
    #[serde(default)]
    pub ranged: Option<u16>,
}

# [derive(Debug, Clone, Serialize, Deserialize)]
# [serde(deny\_unknown\_fields)]
pub struct DefenceBonuses {
    pub stab: i16,
    pub slash: i16,
    pub crush: i16,
    pub magic: i16,
    pub ranged: i16,
}

// -----------------------------
// Optional: minimal validation helpers
// -----------------------------

impl PlayerInput {
    pub fn validate\_basic(&self) -> Result<(), String> {
        if self.schema\_version != SCHEMA\_VERSION\_V1 {
            return Err(format!("player.schema\_version must be {SCHEMA\_VERSION\_V1}"));
        }
        Ok(())
    }
}

impl BuildInput {
    pub fn validate\_basic(&self) -> Result<(), String> {
        if self.schema\_version != SCHEMA\_VERSION\_V1 {
            return Err(format!("build.schema\_version must be {SCHEMA\_VERSION\_V1}"));
        }
        Ok(())
    }
}

impl TargetInput {
    pub fn validate\_basic(&self) -> Result<(), String> {
        match self {
            TargetInput::ById(t) => {
                if t.schema\_version != SCHEMA\_VERSION\_V1 {
                    return Err(format!("target.schema\_version must be {SCHEMA\_VERSION\_V1}"));
                }
            }
            TargetInput::Custom(t) => {
                if t.schema\_version != SCHEMA\_VERSION\_V1 {
                    return Err(format!("target.schema\_version must be {SCHEMA\_VERSION\_V1}"));
                }
            }
        }
        Ok(())
    }
}
\`\`\`

A couple of idiomatic notes (worth doing up front):

Keep input structs separate from your resolved internal model (where item IDs become real stats, weapon profiles, etc.). Don’t pollute the engine with Option<u32> everywhere.

Use validate\_basic() early in the CLI; deeper validation (e.g., “stance exists for this weapon”) belongs in your resolver layer, not in serde parsing.

If you want, next I can sketch the “resolved” structs (BuildResolved, TargetResolved) and the resolver signatures so your engine never has to touch JSON-y optionality.

---
