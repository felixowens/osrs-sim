// src/effects.rs
//
// Effects system with staged modifiers.
// Effects hook into different stages of the DPS calculation pipeline.

use serde::{Deserialize, Serialize};

/// Stages in the DPS calculation pipeline where effects can be applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    /// Applied after effective level calculation (e.g., Void Knight)
    PostEffectiveLevel,
    /// Applied before roll calculation
    PreRolls,
    /// Applied after max hit calculation (e.g., Slayer helm, Salve amulet)
    PostMaxHit,
    /// Applied after accuracy calculation
    PostAccuracy,
}

/// Operations that effects can perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    /// Multiply a stat by num/den
    Multiply { stat: Stat, num: i32, den: i32 },
    /// Add a flat value to a stat
    Add { stat: Stat, value: i32 },
}

/// Stats that effects can modify.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stat {
    EffectiveAttack,
    EffectiveStrength,
    MaxHit,
    AttackRoll,
    Accuracy,
}

/// Conditions for effect activation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    /// Effect only applies on slayer task
    #[serde(default)]
    pub on_slayer_task: Option<bool>,

    /// Effect only applies in wilderness
    #[serde(default)]
    pub in_wilderness: Option<bool>,

    /// Effect only applies against targets with specific attributes
    #[serde(default)]
    pub target_attributes: Vec<String>,

    /// Combat style requirement
    #[serde(default)]
    pub combat: Option<String>,
}

/// A declarative effect that can be applied during DPS calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub id: String,
    pub name: String,
    pub stage: Stage,

    /// Lower priority = applied first
    #[serde(default)]
    pub priority: i16,

    /// Effects in the same exclusive group don't stack
    #[serde(default)]
    pub exclusive_group: Option<String>,

    /// Conditions that must be met for the effect to apply
    #[serde(default)]
    pub condition: Condition,

    /// The operation to perform
    pub op: Op,
}

/// Context for evaluating effect conditions.
pub struct EffectContext {
    pub on_slayer_task: bool,
    pub in_wilderness: bool,
    pub target_attributes: Vec<String>,
    pub combat: String,
}

impl Effect {
    /// Check if the effect's conditions are met.
    pub fn conditions_met(&self, ctx: &EffectContext) -> bool {
        let cond = &self.condition;

        // Check slayer task condition
        if let Some(required) = cond.on_slayer_task {
            if ctx.on_slayer_task != required {
                return false;
            }
        }

        // Check wilderness condition
        if let Some(required) = cond.in_wilderness {
            if ctx.in_wilderness != required {
                return false;
            }
        }

        // Check target attributes
        if !cond.target_attributes.is_empty() {
            let has_required_attr = cond
                .target_attributes
                .iter()
                .any(|attr| ctx.target_attributes.contains(attr));
            if !has_required_attr {
                return false;
            }
        }

        // Check combat style
        if let Some(ref required_combat) = cond.combat {
            if ctx.combat.to_lowercase() != required_combat.to_lowercase() {
                return false;
            }
        }

        true
    }
}

/// Mutable state that effects modify during evaluation.
#[derive(Debug, Clone, Default)]
pub struct EngineState {
    pub effective_attack: i32,
    pub effective_strength: i32,
    pub max_hit: i32,
    pub attack_roll: i32,
    pub accuracy: f64,
}

impl EngineState {
    /// Apply an effect operation to the state.
    pub fn apply(&mut self, op: &Op) {
        match op {
            Op::Multiply { stat, num, den } => {
                let mult = |v: i32| (v * num) / den;
                match stat {
                    Stat::EffectiveAttack => self.effective_attack = mult(self.effective_attack),
                    Stat::EffectiveStrength => {
                        self.effective_strength = mult(self.effective_strength)
                    }
                    Stat::MaxHit => self.max_hit = mult(self.max_hit),
                    Stat::AttackRoll => self.attack_roll = mult(self.attack_roll),
                    Stat::Accuracy => {
                        self.accuracy = self.accuracy * (*num as f64) / (*den as f64)
                    }
                }
            }
            Op::Add { stat, value } => match stat {
                Stat::EffectiveAttack => self.effective_attack += value,
                Stat::EffectiveStrength => self.effective_strength += value,
                Stat::MaxHit => self.max_hit += value,
                Stat::AttackRoll => self.attack_roll += value,
                Stat::Accuracy => self.accuracy += *value as f64,
            },
        }
    }
}

