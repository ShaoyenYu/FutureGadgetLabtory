# 游戏工程规格说明书 (Game Specification Document)

**Project:** Soulforge: Lost in Tiktok (Survivor + Grid Inventory + Extraction + Weapon Forging)  
**Tech Stack:** Rust (Edition 2021) + Bevy Engine (0.14)  
**Genre:** Survivor-like ARPG + Grid Inventory Management + Extraction + Weapon Forging  
**Repository Architecture:** Cargo Workspace with Decoupled ECS Crates

---

## 1. 架构总览 & 模块划分 (Architecture & Workspace)

本项目严格遵循 **ECS (Entity-Component-System)** 设计模式，采用 Cargo Workspace 划分模块以确保秒级增量编译与代码深度解耦。

### 1.1 Workspace 子工程划分

```
crates/
├── core/         # soulforge_core: 全局状态机 (AppState)、事件总线、SpatialHash2D 空间索引、Z-Index 常数
├── data/         # soulforge_data: serde + ron 配置文件加载、数据驱动模型与编译期内嵌回退
├── inventory/    # soulforge_inventory: 网格背包核心数据结构、俄罗斯方块 90° 旋转算法、防重叠校验、魔法格修饰器
├── combat/       # soulforge_combat: 角色/怪物面板、武器攻击、伤害结算管线、Boids 排斥力群怪 AI、投射物与伤害跳字
├── meta/         # soulforge_meta: 局外安全屋铁匠铺、圣骸熔铸随机词条、搜打撤流程控制、天赋升级、存档持久化
├── render/       # soulforge_render: 2D 帧动画 ECS 模型、程序化 2D 像素风贴图生成器、摄像机跟随与震屏、地砖平铺
└── game/         # soulforge_game: 主程序入口、多状态 UI 整合 (MainMenu, BaseCamp, InRun HUD, Grid Inventory, GameOver)
```

### 1.2 核心状态机 (State Machine)

定义全局 `AppState` 驱动系统流转，严格隔离局内与局外逻辑：

```rust
#[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Loading,         // 资产与 RON 配置文件预加载
    MainMenu,        // 主菜单界面（出击、安全屋、退出）
    BaseCamp,        // 局外：安全屋营地（铁匠铺圣骸熔铸、仓库整理、天赋神坛）
    RunSpawning,     // 局内：地图生成、主角/背包/初始武器初始化
    InRun,           // 局内：核心割草战斗、掉落物搜刮与波次推进
    Extraction,      // 局内：矿车撤离法阵激活、引导充能与倒计时
    GameOver,        // 局内：阵亡或撤离完成的结算战报界面
}
```

---

## 2. 核心机制设计与 ECS 建模 (Core Mechanics)

### 2.1 实体与战斗属性 (`soulforge_combat`)

玩家通过基础“剑胚”叠加“圣骸(词条)”。伤害受武器基础、玩家面板、词条乘区、怪物护甲多重影响。

```rust
// 实体面板属性
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct CombatStats {
    pub max_hp: f32,
    pub current_hp: f32,
    pub base_damage: f32,
    pub move_speed: f32,
    pub armor: f32,
}

// 武器构筑组件（挂载在武器实体上）
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

// 词条池 (圣骸系统)
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Affixes {
    pub mods: Vec<AffixModifier>,
}

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
```

### 2.2 俄罗斯方块异形网格背包与魔法格 (`soulforge_inventory`)

物品形状支持任意不规则多边形（俄罗斯方块）；背包支持异形非矩形单元解锁；支持裁缝赋予的特殊魔法格（Magic Slots）。

```rust
// 物品形状与顺时针 90° 旋转定义
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemShape {
    pub width: u8,
    pub height: u8,
    pub mask: Vec<bool>, // 1D 数组模拟 2D 矩阵，行优先存储
}

impl ItemShape {
    /// 顺时针旋转 90 度的矩阵转置算法: (x, y) 映射为 (height - 1 - y, x)
    pub fn rotate_90(&mut self) {
        let old_w = self.width as usize;
        let old_h = self.height as usize;
        let new_w = old_h;
        let new_h = old_w;
        let mut new_mask = vec![false; new_w * new_h];

        for y in 0..old_h {
            for x in 0..old_w {
                let old_idx = y * old_w + x;
                let val = self.mask.get(old_idx).copied().unwrap_or(false);
                let new_x = old_h - 1 - y;
                let new_y = x;
                let new_idx = new_y * new_w + new_x;
                if new_idx < new_mask.len() {
                    new_mask[new_idx] = val;
                }
            }
        }
        self.width = new_w as u8;
        self.height = new_h as u8;
        self.mask = new_mask;
    }
}

// 背包格子状态定义
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridSlot {
    pub item_entity: Option<Entity>,
    pub item_id: Option<String>,
    pub magic_buff: Option<String>, // 裁缝/附魔赋予的特殊 Buff ID (如 "magic_dmg_slot")
}

// 核心背包组件 (挂载在 Player 或 BaseCamp Storage 上)
#[derive(Component, Debug, Clone)]
pub struct Inventory {
    pub max_width: u8,
    pub max_height: u8,
    pub unlocked_cells: HashSet<(u8, u8)>, // 支持非矩形异形背包解锁
    pub slots: HashMap<(u8, u8), GridSlot>,
}
```

