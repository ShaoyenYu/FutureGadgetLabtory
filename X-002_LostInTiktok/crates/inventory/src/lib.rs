pub mod components;
pub mod inventory;
pub mod item_shape;
pub mod systems;

use bevy::prelude::*;
use systems::*;

pub struct SoulforgeInventoryPlugin;

impl Plugin for SoulforgeInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, inventory_movement_handler_system);
    }
}
