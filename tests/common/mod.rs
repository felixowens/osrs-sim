// tests/common/mod.rs
//
// Shared test utilities and fixture loading

use osrs_sim::{
    data::{resolve_player, DataStore, Resolver},
    evaluate, BuildInput, EvalContext, EvalResult, PlayerInput, TargetInput, TargetResolved,
};
use std::path::Path;

/// Load fixtures and evaluate DPS, returning the result.
pub fn eval_from_fixtures(player_file: &str, build_file: &str, target_file: &str) -> EvalResult {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");

    // Load data store
    let store = DataStore::load_from_dir(&data_dir).expect("Failed to load data store");

    // Load player
    let player_path = fixtures_dir.join(player_file);
    let player_json = std::fs::read_to_string(&player_path)
        .unwrap_or_else(|_| panic!("Failed to read player fixture: {:?}", player_path));
    let player_input: PlayerInput = serde_json::from_str(&player_json)
        .unwrap_or_else(|e| panic!("Failed to parse player JSON: {}", e));

    // Load build
    let build_path = fixtures_dir.join(build_file);
    let build_json = std::fs::read_to_string(&build_path)
        .unwrap_or_else(|_| panic!("Failed to read build fixture: {:?}", build_path));
    let build_input: BuildInput = serde_json::from_str(&build_json)
        .unwrap_or_else(|e| panic!("Failed to parse build JSON: {}", e));

    // Load target
    let target_path = fixtures_dir.join(target_file);
    let target_json = std::fs::read_to_string(&target_path)
        .unwrap_or_else(|_| panic!("Failed to read target fixture: {:?}", target_path));
    let target_input: TargetInput = serde_json::from_str(&target_json)
        .unwrap_or_else(|e| panic!("Failed to parse target JSON: {}", e));

    // Resolve player
    let player = resolve_player(
        &player_input.skills,
        &player_input.prayers,
        &player_input.boosts,
        &player_input.flags,
    );

    // Resolve build
    let resolver = Resolver::new(&store);
    let equipment_ids = [
        build_input.equipment.head,
        build_input.equipment.cape,
        build_input.equipment.neck,
        build_input.equipment.ammo,
        build_input.equipment.body,
        build_input.equipment.shield,
        build_input.equipment.legs,
        build_input.equipment.hands,
        build_input.equipment.feet,
        build_input.equipment.ring,
    ];

    let build = resolver
        .resolve_equipment(
            &equipment_ids,
            build_input.equipment.weapon,
            build_input.style.stance,
            build_input.style.attack_type,
        )
        .expect("Failed to resolve equipment");

    // Resolve target
    let target = match target_input {
        TargetInput::ById(ref by_id) => {
            let mut resolved = resolver
                .resolve_monster(by_id.monster_id)
                .expect("Failed to resolve monster");

            if let Some(ref overrides) = by_id.overrides {
                if let Some(def_level) = overrides.defence_level {
                    resolved.defence_level = def_level;
                }
                if let Some(ref def_bonuses) = overrides.defence_bonuses {
                    if let Some(v) = def_bonuses.stab {
                        resolved.defence_bonuses.stab = v;
                    }
                    if let Some(v) = def_bonuses.slash {
                        resolved.defence_bonuses.slash = v;
                    }
                    if let Some(v) = def_bonuses.crush {
                        resolved.defence_bonuses.crush = v;
                    }
                    if let Some(v) = def_bonuses.magic {
                        resolved.defence_bonuses.magic = v;
                    }
                    if let Some(v) = def_bonuses.ranged {
                        resolved.defence_bonuses.ranged = v;
                    }
                }
                for attr in &overrides.attributes_add {
                    if !resolved.attributes.contains(attr) {
                        resolved.attributes.push(attr.clone());
                    }
                }
                for attr in &overrides.attributes_remove {
                    resolved.attributes.retain(|a| a != attr);
                }
            }
            resolved
        }
        TargetInput::Custom(ref custom) => TargetResolved {
            name: custom.custom.name.clone(),
            hitpoints: custom.custom.levels.hitpoints,
            defence_level: custom.custom.levels.defence,
            defence_bonuses: custom.custom.defence_bonuses,
            attributes: custom.custom.attributes.clone(),
        },
    };

    // Evaluate
    let ctx = EvalContext {
        player: &player,
        build: &build,
        target: &target,
    };

    evaluate(&ctx)
}

/// Expected values for a golden test
#[derive(Debug, Clone)]
pub struct Expected {
    pub dps: f64,
    pub max_hit: u32,
    pub accuracy: f64, // as decimal, e.g. 0.2531 for 25.31%
}

/// Assert that result matches expected values within tolerance
pub fn assert_result_matches(result: &EvalResult, expected: &Expected, test_name: &str) {
    assert_eq!(
        result.max_hit, expected.max_hit,
        "[{}] Max hit mismatch: expected {}, got {}",
        test_name, expected.max_hit, result.max_hit
    );

    let acc_tolerance = 0.001;
    assert!(
        (result.accuracy - expected.accuracy).abs() < acc_tolerance,
        "[{}] Accuracy mismatch: expected {:.2}%, got {:.2}%",
        test_name,
        expected.accuracy * 100.0,
        result.accuracy * 100.0
    );

    let dps_tolerance = 0.01;
    assert!(
        (result.dps - expected.dps).abs() < dps_tolerance,
        "[{}] DPS mismatch: expected {:.4}, got {:.4}",
        test_name,
        expected.dps,
        result.dps
    );
}