### 2.3 搜打撤流程控制 (Extraction Flow)

局内时间推进时触发撤离点刷新；进入撤离点完成引导充能带出全部搜刮物品；阵亡则扣除非绑定资产。

```rust
#[derive(Component, Debug, Clone)]
pub struct ExtractionPoint {
    pub active: bool,
    pub countdown_timer: Timer,
    pub radius: f32,
    pub channel_timer: Timer, // 引导充能计时器 (3.0s)
}

// 全局资源，记录本局会话收益
#[derive(Resource, Debug, Clone, Default)]
pub struct RunSessionContext {
    pub time_survived: f32,
    pub collected_items: Vec<String>, 
    pub secured: bool,                // 是否成功搭乘矿车撤离
    pub kills: u32,
    pub gold_earned: u32,
    pub soul_shards_earned: u32,
    pub score: u32,
    pub extraction_available: bool,
}
```

---

## 3. 核心解耦事件总线 (`soulforge_core::events`)

系统间完全通过 `#[derive(Event)]` 事件通道通信：

```rust
// --- 战斗伤害事件 ---
pub struct DamageEvent {
    pub source: Entity,
    pub target: Entity,
    pub base_damage: f32,
    pub is_bleed: bool,
    pub is_crit: bool,
}

// --- 掉落物生成事件 ---
pub struct LootDropEvent {
    pub item_id: String,
    pub world_position: Vec2,
}

// --- 背包移动与校验事件 ---
pub struct InventoryMoveEvent {
    pub item_entity: Entity,
    pub source_pos: Option<(u8, u8)>, // None 表示从地面拾取
    pub target_pos: Option<(u8, u8)>, // None 表示丢弃到地面
}

// --- 铁匠铺锻造请求事件 ---
pub struct ForgeRequestEvent {
    pub target_weapon: Entity,
    pub material_used: Entity,
}

// --- 撤离与死亡结算事件 ---
pub enum ExtractionEvent {
    Success { items: Vec<Entity> },
    Death { kept_items: Vec<Entity> },
}

// --- 浮动伤害跳字事件 ---
pub struct SpawnDamageTextEvent {
    pub position: Vec2,
    pub amount: f32,
    pub is_crit: bool,
    pub is_bleed: bool,
    pub is_heal: bool,
}

// --- 击杀奖励事件 ---
pub struct KillRewardEvent {
    pub position: Vec2,
    pub exp: u32,
    pub gold: u32,
    pub soul_shards: u32,
}
```

---

## 4. 核心系统管线实现与验收标准 (System Pipelines)

