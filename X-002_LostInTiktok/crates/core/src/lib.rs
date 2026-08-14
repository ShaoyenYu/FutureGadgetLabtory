pub mod constants;
pub mod events;
pub mod resources;
pub mod spatial_hash;
pub mod states;

use bevy::prelude::*;

pub struct SoulforgeCorePlugin;

impl Plugin for SoulforgeCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<states::AppState>()
            .init_resource::<resources::RunSessionContext>()
            .init_resource::<resources::RunTimer>()
            .init_resource::<resources::GamePaused>()
            .init_resource::<spatial_hash::SpatialHash2D>()
            .add_event::<events::DamageEvent>()
            .add_event::<events::LootDropEvent>()
            .add_event::<events::InventoryMoveEvent>()
            .add_event::<events::ForgeRequestEvent>()
            .add_event::<events::ExtractionEvent>()
            .add_event::<events::SpawnDamageTextEvent>()
            .add_event::<events::KillRewardEvent>();
    }
}
