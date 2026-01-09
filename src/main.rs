// src/main.rs
//
// CLI for the OSRS DPS Simulator

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use osrs_sim::{
    data::{resolve_player, DataStore, Resolver},
    evaluate, BuildInput, EvalContext, PlayerInput, TargetInput, TargetResolved,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "osrs-sim")]
#[command(about = "OSRS DPS Simulator and Gear Optimizer")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate DPS for a given setup
    Eval {
        /// Path to player JSON file
        #[arg(long)]
        player: PathBuf,

        /// Path to build JSON file
        #[arg(long)]
        build: PathBuf,

        /// Path to target JSON file
        #[arg(long)]
        target: PathBuf,

        /// Path to data directory (default: ./data)
        #[arg(long, default_value = "./data")]
        data_dir: PathBuf,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Show detailed breakdown
        #[arg(long)]
        explain: bool,
    },

    /// Validate data files
    ValidateData {
        /// Path to data directory
        #[arg(default_value = "./data")]
        data_dir: PathBuf,
    },

    /// Show info about a specific item
    ItemInfo {
        /// Item ID
        id: u32,

        /// Path to data directory
        #[arg(long, default_value = "./data")]
        data_dir: PathBuf,
    },

    /// Show info about a specific monster
    MonsterInfo {
        /// Monster ID
        id: u32,

        /// Path to data directory
        #[arg(long, default_value = "./data")]
        data_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Eval {
            player,
            build,
            target,
            data_dir,
            json,
            explain,
        } => cmd_eval(&player, &build, &target, &data_dir, json, explain),

        Commands::ValidateData { data_dir } => cmd_validate_data(&data_dir),

        Commands::ItemInfo { id, data_dir } => cmd_item_info(id, &data_dir),

        Commands::MonsterInfo { id, data_dir } => cmd_monster_info(id, &data_dir),
    }
}

