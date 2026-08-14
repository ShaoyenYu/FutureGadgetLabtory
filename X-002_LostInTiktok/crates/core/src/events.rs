use bevy::prelude::*;

/// 战斗伤害事件
#[derive(Event, Debug, Clone)]
pub struct DamageEvent {
    pub source: Entity,
    pub target: Entity,
    pub base_damage: f32,
    pub is_bleed: bool,
    pub is_crit: bool,
}

/// 搜刮物品掉落事件
#[derive(Event, Debug, Clone)]
pub struct LootDropEvent {
    pub item_id: String,
    pub world_position: Vec2,
}

/// 网格背包移动/放置/丢弃事件
#[derive(Event, Debug, Clone)]
pub struct InventoryMoveEvent {
    pub item_entity: Entity,
    pub source_pos: Option<(u8, u8)>, // None 表示从地面或临时栏拾取
    pub target_pos: Option<(u8, u8)>, // None 表示丢弃到地面
}

/// 局外铁匠铺锻造请求事件
#[derive(Event, Debug, Clone)]
pub struct ForgeRequestEvent {
    pub target_weapon: Entity,
    pub material_used: Entity,
}

/// 撤离/死亡结算事件
#[derive(Event, Debug, Clone)]
pub enum ExtractionEvent {
    Success { items: Vec<Entity> },
    Death { kept_items: Vec<Entity> },
}

/// 浮动伤害跳字事件
#[derive(Event, Debug, Clone)]
pub struct SpawnDamageTextEvent {
    pub position: Vec2,
    pub amount: f32,
    pub is_crit: bool,
    pub is_bleed: bool,
    pub is_heal: bool,
}

/// 击杀结算/获得奖励事件
#[derive(Event, Debug, Clone)]
pub struct KillRewardEvent {
    pub position: Vec2,
    pub exp: u32,
    pub gold: u32,
    pub soul_shards: u32,
}
