pub mod base_camp;
pub mod game_over;
pub mod hud;
pub mod inventory_ui;
pub mod main_menu;

use base_camp::*;
use bevy::prelude::*;
use game_over::*;
use hud::*;
use inventory_ui::*;
use main_menu::*;
use soulforge_core::states::AppState;

pub struct SoulforgeUiPlugin;

impl Plugin for SoulforgeUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedInventoryItem>()
            // 1. 主菜单生命周期
            .add_systems(OnEnter(AppState::MainMenu), setup_main_menu_system)
            .add_systems(Update, main_menu_interaction_system.run_if(in_state(AppState::MainMenu)))
            .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu_system)
            // 2. 安全屋 / 铁匠铺生命周期
            .add_systems(OnEnter(AppState::BaseCamp), setup_base_camp_system)
            .add_systems(Update, base_camp_interaction_system.run_if(in_state(AppState::BaseCamp)))
            .add_systems(OnExit(AppState::BaseCamp), cleanup_base_camp_system)
            // 3. 局内 HUD 生命周期
            .add_systems(OnEnter(AppState::InRun), setup_hud_system)
            .add_systems(Update, update_hud_system.run_if(in_state(AppState::InRun).or_else(in_state(AppState::Extraction))))
            .add_systems(OnExit(AppState::InRun), cleanup_hud_system)
            // 4. 网格背包交互 (InRun & Extraction 下通过 Tab / I 开关)
            .add_systems(
                Update,
                (
                    toggle_inventory_system,
                    rotate_selected_item_system,
                    inventory_interaction_system,
                )
                    .run_if(in_state(AppState::InRun).or_else(in_state(AppState::Extraction))),
            )
            // 5. 结算界面 / GameOver 生命周期
            .add_systems(OnEnter(AppState::GameOver), setup_game_over_settlement_system)
            .add_systems(Update, settlement_interaction_system.run_if(in_state(AppState::GameOver)))
            .add_systems(OnExit(AppState::GameOver), cleanup_settlement_system);
    }
}
