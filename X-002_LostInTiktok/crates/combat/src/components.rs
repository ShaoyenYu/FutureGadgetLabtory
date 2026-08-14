use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 实体面板属性
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct CombatStats {
    pub max_hp: f32,
    pub current_hp: f32,
    pub base_damage: f32,
    pub move_speed: f32,
    pub armor: f32,
}

impl Default for CombatStats {
    fn default() -> Self {
        Self {
            max_hp: 100.0,
            current_hp: 100.0,
            base_damage: 10.0,
            move_speed: 130.0,
            armor: 0.0,
        }
    }
}

/// 武器类型分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponType {
    MeleeSlash,
    SwiftDagger,
    Crossbow,
    ArcaneOrb,
}

/// 武器构筑组件（挂载在武器实体上）
#[derive(Component, Debug, Clone)]
pub struct Weapon {
    pub id: String,
    pub weapon_type: WeaponType,
    pub base_attack_rate: f32,
    pub attack_range: f32,
    pub projectile_count: u8,
    pub cooldown_timer: Timer,
    pub active: bool,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            id: "sword_embryo_1".to_string(),
            weapon_type: WeaponType::MeleeSlash,
            base_attack_rate: 1.2,
            attack_range: 65.0,
            projectile_count: 1,
            cooldown_timer: Timer::from_seconds(1.0 / 1.2, TimerMode::Repeating),
            active: true,
        }
    }
}

/// 词条修饰器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AffixModifier {
    FlatDamage(f32),                                 // 附加基础伤害
    MultiplierDamage(f32),                           // 伤害乘区 (%)
    LifeSteal(f32),                                  // 吸血比例 (%)
    Pierce(u8),                                      // 穿透次数
    Bleed { chance: f32, dps: f32, duration: f32 },  // DOT 异常状态
    AttackSpeed(f32),                                // 攻击速度加成 (%)
    RangeBoost(f32),                                 // 攻击范围加成 (%)
}

/// 词条池 (圣骸系统)
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Affixes {
    pub mods: Vec<AffixModifier>,
}

/// 玩家角色组件
#[derive(Component, Debug, Clone)]
pub struct Player {
    pub level: u32,
    pub experience: u32,
    pub exp_to_next_level: u32,
    pub gold: u32,
    pub soul_shards: u32,
    pub kills: u32,
    pub dash_timer: Timer,
    pub dash_cooldown: Timer,
    pub is_dashing: bool,
    pub dash_direction: Vec2,
    pub invulnerability_timer: Timer,
    pub pickup_radius: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            level: 1,
            experience: 0,
            exp_to_next_level: 20,
            gold: 0,
            soul_shards: 0,
            kills: 0,
            dash_timer: Timer::from_seconds(0.2, TimerMode::Once),
            dash_cooldown: Timer::from_seconds(1.5, TimerMode::Once),
            is_dashing: false,
            dash_direction: Vec2::ZERO,
            invulnerability_timer: Timer::from_seconds(0.3, TimerMode::Once),
            pickup_radius: 64.0,
        }
    }
}

/// 怪物组件
#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub id: String,
    pub name: String,
    pub max_hp: f32,
    pub damage: f32,
    pub move_speed: f32,
    pub armor: f32,
    pub exp_reward: u32,
    pub gold_reward: u32,
    pub is_explosive: bool,
    pub is_ranged: bool,
    pub is_boss: bool,
    pub attack_cooldown: Timer,
    pub size: f32,
    pub exploding: bool,
    pub explosion_timer: Timer,
}

/// 投射物组件
#[derive(Component, Debug, Clone)]
pub struct Projectile {
    pub velocity: Vec2,
    pub damage: f32,
    pub pierce_remaining: u8,
    pub hit_entities: HashSet<Entity>,
    pub lifetime: Timer,
    pub is_player: bool,
    pub radius: f32,
    pub affixes: Vec<AffixModifier>,
}

/// 近战挥砍特效与伤害判定区
#[derive(Component, Debug, Clone)]
pub struct MeleeSlash {
    pub damage: f32,
    pub lifetime: Timer,
    pub hit_entities: HashSet<Entity>,
    pub radius: f32,
    pub arc_angle: f32,
    pub center_direction: Vec2,
    pub affixes: Vec<AffixModifier>,
}

/// 流血异常状态组件
#[derive(Component, Debug, Clone)]
pub struct BleedStatus {
    pub dps: f32,
    pub remaining_time: f32,
    pub tick_timer: Timer,
    pub source: Entity,
}

/// 地面掉落物组件
#[derive(Component, Debug, Clone)]
pub struct LootItem {
    pub item_id: String,
    pub magnet_active: bool,
    pub velocity: Vec2,
    pub despawn_timer: Timer,
}

/// 浮动伤害跳字组件
#[derive(Component, Debug, Clone)]
pub struct FloatingDamageText {
    pub timer: Timer,
    pub velocity: Vec2,
    pub initial_position: Vec2,
}
