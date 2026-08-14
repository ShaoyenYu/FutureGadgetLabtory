# Soulforge: Lost in Tiktok (X-002)

> **Survivor-like ARPG + 战术异形网格背包 + 搜打撤 (Extraction) + 圣骸铁匠铺锻造**
> 
> 基于 **Rust (Edition 2021)** 与 **Bevy Engine (0.14)** 构建的高性能 2D 像素风硬核动作游戏。

---

## 🎮 游戏核心玩法与机制

1. **幸存者割草战斗 (Survivor ARPG)**：
   - 玩家操控主角在深渊地牢中迎战潮水般的腐化怨魂、自爆燃魂、窥视之瞳与肉山巨兽。
   - 拥有武器自动寻敌攻击机制（剑胚挥砍、影袭迅捷匕首、穿透重弩、环绕秘术脉冲球）。
   - 空格键战术翻滚冲刺，具有极限位移与无敌帧规避伤害。

2. **俄罗斯方块式异形网格背包 (Grid Inventory & Magic Slots)**：
   - 物品拥有不规则形状（1x3、2x2、L形等），支持在背包中按 **`R` 键进行 90° 顺时针旋转**。
   - 网格内置裁缝与附魔**魔法格（Magic Slots）**，物品覆盖魔法格时动态激活特殊属性加成与流血撕裂词条。
   - 严谨的越界检查（Bounds Check）与多实体重叠检查（Overlap Check）。

3. **搜刮与撤离循环 (Extraction Loop)**：
   - 局内计时达到阈值时在视野边缘刷新**撤离矿车/传送法阵**。
   - 玩家进入撤离点完成 3 秒充能引导即可成功撤离，将搜刮到的圣骸与战利品安全运回安全屋仓库。
   - 若局内阵亡则清空未绑定战利品，仅保留保底灵魂碎片。

4. **安全屋营地与圣骸熔铸 (Blacksmith & Meta Progression)**：
   - **铁匠铺 (Forge)**：消耗仓库中搜刮出的圣骸材料，根据概率权重表（Loot Table）为武器熔铸随机强力词条（基础伤害、伤害乘区、吸血、穿透、撕裂流血 DOT 等）。
   - **天赋神坛 (Talents)**：消耗金币与灵魂碎片升级被动属性（生命上限、基础攻击、移动速度、护甲减伤）。
   - **存档持久化**：全局资产与仓库自动持久化存储至 `savegame.json`。

---

## 🏗️ 架构与 Workspace 模块划分

本项目采用严格的 **ECS (Entity-Component-System)** 设计模式与 Cargo Workspace 架构：

```
X-002_LostInTiktok/
├── assets/
│   └── data/               # 数据驱动 RON 配置文件
│       ├── items.ron       # 武器、圣骸材料、消耗品定义
│       ├── waves.ron       # 怪物波次、权重、生成间隔与上限
│       ├── enemies.ron     # 怪物属性、攻击方式与掉落表
│       └── talents.ron     # 局外天赋升级数值与消耗
├── crates/
│   ├── core/               # 全局状态机 (AppState)、解耦事件总线、空间哈希 (SpatialHash2D)
│   ├── data/               # serde + ron 数据反序列化与嵌入回退
│   ├── inventory/          # 异形背包、矩阵转置旋转算法、魔法格修饰器
│   ├── combat/             # 属性面板、武器攻击、伤害结算管线、群怪 Boids AI、投射物与跳字
│   ├── meta/               # 铁匠铺锻造、撤离管线、天赋系统、存档序列化
│   ├── render/             # 2D 动画 ECS 模型、程序化像素贴图生成、摄像机震屏、地砖渲染
│   └── game/               # 主程序入口、MainMenu UI、BaseCamp UI、HUD 与背包交互弹窗
├── Cargo.toml              # Workspace 根配置
├── project_spec.md         # 详细工程技术规格说明书
└── README.md               # 项目介绍与运行说明
```

### 核心子 Crate 职责清单

| Crate 名称 | 对应 Package | 核心职责 |
|---|---|---|
| `crates/core` | `soulforge_core` | 全局状态机 (`AppState`)、7大核心解耦事件、高性能 `SpatialHash2D` 空间分区索引 |
| `crates/data` | `soulforge_data` | `serde` + `ron` 数据驱动模型，支持热加载与编译期内嵌回退 |
| `crates/inventory` | `soulforge_inventory` | `ItemShape` 90度顺时针转置矩阵、`Inventory` 放置与重叠校验、魔法格 Buff |
| `crates/combat` | `soulforge_combat` | `CombatStats`、`Weapon`、`Affixes`、`DamageEvent` 护甲减免公式计算、Boids 分离力集群 AI |
| `crates/meta` | `soulforge_meta` | 铁匠铺随机词条锻造、撤离点生成与充能结算、`PersistentSaveData` 存档持久化 |
| `crates/render` | `soulforge_render` | 纯程序化 2D 像素风美术生成、`AnimationState` 动画状态机、受击震屏与地牢地块 |
| `crates/game` | `soulforge_game` | 主游戏二进制入口、生命周期管理与完整 UI 交互系统（主菜单/营地/HUD/网格背包/结算） |

---

## ⌨️ 游戏操作指南

| 按键 | 功能说明 |
|---|---|
| **`W / A / S / D`** 或 **方向键** | 控制主角八方向平滑移动 |
| **`SPACE` (空格键)** | 战术翻滚冲刺（极速位移 + 期间享有无敌帧） |
| **`TAB` / `I`** | 开启 / 关闭战术异形网格背包 |
| **`R`** | 在背包开启状态下，顺时针旋转当前选中的物品 90° |
| **鼠标左键** | 点击 UI 按钮 / 选中背包物品 / 点击目标网格进行移动放置 |

---

## 🚀 编译、测试与运行

### 1. 运行所有单元测试
```powershell
cargo test --workspace
```
测试覆盖：
- 空间哈希半球与圆形范围查询及 Boids 排斥力计算
- 俄罗斯方块式多边形矩阵顺时针 90° 旋转算法
- 异形背包越界检查、非矩形格子解锁与多实体防重叠检测
- 魔法格 Buff 动态覆盖检索
- 伤害管线公式、护甲免伤系数与词条乘区计算
- 天赋神坛升级消费与属性注入

### 2. 启动游戏
```powershell
cargo run --bin soulforge_game
```

---

## 📐 核心公式与算法说明

1. **矩阵 90° 顺时针旋转算法**：
   $$\text{NewWidth} = \text{OldHeight}, \quad \text{NewHeight} = \text{OldWidth}$$
   $$(x, y) \mapsto (\text{OldHeight} - 1 - y, x)$$

2. **伤害结算公式 (Damage Mitigation Pipeline)**：
   $$\text{Mitigation} = \frac{\text{Armor}}{\text{Armor} + 50}$$
   $$\text{Final Damage} = (\text{Base Damage} + \text{Flat Damage}) \times (1 + \text{Multiplier}) \times (1 - \text{Mitigation})$$

3. **群怪无 $O(N^2)$ 空间哈希排斥力 (Boids Separation)**：
   $$\vec{F}_{\text{separation}} = \frac{1}{K} \sum_{i=1}^{K} \frac{\vec{p}_{\text{self}} - \vec{p}_i}{\|\vec{p}_{\text{self}} - \vec{p}_i\|} \cdot \left(1 - \frac{\|\vec{p}_{\text{self}} - \vec{p}_i\|}{R_{\text{sep}}}\right)$$
