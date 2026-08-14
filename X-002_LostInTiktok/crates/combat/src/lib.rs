pub mod components;
pub mod damage_pipeline;
pub mod enemy_systems;
pub mod player_systems;
pub mod projectile_systems;
pub mod spawner_systems;
pub mod weapon_systems;

use bevy::prelude::*;
use damage_pipeline::*;
use enemy_systems::*;
use player_systems::*;
use projectile_systems::*;
use soulforge_core::resources::GamePaused;
use soulforge_core::states::AppState;
use spawner_systems::*;
use weapon_systems::*;

pub struct SoulforgeCombatPlugin;

impl Plugin for SoulforgeCombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaveSpawnerTimer>()
            .add_systems(
                Update,
                (
                    update_spatial_hash_system,
                    player_movement_system,
                    weapon_auto_attack_system,
                    enemy_swarm_ai_system,
                    projectile_update_system,
                    melee_slash_update_system,
                    damage_pipeline_system,
                    bleed_tick_system,
                    loot_pickup_system,
                    handle_loot_drop_events_system,
                    floating_damage_text_system,
                    wave_spawner_system,
                )
                    .chain()
                    .run_if(in_state(AppState::InRun).or_else(in_state(AppState::Extraction)))
                    .run_if(is_not_paused),
            );
    }
}

fn is_not_paused(paused: Res<GamePaused>) -> bool {
    !paused.0
}
