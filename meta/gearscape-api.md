# Gearscape API Reference

**Base URL:** `https://api.gearscape.net/api`

**Discovered:** 2026-01-09

## Endpoints

### GET `/equipment/all`

Returns all non-weapon equipment organized by slot.

**Response structure:**
```json
{
  "equipment": {
    "head": [...],
    "body": [...],
    "legs": [...],
    "cape": [...],
    "shield": [...],
    "ammunition": [...],
    "hands": [...],
    "feet": [...],
    "neck": [...],
    "ring": [...]
  }
}
```

**Item counts (as of 2026-01-09):**
| Slot       | Count |
|------------|-------|
| head       | 205   |
| body       | 186   |
| legs       | 183   |
| cape       | 144   |
| shield     | 115   |
| ammunition | 113   |
| hands      | 98    |
| feet       | 96    |
| neck       | 42    |
| ring       | 37    |
| **Total**  | **1,219** |

**Equipment item schema:**
```json
{
  "id": 10350,
  "name": "3rd age full helmet",
  "price": 49487000,
  "combat_style": "other",
  "icon": "<base64 PNG>",
  "tradeable": true,
  "members": true,
  "weight": 0.907,

  // Attack bonuses
  "stab_bonus": 0,
  "slash_bonus": 0,
  "crush_bonus": 0,
  "ranged_bonus": -2,
  "magic_bonus": -5,

  // Strength bonuses
  "melee_str": 0,
  "ranged_str": 0,
  "magic_str": 0,

  // Defence bonuses
  "stab_def": 47,
  "slash_def": 49,
  "crush_def": 43,
  "ranged_def": 48,
  "magic_def": -3,

  // Other
  "prayer_bonus": 0,

  // Requirements
  "attack_req": 1,
  "strength_req": 1,
  "defence_req": 65,
  "ranged_req": 1,
  "magic_req": 1,
  "prayer_req": 1,
  "hitpoints_req": 1,
  "slayer_req": 1
}
```

---

### GET `/weapon/all`

Returns all weapons.

**Response structure:**
```json
{
  "weapons": [...]
}
```

**Count:** 564 weapons (as of 2026-01-09)

**Weapon item schema:**
```json
{
  "id": 4151,
  "name": "abyssal whip",
  "slot": "weapon",
  "price": 1410053,
  "combat_style": "melee",
  "two_handed": false,
  "icon": "<base64 PNG>",
  "subcategory": "whip",
  "attack_speed": 4,
  "tradeable": true,
  "members": true,
  "weight": 0.453,

  // Attack styles (format: "name,type,stance")
  "styles": [
    "flick,slash,accurate",
    "lash,slash,controlled",
    "deflect,slash,defensive"
  ],

  // Ammunition (for ranged weapons)
  "ammunition": [],

  // Attack bonuses
  "stab_bonus": 0,
  "slash_bonus": 82,
  "crush_bonus": 0,
  "ranged_bonus": 0,
  "magic_bonus": 0,

  // Strength bonuses
  "melee_str": 82,
  "ranged_str": 0,
  "magic_str": 0,

  // Defence bonuses
  "stab_def": 0,
  "slash_def": 0,
  "crush_def": 0,
  "ranged_def": 0,
  "magic_def": 0,

  // Other
  "prayer_bonus": 0,

  // Special attack (null if none)
  "spec_accuracy": 0.25,
  "spec_damage": 0,
  "spec_defence": "slash",
  "spec_energy": 50,

  // Requirements
  "attack_req": 70,
  "strength_req": 1,
  "defence_req": 1,
  "ranged_req": 1,
  "magic_req": 1,
  "prayer_req": 1,
  "hitpoints_req": 1,
  "slayer_req": 1
}
```

**Weapon subcategories observed:** axe, whip, sword, scimitar, mace, dagger, spear, halberd, staff, wand, bow, crossbow, thrown, etc.

**Style format:** `"<style_name>,<attack_type>,<stance>"`
- attack_type: stab, slash, crush, ranged, magic
- stance: accurate, aggressive, controlled, defensive, rapid, longrange

---

### GET `/monster`

Returns monster metadata only. **NOT useful for DPS calculation** - missing combat stats.

**Response structure:**
```json
{
  "monsters": [...]
}
```

**Count:** 2,541 monsters (as of 2026-01-09)

**Monster schema (sparse):**
```json
{
  "id": 3127,
  "name": "tztok-jad",
  "level_cb": 702,
  "boss": true,
  "attributes": ["boss"]
}
```

**Missing fields we need:**
- `hitpoints`
- `defence_level`
- `attack_level`, `strength_level`, `magic_level`, `ranged_level`
- `defence_stab`, `defence_slash`, `defence_crush`, `defence_magic`, `defence_ranged`

**Conclusion:** Use OSRS Wiki for monster data instead.

---

## Notes

- Response size: ~1MB for equipment, ~500KB for weapons
- Icons are base64-encoded PNGs (small thumbnails)
- Prices appear to be GE prices (updated periodically)
- IDs match OSRS item IDs
- No authentication required
- No rate limiting observed (be respectful)
