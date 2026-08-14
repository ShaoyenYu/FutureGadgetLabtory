use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use soulforge_data::models::ItemType;

/// 背包格子的状态定义
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridSlot {
    #[serde(skip)]
    pub item_entity: Option<Entity>,
    pub item_id: Option<String>,
    pub magic_buff: Option<String>, // 裁缝/附魔赋予的特殊 Buff ID (如 "bonus_damage_20", "extra_pierce", "lifesteal_slot")
}

impl Default for GridSlot {
    fn default() -> Self {
        Self {
            item_entity: None,
            item_id: None,
            magic_buff: None,
        }
    }
}

/// 物品组件（挂载在物品实体上）
#[derive(Component, Debug, Clone)]
pub struct ItemComponent {
    pub item_id: String,
    pub name: String,
    pub item_type: ItemType,
    pub base_damage: f32,
    pub attack_rate: f32,
    pub attack_range: f32,
    pub projectile_count: u8,
    pub color_hex: String,
    pub description: String,
    pub is_equipped: bool,
    pub bound_to_player: bool, // 是否局内死亡不掉落的保底物品
}

/// 物品在背包中的左上角锚点坐标与所属背包标记
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemGridPosition {
    pub grid_x: u8,
    pub grid_y: u8,
    pub is_in_base_stash: bool, // 区分局内背包 vs 局外安全屋仓库
}

/// 装备槽位类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipSlotType {
    MainWeapon,
    SubWeapon,
    Relic1,
    Relic2,
}
