### 项目模块架构文档

该项目是一个基于 Bevy 引擎开发的双人贪吃蛇游戏，视觉风格为「糖果软萌像素风」（Candy Kawaii）。所有像素美术在启动时由代码生成，仓库中不含任何二进制素材。

#### 目录结构概览
```text
src/
├── main.rs              # 项目入口：App 配置、状态管理、系统注册与调度顺序
├── components.rs        # 定义 ECS 组件（Components）与资源、事件
├── constants.rs         # 游戏常量与调色板（Candy Kawaii palette）
├── pixel_art.rs         # 像素美术：字符画 + 调色板 → 运行时烘焙为 Image 纹理
└── systems/             # 游戏逻辑系统模块
    ├── mod.rs           # 模块导出文件，整合所有系统模块
    ├── snake.rs         # 贪吃蛇逻辑（移动、进食、生长、死亡）与蛇身外观同步
    ├── environment.rs   # 环境逻辑（苹果生成、炸弹陷阱生成与倒计时）
    ├── ui.rs            # 用户界面（HUD 玩家卡、暂停/设置菜单、按钮交互）
    └── render.rs        # 渲染布局（棋盘、边框、坐标换算、缩放、呼吸动画）
```

#### 模块详细说明

1.  **`main.rs` (核心控制层)**
    *   作为 Bevy 应用的配置中心，启用 `ImagePlugin::default_nearest()` 保证像素art放大后不被平滑。
    *   定义游戏状态 (`GameState`)，注册资源 (`Resources`) 与事件 (`Events`)。
    *   `Startup` 阶段以 `.chain()` 保证 `setup_pixel_assets` 先于所有绘制系统执行。
    *   `Update` 分为两组：仅 `Playing` 状态运行的游戏逻辑，以及始终运行的布局 / HUD / 输入系统（暂停时窗口缩放也能正确重排）。

2.  **`components.rs` (数据定义层)**
    *   游戏数据结构：`Position`、`Size`、`SnakeHead`、`Trap` 等。
    *   UI 相关组件：`PlayerCard`、`PlayerLabel`、`HeartIcon`、`ButtonTheme`。
    *   表现层组件：`Pulse`（呼吸缩放）、`ArenaFrame`（棋盘外框）。

3.  **`constants.rs` (配置层)**
    *   网格尺寸 (`ARENA_WIDTH` / `ARENA_HEIGHT`)、HUD 高度 (`HUD_HEIGHT`)、HP 上限 (`MAX_HP`)。
    *   Candy Kawaii 调色板以 `[u8; 4]` RGBA 存储，同一份数值同时供纹理烘焙与 UI 使用；`col()` / `col_a()` 负责转换为 `Color`。

4.  **`pixel_art.rs` (美术层)**
    *   每个精灵以 16×16 字符画（`Art`）描述，配合调色板（`Palette`）映射字符 → 颜色，`bake()` 在启动时生成 `Image`。
    *   同一份字符画换调色板即可复用：两条蛇共用蛇身美术，满 / 空爱心共用心形美术。
    *   `PixelAssets` 资源持有全部纹理句柄，并提供 `head()` / `body()` / `tail()` 按玩家取图。
    *   自带单元测试：校验每行宽度为 16、字符均在对应调色板中定义（`cargo test`）。

5.  **`systems/` (业务逻辑层)**
    *   通过 `mod.rs` 暴露给主程序，按功能拆分为以下子模块：
        *   **`snake.rs`**: 移动输入、运动、进食、生长、死亡与复活；`update_snake_visuals` 负责蛇头朝向旋转、蛇尾贴图与朝向（含穿墙修正），`animate_blink` 负责眨眼。
        *   **`environment.rs`**: 苹果与炸弹陷阱的生成、陷阱倒计时与销毁。
        *   **`ui.rs`**: 顶部 HUD（玩家卡 / 分数 / 像素爱心 / K.O. 状态）、暂停与设置菜单面板、按钮主题与交互。
        *   **`render.rs`**: `playfield()` 计算正方形格子与居中偏移（避开 HUD），并负责棋盘、装饰小花、双层外框、坐标换算、尺寸缩放与呼吸动画。

#### 视觉与布局约定

*   **格子始终为正方形**：`playfield()` 取 `min(窗口宽 / 32, (窗口高 - HUD) / 18)`，避免像素art被拉伸；棋盘在 HUD 下方的区域内居中。
*   **Z 轴层次**：外框 `-0.2 / -0.1` → 棋盘 `0.0` → 装饰 `0.05` → 陷阱 `1.2` → 食物 `1.5` → 蛇身 `2.0` → 蛇头 `2.1` → 文字 `2.5 / 3.0`。
*   **精灵尺寸**：纹理 sprite 统一设置 `custom_size = Vec2::ONE`，实际大小完全由 `Size` × 格子边长决定，因此旋转不会产生形变。
*   **字体**：使用 Bevy 内置默认字体（子集），文本仅使用 ASCII 字符；血量、图标等一律使用像素纹理而非 emoji。
