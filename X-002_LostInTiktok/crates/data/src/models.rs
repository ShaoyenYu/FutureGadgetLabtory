use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 物品种类
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Weapon,
    Material,
    Consumable,
    Artifact,
}

/// 物品形状定义
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemShapeData {
    pub width: u8,
    pub height: u8,
    pub mask: Vec<bool>,
}

/// 词条生成权重配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffixWeightData {
    pub affix_type: String, // FlatDamage, MultiplierDamage, LifeSteal, Pierce, Bleed, AttackSpeed, RangeBoost
    pub value: f32,
    pub secondary_value: Option<f32>, // 用于 Bleed 等多参数
    pub duration: Option<f32>,
    pub weight: u32,
}

/// 物品配置数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDataConfig {
    pub id: String,
    pub name: String,
    pub item_type: ItemType,
    pub shape: ItemShapeData,
    #[serde(default)]
    pub base_damage: f32,
    #[serde(default = "default_attack_rate")]
    pub attack_rate: f32,
    #[serde(default = "default_attack_range")]
    pub attack_range: f32,
    #[serde(default = "default_projectile_count")]
    pub projectile_count: u8,
    #[serde(default)]
    pub possible_affixes: Vec<AffixWeightData>,
    #[serde(default)]
    pub icon_type: String,
    #[serde(default = "default_color_hex")]
    pub color_hex: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub sell_value: u32,
}

fn default_attack_rate() -> f32 { 1.0 }
fn default_attack_range() -> f32 { 50.0 }
fn default_projectile_count() -> u8 { 1 }
fn default_color_hex() -> String { "#ffffff".to_string() }

/// 物品库顶层配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemDatabaseConfig {
    pub items: HashMap<String, ItemDataConfig>,
}

/// 波次敌人权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveEnemyData {
    pub id: String,
    pub weight: u32,
}

/// 单个波次配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveData {
    pub time_start: f32,
    pub time_end: f32,
    pub spawn_interval: f32,
    pub enemies: Vec<WaveEnemyData>,
    pub max_concurrent: usize,
    #[serde(default = "default_spawn_batch")]
    pub spawn_batch_size: usize,
}

fn default_spawn_batch() -> usize { 3 }

/// 波次库配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WaveDatabaseConfig {
    pub waves: Vec<WaveData>,
}

/// 怪物属性配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyDataConfig {
    pub id: String,
    pub name: String,
    pub max_hp: f32,
    pub move_speed: f32,
    pub damage: f32,
    pub armor: f32,
    pub exp_reward: u32,
    pub gold_reward: u32,
    pub size: f32,
    pub color_hex: String,
    pub is_explosive: bool,
    pub is_ranged: bool,
    pub is_boss: bool,
    pub attack_cooldown: f32,
    #[serde(default)]
    pub drop_table: Vec<(String, u32)>, // item_id, weight
}

/// 怪物库配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnemyDatabaseConfig {
    pub enemies: HashMap<String, EnemyDataConfig>,
}

/// 天赋配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalentConfig {
    pub id: String,
    pub name: String,
    pub max_level: u32,
    pub cost_base: u32,
    pub cost_multiplier: f32,
    pub stat_type: String, // MaxHp, Damage, MoveSpeed, StashCapacity, Armor
    pub value_per_level: f32,
    pub description: String,
}

/// 天赋库配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TalentDatabaseConfig {
    pub talents: Vec<TalentConfig>,
}
