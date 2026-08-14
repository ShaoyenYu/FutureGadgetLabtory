use crate::models::*;
use bevy::prelude::*;
use std::fs;
use std::path::Path;

/// 游戏配置数据资源中心
#[derive(Resource, Debug, Clone, Default)]
pub struct GameDatabase {
    pub items: ItemDatabaseConfig,
    pub waves: WaveDatabaseConfig,
    pub enemies: EnemyDatabaseConfig,
    pub talents: TalentDatabaseConfig,
}

pub const DEFAULT_ITEMS_RON: &str = include_str!("../../../assets/data/items.ron");
pub const DEFAULT_WAVES_RON: &str = include_str!("../../../assets/data/waves.ron");
pub const DEFAULT_ENEMIES_RON: &str = include_str!("../../../assets/data/enemies.ron");
pub const DEFAULT_TALENTS_RON: &str = include_str!("../../../assets/data/talents.ron");

impl GameDatabase {
    pub fn load_from_disk_or_default() -> Self {
        let items = Self::load_file_or_fallback("assets/data/items.ron", DEFAULT_ITEMS_RON)
            .unwrap_or_else(|e| {
                warn!("Failed to load items.ron, fallback to default: {:?}", e);
                ron::from_str(DEFAULT_ITEMS_RON).expect("Default items.ron must parse")
            });

        let waves = Self::load_file_or_fallback("assets/data/waves.ron", DEFAULT_WAVES_RON)
            .unwrap_or_else(|e| {
                warn!("Failed to load waves.ron, fallback to default: {:?}", e);
                ron::from_str(DEFAULT_WAVES_RON).expect("Default waves.ron must parse")
            });

        let enemies = Self::load_file_or_fallback("assets/data/enemies.ron", DEFAULT_ENEMIES_RON)
            .unwrap_or_else(|e| {
                warn!("Failed to load enemies.ron, fallback to default: {:?}", e);
                ron::from_str(DEFAULT_ENEMIES_RON).expect("Default enemies.ron must parse")
            });

        let talents = Self::load_file_or_fallback("assets/data/talents.ron", DEFAULT_TALENTS_RON)
            .unwrap_or_else(|e| {
                warn!("Failed to load talents.ron, fallback to default: {:?}", e);
                ron::from_str(DEFAULT_TALENTS_RON).expect("Default talents.ron must parse")
            });

        Self {
            items,
            waves,
            enemies,
            talents,
        }
    }

    fn load_file_or_fallback<T: for<'de> serde::Deserialize<'de>>(
        path_str: &str,
        fallback_ron: &str,
    ) -> Result<T, ron::error::SpannedError> {
        let path = Path::new(path_str);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(data) = ron::from_str::<T>(&content) {
                    return Ok(data);
                }
            }
        }
        ron::from_str::<T>(fallback_ron)
    }
}