/// Collection of effects that can be applied.
#[derive(Debug, Clone, Default)]
pub struct EffectRegistry {
    pub effects: Vec<Effect>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a built-in effect.
    pub fn register(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    /// Get effects for a specific stage, sorted by priority.
    pub fn for_stage(&self, stage: Stage) -> Vec<&Effect> {
        let mut effects: Vec<_> = self.effects.iter().filter(|e| e.stage == stage).collect();
        effects.sort_by_key(|e| e.priority);
        effects
    }

    /// Apply all effects for a stage that meet their conditions.
    /// Handles exclusive groups (only first matching effect in group applies).
    pub fn apply_stage(
        &self,
        stage: Stage,
        ctx: &EffectContext,
        state: &mut EngineState,
    ) -> Vec<String> {
        let mut applied = Vec::new();
        let mut used_groups = std::collections::HashSet::new();

        for effect in self.for_stage(stage) {
            // Check exclusive group
            if let Some(ref group) = effect.exclusive_group {
                if used_groups.contains(group) {
                    continue;
                }
            }

            // Check conditions
            if !effect.conditions_met(ctx) {
                continue;
            }

            // Apply the effect
            state.apply(&effect.op);
            applied.push(effect.id.clone());

            // Mark exclusive group as used
            if let Some(ref group) = effect.exclusive_group {
                used_groups.insert(group.clone());
            }
        }

        applied
    }
}

// =============================================================================
// Built-in Effects
// =============================================================================

/// Create the default effect registry with common effects.
pub fn default_effects() -> EffectRegistry {
    let mut registry = EffectRegistry::new();

    // Slayer helm (i) - 7/6 multiplier to max hit and accuracy on task
    registry.register(Effect {
        id: "slayer_helm_melee".to_string(),
        name: "Slayer helm (i) - Melee".to_string(),
        stage: Stage::PostMaxHit,
        priority: 50,
        exclusive_group: Some("slayer_salve".to_string()),
        condition: Condition {
            on_slayer_task: Some(true),
            combat: Some("melee".to_string()),
            ..Default::default()
        },
        op: Op::Multiply {
            stat: Stat::MaxHit,
            num: 7,
            den: 6,
        },
    });

    // Salve amulet (ei) - 6/5 multiplier against undead
    registry.register(Effect {
        id: "salve_ei_melee".to_string(),
        name: "Salve amulet (ei) - Melee".to_string(),
        stage: Stage::PostMaxHit,
        priority: 50,
        exclusive_group: Some("slayer_salve".to_string()),
        condition: Condition {
            target_attributes: vec!["undead".to_string()],
            combat: Some("melee".to_string()),
            ..Default::default()
        },
        op: Op::Multiply {
            stat: Stat::MaxHit,
            num: 6,
            den: 5,
        },
    });

    // Void Knight melee - 11/10 multiplier to effective levels
    registry.register(Effect {
        id: "void_melee_str".to_string(),
        name: "Void Knight - Melee Strength".to_string(),
        stage: Stage::PostEffectiveLevel,
        priority: 100,
        exclusive_group: None,
        condition: Condition {
            combat: Some("melee".to_string()),
            ..Default::default()
        },
        op: Op::Multiply {
            stat: Stat::EffectiveStrength,
            num: 11,
            den: 10,
        },
    });

    registry.register(Effect {
        id: "void_melee_atk".to_string(),
        name: "Void Knight - Melee Attack".to_string(),
        stage: Stage::PostEffectiveLevel,
        priority: 100,
        exclusive_group: None,
        condition: Condition {
            combat: Some("melee".to_string()),
            ..Default::default()
        },
        op: Op::Multiply {
            stat: Stat::EffectiveAttack,
            num: 11,
            den: 10,
        },
    });

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_conditions_slayer_task() {
        let effect = Effect {
            id: "test".to_string(),
            name: "Test".to_string(),
            stage: Stage::PostMaxHit,
            priority: 0,
            exclusive_group: None,
            condition: Condition {
                on_slayer_task: Some(true),
                ..Default::default()
            },
            op: Op::Multiply {
                stat: Stat::MaxHit,
                num: 7,
                den: 6,
            },
        };

        let ctx_on_task = EffectContext {
            on_slayer_task: true,
            in_wilderness: false,
            target_attributes: vec![],
            combat: "melee".to_string(),
        };

        let ctx_off_task = EffectContext {
            on_slayer_task: false,
            in_wilderness: false,
            target_attributes: vec![],
            combat: "melee".to_string(),
        };

        assert!(effect.conditions_met(&ctx_on_task));
        assert!(!effect.conditions_met(&ctx_off_task));
    }

    #[test]
    fn test_effect_apply_multiply() {
        let mut state = EngineState {
            max_hit: 30,
            ..Default::default()
        };

        let op = Op::Multiply {
            stat: Stat::MaxHit,
            num: 7,
            den: 6,
        };

        state.apply(&op);
        assert_eq!(state.max_hit, 35); // 30 * 7/6 = 35
    }

    #[test]
    fn test_exclusive_group() {
        let mut registry = EffectRegistry::new();

        registry.register(Effect {
            id: "effect_a".to_string(),
            name: "Effect A".to_string(),
            stage: Stage::PostMaxHit,
            priority: 10,
            exclusive_group: Some("group1".to_string()),
            condition: Condition::default(),
            op: Op::Multiply {
                stat: Stat::MaxHit,
                num: 2,
                den: 1,
            },
        });

        registry.register(Effect {
            id: "effect_b".to_string(),
            name: "Effect B".to_string(),
            stage: Stage::PostMaxHit,
            priority: 20,
            exclusive_group: Some("group1".to_string()),
            condition: Condition::default(),
            op: Op::Multiply {
                stat: Stat::MaxHit,
                num: 3,
                den: 1,
            },
        });

        let ctx = EffectContext {
            on_slayer_task: false,
            in_wilderness: false,
            target_attributes: vec![],
            combat: "melee".to_string(),
        };

        let mut state = EngineState {
            max_hit: 10,
            ..Default::default()
        };

        let applied = registry.apply_stage(Stage::PostMaxHit, &ctx, &mut state);

        // Only effect_a should apply (lower priority)
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0], "effect_a");
        assert_eq!(state.max_hit, 20); // 10 * 2 = 20, not 10 * 2 * 3 = 60
    }
}