fn cmd_eval(
    player_path: &PathBuf,
    build_path: &PathBuf,
    target_path: &PathBuf,
    data_dir: &PathBuf,
    json_output: bool,
    explain: bool,
) -> Result<()> {
    // Load data store
    let store = DataStore::load_from_dir(data_dir)
        .with_context(|| format!("Failed to load data from {:?}", data_dir))?;

    // Load and parse input files
    let player_json = std::fs::read_to_string(player_path)
        .with_context(|| format!("Failed to read player file: {:?}", player_path))?;
    let player_input: PlayerInput = serde_json::from_str(&player_json)
        .with_context(|| format!("Failed to parse player JSON: {:?}", player_path))?;

    let build_json = std::fs::read_to_string(build_path)
        .with_context(|| format!("Failed to read build file: {:?}", build_path))?;
    let build_input: BuildInput = serde_json::from_str(&build_json)
        .with_context(|| format!("Failed to parse build JSON: {:?}", build_path))?;

    let target_json = std::fs::read_to_string(target_path)
        .with_context(|| format!("Failed to read target file: {:?}", target_path))?;
    let target_input: TargetInput = serde_json::from_str(&target_json)
        .with_context(|| format!("Failed to parse target JSON: {:?}", target_path))?;

    // Resolve player state
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
        .with_context(|| "Failed to resolve equipment")?;

    // Resolve target
    let target = match target_input {
        TargetInput::ById(ref by_id) => {
            let mut resolved = resolver
                .resolve_monster(by_id.monster_id)
                .with_context(|| format!("Failed to resolve monster {}", by_id.monster_id))?;

            // Apply overrides if any
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

    // Create eval context and evaluate
    let ctx = EvalContext {
        player: &player,
        build: &build,
        target: &target,
    };

    let result = evaluate(&ctx);

    // Output results
    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("=== DPS Evaluation ===");
        println!();
        println!("Target: {}", target.name);
        println!();
        println!("Results:");
        println!("  DPS:            {:.4}", result.dps);
        println!("  Max Hit:        {}", result.max_hit);
        println!("  Accuracy:       {:.2}%", result.accuracy * 100.0);
        println!("  Attack Roll:    {}", result.attack_roll);
        println!("  Defence Roll:   {}", result.defence_roll);
        println!(
            "  Attack Speed:   {} ticks ({:.1}s)",
            result.interval_ticks,
            result.interval_ticks as f64 * 0.6
        );

        if explain {
            println!();
            println!("=== Breakdown ===");
            println!();
            println!("Player Stats:");
            println!(
                "  Attack:   {} (base) + {} (potion) * {}/{} (prayer)",
                player.attack,
                player.potion_attack_boost,
                player.prayer_attack_mult.0,
                player.prayer_attack_mult.1
            );
            println!(
                "  Strength: {} (base) + {} (potion) * {}/{} (prayer)",
                player.strength,
                player.potion_strength_boost,
                player.prayer_strength_mult.0,
                player.prayer_strength_mult.1
            );
            println!();
            println!("Equipment Bonuses:");
            println!(
                "  Attack ({:?}): {}",
                build.attack_type,
                build.bonuses.attack_bonus_for(build.attack_type)
            );
            println!("  Melee Strength: {}", build.bonuses.melee_strength);
            println!();
            println!("Target:");
            println!("  Defence Level: {}", target.defence_level);
            println!(
                "  Defence ({:?}): {}",
                build.attack_type,
                target.defence_bonuses.defence_bonus_for(build.attack_type)
            );
            println!("  Attributes: {:?}", target.attributes);
        }
    }

    Ok(())
}

fn cmd_validate_data(data_dir: &PathBuf) -> Result<()> {
    println!("Validating data in {:?}...", data_dir);

    let store = DataStore::load_from_dir(data_dir)
        .with_context(|| format!("Failed to load data from {:?}", data_dir))?;

    println!("Loaded {} items", store.items.len());
    println!("Loaded {} monsters", store.monsters.len());

    // Basic validation
    let mut warnings = 0;

    for (id, item) in &store.items {
        if item.equipable && item.equipment.is_none() {
            println!(
                "Warning: Item {} ({}) is equipable but has no equipment data",
                id, item.name
            );
            warnings += 1;
        }
        if item.equipable_weapon && item.weapon.is_none() {
            println!(
                "Warning: Item {} ({}) is a weapon but has no weapon data",
                id, item.name
            );
            warnings += 1;
        }
    }

    for (id, monster) in &store.monsters {
        if monster.hitpoints == 0 {
            println!("Warning: Monster {} ({}) has 0 hitpoints", id, monster.name);
            warnings += 1;
        }
    }

    if warnings > 0 {
        println!();
        println!("Found {} warnings", warnings);
    } else {
        println!("All data validated successfully!");
    }

    Ok(())
}

fn cmd_item_info(id: u32, data_dir: &PathBuf) -> Result<()> {
    let store = DataStore::load_from_dir(data_dir)
        .with_context(|| format!("Failed to load data from {:?}", data_dir))?;

    let item = store.get_item(id)?;

    println!("=== Item: {} (ID: {}) ===", item.name, item.id);
    println!();
    println!("Equipable: {}", item.equipable);
    println!("Is Weapon: {}", item.equipable_weapon);

    if let Some(ref eq) = item.equipment {
        println!();
        println!("Equipment Stats:");
        println!("  Slot: {}", eq.slot);
        println!();
        println!("  Attack Bonuses:");
        println!("    Stab:   {:+}", eq.attack_stab);
        println!("    Slash:  {:+}", eq.attack_slash);
        println!("    Crush:  {:+}", eq.attack_crush);
        println!("    Magic:  {:+}", eq.attack_magic);
        println!("    Ranged: {:+}", eq.attack_ranged);
        println!();
        println!("  Defence Bonuses:");
        println!("    Stab:   {:+}", eq.defence_stab);
        println!("    Slash:  {:+}", eq.defence_slash);
        println!("    Crush:  {:+}", eq.defence_crush);
        println!("    Magic:  {:+}", eq.defence_magic);
        println!("    Ranged: {:+}", eq.defence_ranged);
        println!();
        println!("  Other:");
        println!("    Melee Str:  {:+}", eq.melee_strength);
        println!("    Ranged Str: {:+}", eq.ranged_strength);
        println!("    Magic Dmg:  {}%", eq.magic_damage);
        println!("    Prayer:     {:+}", eq.prayer);
    }

    if let Some(ref wpn) = item.weapon {
        println!();
        println!("Weapon Data:");
        println!(
            "  Attack Speed: {} ticks ({:.1}s)",
            wpn.attack_speed,
            wpn.attack_speed as f64 * 0.6
        );
        println!("  Weapon Type:  {}", wpn.weapon_type);
        println!();
        println!("  Stances:");
        for stance in &wpn.stances {
            println!(
                "    - {} ({}) -> {} XP",
                stance.combat_style, stance.attack_type, stance.experience
            );
        }
    }

    Ok(())
}

fn cmd_monster_info(id: u32, data_dir: &PathBuf) -> Result<()> {
    let store = DataStore::load_from_dir(data_dir)
        .with_context(|| format!("Failed to load data from {:?}", data_dir))?;

    let monster = store.get_monster(id)?;

    println!("=== Monster: {} (ID: {}) ===", monster.name, monster.id);
    println!();
    println!("Combat Level: {}", monster.combat_level);
    println!("Hitpoints:    {}", monster.hitpoints);
    println!("Size:         {}", monster.size);
    println!();
    println!("Combat Stats:");
    println!("  Attack:   {}", monster.attack_level);
    println!("  Strength: {}", monster.strength_level);
    println!("  Defence:  {}", monster.defence_level);
    println!("  Magic:    {}", monster.magic_level);
    println!("  Ranged:   {}", monster.ranged_level);
    println!();
    println!("Defence Bonuses:");
    println!("  Stab:   {:+}", monster.defence_stab);
    println!("  Slash:  {:+}", monster.defence_slash);
    println!("  Crush:  {:+}", monster.defence_crush);
    println!("  Magic:  {:+}", monster.defence_magic);
    println!("  Ranged: {:+}", monster.defence_ranged);

    if !monster.attributes.is_empty() {
        println!();
        println!("Attributes: {:?}", monster.attributes);
    }

    if !monster.category.is_empty() {
        println!();
        println!("Categories: {:?}", monster.category);
    }

    if monster.slayer_monster {
        println!();
        println!("Slayer Info:");
        println!("  Slayer Monster: Yes");
        println!("  Slayer Level Required: {}", monster.slayer_level);
    }

    Ok(())
}