### 4.1 战斗管线 (`soulforge_combat::damage_pipeline`)
*   **空间哈希优化:** 严禁使用 $O(N^2)$ 的 `Query::iter_combinations`。通过 [`SpatialHash2D`](file:///C:/Users/Admin/Documents/PycharmProjects/FutureGadgetLabtory/X-002_LostInTiktok/crates/core/src/spatial_hash.rs) 统一处理怪物群聚排斥力与投射物碰撞查询。
*   **伤害公式:** 订阅 `DamageEvent`，使用 `ParamSet` 隔离读写别名：
    $$\text{Mitigation} = \frac{\text{Armor}}{\text{Armor} + 50}$$
    $$\text{Final Damage} = (\text{Base} + \text{Flat}) \times (1 + \text{Multiplier}) \times (1 - \text{Mitigation})$$
*   **吸血与流血状态:** 依据 `AffixModifier::LifeSteal` 实时回复玩家 HP；依据 `AffixModifier::Bleed` 挂载 `BleedStatus` 并按 0.5s 周期跳动流血 DOT 伤害。
*   **浮动跳字:** 依据暴击、流血、治疗不同类型派发 `SpawnDamageTextEvent` 并在世界空间漂浮淡出。

### 4.2 背包管线 (`soulforge_inventory::systems`)
*   **越界与重叠检查:** 订阅 `InventoryMoveEvent`，调用 `Inventory::can_place_item` 进行逐单元 mask 校验与已解锁检查。
*   **魔法格 Buff 识别:** 物品落位时调用 `Inventory::get_magic_buffs_under_item` 提取当前格子的附魔修饰器。

### 4.3 撤离与结算管线 (`soulforge_meta::flow_systems`)
*   **生成规则:** 监听局内 `RunTimer`，达到设定时间间隔（如 45s~60s）在玩家可视范围外 (280~360px) 刷新 `ExtractionPoint`。
*   **结算规则:**
    *   **Success**: 将临时背包中搜刮的所有物品转移至局外持久化仓库 `save_data.stash_items`，保存金币与灵魂碎片，转入 `AppState::BaseCamp`。
    *   **Death**: 清空本局搜刮物资，保留保底灵魂碎片，转入 `AppState::GameOver`。

### 4.4 铁匠铺锻造管线 (`soulforge_meta::forging_systems`)
*   **合成规则:** 触发条件 `AppState::BaseCamp`。接收 `ForgeRequestEvent`，消费选定的 `material_used` 圣骸材料实体，按配置表中的 Loot Table 权重（`AffixWeightData`）掷骰子随机抽取词条并注入目标武器的 `Affixes` 组件中。

---

## 5. 2D 渲染与程序化像素美术 (`soulforge_render`)

*   **2D 动画 ECS 模型:**
    ```rust
    #[derive(Component, Default, PartialEq, Eq)]
    pub enum AnimationState {
        #[default] Idle, Walk, Attack, Death,
    }

    #[derive(Component)]
    pub struct AnimationTimer {
        pub timer: Timer,
        pub loop_anim: bool,
        pub frame_count: usize,
        pub current_frame: usize,
    }
    ```
*   **程序化像素贴图生成器 (`pixel_generator.rs`):**
    *   主角：32x32 骑士/铁匠模型，支持走动与待机微动。
    *   基础群怪 (Corrupted Soul)：16x16 灰白怨魂，自爆怪通过 `Sprite::color` 设为 `srgba(1.0, 0.25, 0.25, 0.9)` 调色盘变种。
    *   肉山巨兽 (Flesh Brute)：32x32 重甲巨兽。
    *   窥视之瞳 (Eye Stalker)：24x24 触手紫瞳，发射 8x8 能量弹幕。
    *   刀光与 VFX：64x64 弧形刀光特效图。
    *   地牢地砖：32x32 石砖贴图动态循环平铺。
*   **渲染层级 (`Z-Index`):**
    背景(0.0) -> 地表(0.5) -> 陷阱/撤离法阵(1.5) -> 掉落物(2.0) -> 怪物(5.0) -> 玩家(10.0) -> 投射物/刀光/跳字(15~25.0) -> UI(100.0)。
*   **摄像机:** 支持平滑跟随与受击震屏衰减（`CameraScreenShake`）。

---

## 6. 数据驱动配置标准 (`assets/data/*.ron`)

所有游戏数值与波次均由外部 RON 文件驱动，提供编译期嵌入与运行时热加载回退保障：

1. [`assets/data/items.ron`](file:///C:/Users/Admin/Documents/PycharmProjects/FutureGadgetLabtory/X-002_LostInTiktok/assets/data/items.ron): 武器胚子、圣骸材料、药剂与结晶配置。
2. [`assets/data/waves.ron`](file:///C:/Users/Admin/Documents/PycharmProjects/FutureGadgetLabtory/X-002_LostInTiktok/assets/data/waves.ron): 关卡波次区间、怪物权重、生成间隔与最大同屏上限。
3. [`assets/data/enemies.ron`](file:///C:/Users/Admin/Documents/PycharmProjects/FutureGadgetLabtory/X-002_LostInTiktok/assets/data/enemies.ron): 怪物基础属性、移速、攻击冷却与掉落表。
4. [`assets/data/talents.ron`](file:///C:/Users/Admin/Documents/PycharmProjects/FutureGadgetLabtory/X-002_LostInTiktok/assets/data/talents.ron): 局外天赋升级成本指数与属性增幅。

---

## 7. 验收测试结论

| 测试项目 | 验证结果 | 说明 |
|---|---|---|
| `cargo check --workspace` | **PASS (0 errors, 0 warnings)** | 全 Workspace 静态检查通过 |
| `cargo test --workspace` | **PASS (6 tests passed)** | 矩阵旋转、网格防重叠、空间哈希、伤害公式、天赋升级单测全通过 |
| `cargo build --bin soulforge_game` | **PASS** | 二进制构建成功 (`target/debug/soulforge_game.exe`) |
| `cargo run` 实际启动验证 | **PASS (Running)** | 窗口成功弹出，GPU Vulkan 初始化正常，UI 与主循环稳定运行 |