mod run_spawner;
mod ui;

use bevy::prelude::*;
use run_spawner::spawn_run_world_system;
use soulforge_combat::SoulforgeCombatPlugin;
use soulforge_core::states::AppState;
use soulforge_core::SoulforgeCorePlugin;
use soulforge_data::SoulforgeDataPlugin;
use soulforge_inventory::SoulforgeInventoryPlugin;
use soulforge_meta::SoulforgeMetaPlugin;
use soulforge_render::SoulforgeRenderPlugin;
use ui::SoulforgeUiPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Soulforge: Lost in Tiktok - Survivor & Grid Extraction".into(),
                        resolution: (1280.0_f32, 720.0_f32).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()), // 保证像素风最近邻采样，边缘清晰无模糊
        )
        // 核心插件集
        .add_plugins((
            SoulforgeCorePlugin,
            SoulforgeDataPlugin,
            SoulforgeRenderPlugin,
            SoulforgeInventoryPlugin,
            SoulforgeCombatPlugin,
            SoulforgeMetaPlugin,
            SoulforgeUiPlugin,
        ))
        // 局内初始化系统挂载
        .add_systems(OnEnter(AppState::RunSpawning), spawn_run_world_system)
        .run();
}
