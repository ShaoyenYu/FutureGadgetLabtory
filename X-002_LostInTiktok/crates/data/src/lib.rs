pub mod loader;
pub mod models;

use bevy::prelude::*;
use loader::GameDatabase;
use soulforge_core::states::AppState;

pub struct SoulforgeDataPlugin;

impl Plugin for SoulforgeDataPlugin {
    fn build(&self, app: &mut App) {
        let db = GameDatabase::load_from_disk_or_default();
        app.insert_resource(db)
            .add_systems(OnEnter(AppState::Loading), load_game_data_system);
    }
}

fn load_game_data_system(
    mut db: ResMut<GameDatabase>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    info!("Loading game configuration from data files / embedded RON...");
    *db = GameDatabase::load_from_disk_or_default();
    info!("Successfully loaded {} items, {} waves, {} enemies, {} talents",
        db.items.items.len(),
        db.waves.waves.len(),
        db.enemies.enemies.len(),
        db.talents.talents.len()
    );
    // 切换到 MainMenu
    next_state.set(AppState::MainMenu);
}
