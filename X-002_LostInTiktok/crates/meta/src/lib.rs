pub mod flow_systems;
pub mod forging_systems;
pub mod save_data;
pub mod talent_systems;

use bevy::prelude::*;
use flow_systems::*;
use forging_systems::*;
use save_data::PersistentSaveData;
use soulforge_core::resources::GamePaused;
use soulforge_core::states::AppState;

pub struct SoulforgeMetaPlugin;

impl Plugin for SoulforgeMetaPlugin {
    fn build(&self, app: &mut App) {
        let save_data = PersistentSaveData::load_or_default();
        app.insert_resource(save_data)
            .add_systems(
                Update,
                (
                    extraction_spawner_timer_system,
                    extraction_channeling_system,
                )
                    .run_if(in_state(AppState::InRun).or_else(in_state(AppState::Extraction)))
                    .run_if(is_not_paused),
            )
            .add_systems(
                Update,
                (
                    forging_pipeline_system,
                    extraction_settlement_system,
                ),
            );
    }
}

fn is_not_paused(paused: Res<GamePaused>) -> bool {
    !paused.0
}
