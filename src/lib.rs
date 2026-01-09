// src/lib.rs
//
// OSRS DPS Simulator - Library entry point

pub mod data;
pub mod effects;
pub mod formulas;
pub mod model;

// Re-export commonly used types
pub use data::{DataError, DataStore, Resolver};
pub use effects::{default_effects, EffectContext, EffectRegistry, EngineState, Stage};
pub use formulas::evaluate;
pub use model::{
    AttackType, BuildInput, BuildResolved, Combat, DefenceBonuses, EquipmentBonuses,
    EquipmentInput, EvalContext, EvalResult, PlayerInput, PlayerState, Stance, StyleInput,
    TargetInput, TargetResolved,
};
