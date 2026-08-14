use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 存储在仓库中的物品存档结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredItemData {
    pub item_id: String,
    pub grid_x: u8,
    pub grid_y: u8,
    pub width: u8,
    pub height: u8,
    pub mask: Vec<bool>,
    pub affixes: Vec<soulforge_combat::components::AffixModifier>,
}

/// 局外持久化存档结构
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct PersistentSaveData {
    pub gold: u32,
    pub soul_shards: u32,
    pub high_score: u32,
    pub total_runs: u32,
    pub successful_extractions: u32,
    pub talent_levels: HashMap<String, u32>,
    pub stash_items: Vec<StoredItemData>,
    pub starting_weapon_id: String,
}

impl Default for PersistentSaveData {
    fn default() -> Self {
        let mut talents = HashMap::new();
        talents.insert("talent_hp".to_string(), 0);
        talents.insert("talent_damage".to_string(), 0);
        talents.insert("talent_speed".to_string(), 0);
        talents.insert("talent_armor".to_string(), 0);

        Self {
            gold: 150,
            soul_shards: 5,
            high_score: 0,
            total_runs: 0,
            successful_extractions: 0,
            talent_levels: talents,
            stash_items: vec![
                StoredItemData {
                    item_id: "sword_embryo_1".to_string(),
                    grid_x: 0,
                    grid_y: 0,
                    width: 1,
                    height: 3,
                    mask: vec![true, true, true],
                    affixes: Vec::new(),
                },
                StoredItemData {
                    item_id: "relic_blood".to_string(),
                    grid_x: 2,
                    grid_y: 0,
                    width: 1,
                    height: 1,
                    mask: vec![true],
                    affixes: Vec::new(),
                },
                StoredItemData {
                    item_id: "relic_sharpness".to_string(),
                    grid_x: 3,
                    grid_y: 0,
                    width: 1,
                    height: 1,
                    mask: vec![true],
                    affixes: Vec::new(),
                },
            ],
            starting_weapon_id: "sword_embryo_1".to_string(),
        }
    }
}

impl PersistentSaveData {
    const SAVE_FILE_PATH: &'static str = "savegame.json";

    pub fn load_or_default() -> Self {
        if Path::new(Self::SAVE_FILE_PATH).exists() {
            if let Ok(content) = fs::read_to_string(Self::SAVE_FILE_PATH) {
                if let Ok(data) = serde_json::from_str::<Self>(&content) {
                    info!("Loaded savegame from disk: {} gold, {} shards", data.gold, data.soul_shards);
                    return data;
                }
            }
        }
        info!("No previous savegame found, using default starting profile");
        Self::default()
    }

    pub fn save_to_disk(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if let Err(e) = fs::write(Self::SAVE_FILE_PATH, json) {
                warn!("Failed to save game to {}: {:?}", Self::SAVE_FILE_PATH, e);
            } else {
                info!("Game saved successfully to {}", Self::SAVE_FILE_PATH);
            }
        }
    }
}
