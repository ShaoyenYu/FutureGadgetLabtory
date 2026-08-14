use bevy::prelude::*;

/// 全局资源，记录本局会话收益
#[derive(Resource, Debug, Clone)]
pub struct RunSessionContext {
    pub time_survived: f32,
    pub collected_items: Vec<String>, // 保存物品ID列表用于结算
    pub secured: bool,                // 是否成功搭乘矿车/撤离
    pub kills: u32,
    pub gold_earned: u32,
    pub soul_shards_earned: u32,
    pub score: u32,
    pub extraction_available: bool,
}

impl Default for RunSessionContext {
    fn default() -> Self {
        Self {
            time_survived: 0.0,
            collected_items: Vec::new(),
            secured: false,
            kills: 0,
            gold_earned: 0,
            soul_shards_earned: 0,
            score: 0,
            extraction_available: false,
        }
    }
}

/// 撤离点状态组件与资源
#[derive(Component, Debug, Clone)]
pub struct ExtractionPoint {
    pub active: bool,
    pub countdown_timer: Timer,
    pub radius: f32,
    pub channel_timer: Timer, // 站立撤离引导计时器
}

impl Default for ExtractionPoint {
    fn default() -> Self {
        Self {
            active: true,
            countdown_timer: Timer::from_seconds(60.0, TimerMode::Once),
            radius: 48.0,
            channel_timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}

/// 局内主计时器
#[derive(Resource, Debug, Clone)]
pub struct RunTimer {
    pub timer: Timer,
    pub total_seconds: f32,
    pub extraction_spawned: bool,
    pub next_extraction_spawn_time: f32,
}

impl Default for RunTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(300.0, TimerMode::Once),
            total_seconds: 0.0,
            extraction_spawned: false,
            next_extraction_spawn_time: 45.0, // 每45-60秒出现一次撤离矿车
        }
    }
}

/// 局内游戏是否暂停
#[derive(Resource, Debug, Clone, Default)]
pub struct GamePaused(pub bool);
