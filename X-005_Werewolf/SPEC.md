# X-005 Werewolf（语音狼人杀）— 需求与架构 Spec

> 状态：v0.3 · 2026-09-28 · 分支 `dev_X-005_Werewolf`（独立 worktree）· **M0 已完成**
>
> v0.2 变更：确定"主机 + 手机"的游玩模式、本地语音模型、LLM 选型、默认 9 人局（人数可配）、MVP 包含警长竞选、UI 使用 Bevy。
>
> v0.3 变更：新增 Gemini 后端（Gemini API 直连 + Antigravity `agy` CLI）；M0 规则核心完成；信息过滤函数下沉到 ww-core（见 3.2、4.1）。

---

## 0. 一句话目标

用 Rust 编写的**基础版狼人杀**。一台电脑作为**主机**：用大屏展示公开信息，用音箱进行语音主持。1 名或多名真人**用自己的手机扫码加入**，私密信息的查看和夜间操作都在手机上完成；白天发言用语音。空余座位由 **LLM 驱动的 NPC** 补齐。每个玩家的信息由**上下文互相隔离的独立 agent/通道**维护，整局的发言和事件全部记录。

---

## 1. 需求梳理

### 1.1 已确认的决策

| # | 问题 | 决策 |
|---|------|------|
| D1 | 真人数量与设备 | 一台电脑作为主机，支持 **1–N 名真人**；每名真人通过**自己的手机浏览器**接入主机（局域网，扫码即可，不需要装 App），私密信息和夜间操作都在手机上 |
| D2 | 语音方案 | **全部使用本地模型**（sherpa-onnx：STT + VAD + TTS），可离线运行、零调用费用 |
| D3 | LLM 选型 | 默认 **`claude-sonnet-5`**，开启 prompt caching；可以按座位覆盖；另提供 `economy` 预设（全部用 `claude-haiku-4-5`）、**Gemini（API 直连 / Antigravity `agy` CLI）**，以及 OpenAI 兼容接口和本地 Ollama 后端。同一局里可以混用，例如 Claude 和 Gemini 同桌对战。理由见 2.3 |
| D4 | 板子 | 先开发**9 人局：3 狼 + 预女猎 + 3 民**；人数可配置为 6–12，按人数自动套用预设板子 |
| D5 | 警长 | **MVP 包含**完整的警长竞选、警徽流、归票、移交/撕毁警徽 |
| D6 | UI | 主机端使用 **Bevy**；手机端是一个由主机托管的轻量网页 |

### 1.2 功能需求

| ID | 需求 | 优先级 | 说明 |
|----|------|--------|------|
| F1 | 基础卡组：狼人 / 村民 / 预言家 / 女巫 / 猎人 | P0 | 白痴、守卫作为扩展（P2）；规则层要能插入新角色 |
| F2 | 人数可配置（6–12），默认 9 人 | P0 | 见 1.3 预设表 |
| F3 | 自动主持：昼夜状态机、结算、胜负判定 | P0 | 法官是**确定性规则引擎**，不是 LLM |
| F4 | 语音主持（本地 TTS，通过主机音箱播放） | P0 | 固定台词预先合成并缓存；动态播报实时合成 |
| F5 | 真人语音发言（本地 STT） | P0 | 默认用**主机麦克风**：引擎知道当前该谁发言，所以用一个麦克风就能对应到发言人；手机麦克风作为 P1（需要 HTTPS，见 6.3） |
| F6 | 手机接入：扫码入座、查看身份、夜间技能、投票、上警/退水 | P0 | 每名真人一个独立的 WebSocket 通道，只会收到属于自己的信息 |
| F7 | NPC 补位，LLM 后端可插拔 | P0 | 每个 NPC 有不同的音色和人设 |
| F8 | **信息隔离**（agent 之间、手机之间、主机大屏） | P0 | 见第 4 章 |
| F9 | 警长竞选 / 警徽流 / 归票 / 警徽移交 | P0 | 见 1.4 |
| F10 | 全量记录与回放 | P0 | 见第 7 章 |
| F11 | 无头模式：纯 NPC 自对局 | P0 | 用于测试、调 prompt、评估 |
| F12 | 狼人自爆 | P1 | 白天任意发言阶段（含警上）都可以自爆，直接进入黑夜 |
| F13 | VAD 自动断句（免按键） | P1 | 默认用按键说话（PTT） |
| F14 | 单人模式：真人不用手机，直接在主机上操作私密信息 | P2 | 仅限 1 名真人时，主机可以切换到该真人的私密视角 |

### 1.3 板子预设（`config/boards.toml`）

| 人数 | 狼 | 神 | 民 | 胜利条件 |
|------|----|----|----|----------|
| 6 | 2 | 预、女 | 2 | 屠边 |
| 7 | 2 | 预、女、猎 | 2 | 屠边 |
| 8 | 3 | 预、女、猎 | 2 | 屠边 |
| **9（默认）** | **3** | **预、女、猎** | **3** | **屠边** |
| 10 | 3 | 预、女、猎 | 4 | 屠边 |
| 11 | 4 | 预、女、猎 | 4 | 屠边 |
| 12 | 4 | 预、女、猎（+白痴 P2） | 4–5 | 屠边 |

> 也可以在配置里完全自定义角色组合；引擎启动时会校验组合是否合法（至少 1 狼、至少 1 名好人等）。

### 1.4 规则细节（默认值，均可配置）

**基础规则**
- **胜负**：屠边。狼人杀光所有神或所有民即获胜；放逐所有狼人则好人获胜。每次有人死亡后都检查一次。
- **夜晚顺序**：狼人 → 女巫 → 预言家。
- **女巫**：解药、毒药各一瓶；同一晚不能两瓶都用；**仅首夜可以自救**；解药用掉后不再告知刀口。
- **猎人**：出局时可以开枪带走一人；**被毒死不能开枪**。
- **狼人刀人**：多狼意见不一致时，以多数票为准；平票则取最后提交的目标；允许空刀。
- **遗言**：首夜死亡者有遗言，之后夜里死亡的没有遗言；被放逐的有遗言；被猎人带走的有遗言。
- **放逐投票**：平票时平票者 PK 发言后，由**非 PK 玩家**再投一轮；再平则无人出局。

**警长（MVP）**
1. **时机**：第一天天亮后、**公布夜间死讯之前**进行竞选。
2. **上警**：所有玩家同时决定是否上警（真人在手机上操作，NPC 由 agent 决定），限时 15 秒。
3. **警上发言**：候选人按随机起点顺序发言；发言期间和发言之后都可以**退水**，退水的人不能投票。
4. **投票**：警下玩家投票（可以弃票）。得票最高者当选；平票则 PK 发言后再投一轮；再平则**警徽流失**（本局没有警长）。所有人都上警时，也视为警徽流失。
5. **警长权力**：放逐投票时计 **1.5 票**；决定白天的发言顺序（从死者或警长的左侧/右侧开始）；**最后发言并归票**。
6. **警徽移交**：警长死亡时，可以把警徽**移交**给一名存活玩家，也可以**撕毁**警徽。
7. **自爆（P1）**：警上阶段有狼人自爆时，警徽顺延到第二天竞选（可配置为直接流失）。

---

## 2. 技术选型

### 2.1 总体栈

| 层 | 选型 | 理由 |
|----|------|------|
| 规则核心 | 纯 Rust，无 async、无 IO | 确定性，可复现，便于测试 |
| 法官 / 编排 | **tokio**（每个座位一个 actor，通过 mpsc 通道通信） | 天然隔离，方便处理超时和并发（夜间多名玩家同时操作） |
| 主机 UI | **Bevy 0.16 + bevy_egui** | 与仓库其他 X 项目保持一致；只负责渲染公开视图，通过 channel 与引擎通信 |
| 手机接入 | **axum**（HTTP + WebSocket）+ 静态网页（原生 JS 或 Preact，打包进二进制） | 手机免安装，扫码即用；使用 `rust-embed` 嵌入静态资源，单个可执行文件即可运行 |
| 二维码 | `qrcode` crate 生成图片，在 Bevy 大屏上显示 | — |
| 局域网发现 | `local-ip-address` 获取主机 IP，拼出访问地址 | — |

> Bevy 与 tokio 的集成方式：引擎单独跑在一个 tokio 运行时线程里；Bevy 主线程通过 `crossbeam-channel` 接收 `PublicView` 更新并发出 `HostInput`。Bevy 侧**不包含任何游戏逻辑**。

### 2.2 本地语音（sherpa-onnx，通过 `sherpa-rs` 调用）

| 能力 | 模型（初选，可替换） | 说明 |
|------|----------------------|------|
| STT | **SenseVoice-Small**（中英粤日韩，非流式，速度快） | 按键松开后整段识别，一段 30 秒的发言在 CPU 上约需 1 秒 |
| VAD | silero-vad | P1：自动断句；P0 只用于去掉首尾静音 |
| TTS | **中文多说话人 VITS / Matcha**（例如 `vits-zh-hf-*` / `matcha-icefall-zh-baker`），或 Kokoro 多语言模型 | 法官用固定音色；NPC 从说话人 ID 池中各分配一个不同的音色 |
| 音频 IO | `cpal`（采集）+ `rodio`（播放） | 播放队列串行，保证同一时间只有一个声音 |

- 模型文件放在 `models/`（gitignore），由 `scripts/fetch_models.ps1` 下载并校验 SHA256。
- 法官固定台词（约 40 句，含警长流程）首次运行时合成，缓存到 `cache/tts/`。
- 所有语音能力都通过 trait 抽象（`SttProvider` / `TtsProvider` / `Vad`），以后换成云端服务只需要新增实现。

### 2.3 LLM 选型（D3）

**默认 `claude-sonnet-5`，开启 prompt caching；提供 `economy` 预设（`claude-haiku-4-5`）。**

理由：
- 狼人杀的核心是**长上下文推理和保持一致的伪装**（例如悍跳预言家、编造查验），弱模型容易前后矛盾，或者不小心暴露身份，体验下降很明显。
- 一局只有几十轮发言，上下文增长有限（< 30k tokens）；配合 prompt caching（缓存系统提示、规则和历史前缀），成本可控。
- 按座位配置，可以灵活组合，比如神职和狼用 Sonnet、村民用 Haiku。

| 后端 | 用途 | 隔离方式 |
|------|------|----------|
| `AnthropicApi`（默认） | Messages API；结构化动作使用 tool use | 每个座位只持有自己的 messages 历史 |
| `ClaudeCodeCli` | 以 `claude -p --session-id <uuid>` 子进程运行 | 每个座位一个 session 和一个**独立的空临时目录**，**禁用全部工具** |
| `GeminiApi` | Google Gemini API（`generativelanguage.googleapis.com`，`GEMINI_API_KEY`）；结构化动作使用 `responseSchema` / function calling | 同 API 方式 |
| `AntigravityCli` | 以 Antigravity CLI 子进程运行（见 2.4）；复用 Antigravity 的登录和额度，不需要单独申请 API Key | 每个座位一条独立的 `conversation_id` 和一个独立的空临时目录；不加 `--add-dir`，不开 `--dangerously-skip-permissions` |
| `OpenAiCompatible` | DeepSeek / Qwen / 本地 Ollama | 同 API 方式 |
| `Scripted` | 随机或规则策略，用于测试和压测（`ww_core::sim` 已实现） | — |

### 2.4 Antigravity CLI（`agy`）接入要点（已在本机 agy 1.2.12 上实测）

```sh
# 首次调用：开启一条新对话
agy -p "<prompt>" --output-format json --model gemini-3.8-flash-medium \
    --disable-slash-commands --json-schema '<动作 schema>'
# 同一座位的后续调用：续接自己的对话
agy -p "<新观察 + 决策请求>" --conversation <该座位的 conversation_id> --output-format json ...
```

- **输出**：JSON，其中包含 `conversation_id`、`response`（文本）、`structured_output`（按 schema 解析后的结果）、`usage`、`denied_actions`。
- **动作**：只采用 `structured_output`，再交给规则层校验。实测中文本回答和结构化字段可能不一致（文本说查 4 号，结构化字段却是 0），所以 prompt 要要求模型只从结构化字段作答；不合法就重试一次，仍不合法则按默认动作处理。
- **发言**：发言请求不带 schema，直接使用 `response` 文本。
- **工具与隔离**：非交互模式下，需要授权的工具会被自动拒绝，正好满足"禁用工具"的要求；工作目录是每个座位独立的空目录。**不要加 `--sandbox`**：实测加上后 agent 会申请提权，被拒后返回空回答。
- **延迟和开销**：单次调用约 10–15 秒，并且每次都带着约 3 万 token 的 agent 系统提示。比 API 直连慢得多，适合"没有 API Key 也能玩"的场景，或者不赶时间的自对局评测。需要低延迟时优先用 `GeminiApi`。
- **可用模型**：用 `agy models` 列出，例如 `gemini-3.8-flash-*`、`gemini-3.1-pro-*`；它也能调用 Claude 模型。

---

## 3. 总体架构

```
                 ┌─────────── 主机 (一台电脑) ───────────────────────────────────────┐
  音箱 ◀─ TTS ───┤                                                                 │
  麦克风 ─ STT ─▶│  ww-voice ◀──────────┐                                          │
                 │                      │                                          │
   大屏 ◀────────┤  ww-app (Bevy)  ◀── PublicView ──┐                              │
   (只显示公开信息)│      │ HostInput(PTT/开局/设置)   │                              │
                 │      ▼                           │                              │
                 │  ┌──────────────────── ww-engine (tokio) ──────────────────┐    │
                 │  │ PhaseDriver ─Action─▶ ww-core(纯规则) ─Event─▶ VisibilityRouter │
                 │  │     ▲ Decision                               │ Observation   │    │
                 │  └─────┼──────────────────────────────────────────┼─────────────┘    │
                 │        │         per-seat mpsc (每座位一条专属通道)  │                  │
                 │   ┌────┴─────┬───────────┬───────────┬───────────┴┐                 │
                 │   │ Seat1    │ Seat2     │ Seat3     │ ... SeatN  │                 │
                 │   │ Remote   │ Remote    │ NpcAgent  │ NpcAgent   │   ww-record     │
                 │   │ HumanSeat│ HumanSeat │ (Claude)  │ (Claude)   │   (事件/发言/审计) │
                 │   └────┬─────┴────┬──────┴───────────┴────────────┘                 │
                 │        │ ww-net (axum WS, 每人独立 token)                             │
                 └────────┼──────────┼─────────────────────────────────────────────────┘
                     WiFi │          │ WiFi
                    ┌─────▼──┐  ┌────▼───┐
                    │ 手机 A  │  │ 手机 B  │  私密身份 / 夜间技能 / 投票 / 上警退水
                    └────────┘  └────────┘
```

### 3.1 Crate 划分（独立的 Cargo workspace）

```
X-005_Werewolf/
├── Cargo.toml                 # 独立 workspace，不接入仓库根目录
├── SPEC.md / README.md
├── config/
│   ├── default.toml           # 人数、规则开关、计时、端口
│   ├── boards.toml            # 各人数的预设板子
│   └── npcs.toml              # NPC 人设 / 音色 ID / 后端 / 模型
├── crates/
│   ├── ww-core/               # 角色、规则、状态机（含警长）、Event/Action/Visibility（无 IO）
│   ├── ww-engine/             # PhaseDriver、VisibilityRouter、计时、法官台词表
│   ├── ww-agent/              # Seat trait、NpcAgent、prompt、LLM 后端、意图解析
│   ├── ww-net/                # axum 服务：入座、token、WS 协议、RemoteHumanSeat
│   ├── ww-voice/              # STT / TTS / VAD / 音频 IO（sherpa-rs、cpal、rodio）
│   ├── ww-record/             # 事件存储、发言记录、审计日志、回放
│   ├── ww-app/                # Bevy 主机程序（大屏 + 二维码 + PTT）
│   └── ww-cli/                # 无头模式：纯 NPC 自对局 / 文本调试
├── web/                       # 手机端静态网页（通过 rust-embed 嵌入）
├── scripts/fetch_models.ps1
├── models/  cache/  records/  # gitignore
└── .gitignore
```

### 3.2 核心数据模型（ww-core）

```rust
pub enum Role { Werewolf, Villager, Seer, Witch, Hunter /* , Idiot, Guard */ }
pub enum Phase { Lobby, Night(u32), Day(u32), GameOver(Faction) }
pub enum NightStep { WolfKill, Witch, Seer }
pub enum DayStep {
    SheriffSignup, SheriffSpeech, SheriffVote, SheriffPk,          // 仅第 1 天（或顺延）
    Announce, LastWords, HunterShot, BadgeTransfer,
    Speech, ExileVote, ExilePk, Exile,
}

pub struct SeatId(pub u8);

/// 事件是唯一事实来源；每条事件都带可见性
pub struct Event { pub seq: u64, pub phase: Phase, pub vis: Visibility, pub kind: EventKind }
pub enum Visibility { Public, Team(Faction), Private(SeatId), GodOnly }

pub enum EventKind {
    RoleAssigned { seat: SeatId, role: Role },                 // Private(seat)
    WolfTeammates { wolves: Vec<SeatId> },                      // Team(Wolf)
    WolfChat { from: SeatId, text: String },                    // Team(Wolf)
    WolfTarget { target: Option<SeatId> },                      // Team(Wolf)
    WitchInfo { victim: Option<SeatId> },                       // Private(witch)
    SeerResult { target: SeatId, is_wolf: bool },               // Private(seer)
    NightResolved { deaths: Vec<(SeatId, DeathCause)> },        // GodOnly
    DeathsAnnounced { seats: Vec<SeatId> },                     // Public（不公开死因）
    SheriffCandidates { seats: Vec<SeatId> },                   // Public
    SheriffWithdraw { seat: SeatId },                           // Public
    SheriffElected { seat: Option<SeatId> },                    // Public
    BadgeTransferred { to: Option<SeatId> },                    // Public（None = 撕警徽）
    SpeakOrderChosen { start: SeatId, dir: Direction },         // Public
    Speech { seat: SeatId, text: String, audio: Option<AudioRef> }, // Public
    VoteCast { voter: SeatId, target: Option<SeatId>, weight: f32 }, // Public（票型）
    HunterShot { target: SeatId },                              // Public
    Exiled { seat: SeatId },                                    // Public
    GameOver { winner: Faction, roles: Vec<(SeatId, Role)> },   // Public
}

/// 纯函数：旧状态 + 动作 → 新状态 + 事件；非法动作返回 Err
pub fn apply(state: &GameState, action: Action) -> Result<(GameState, Vec<Event>), RuleError>;
```

### 3.3 座位接口（ww-agent / ww-net）

```rust
#[async_trait]
pub trait Seat: Send {
    async fn observe(&mut self, obs: Observation);                   // 只推送已过滤的信息
    async fn decide(&mut self, req: DecisionRequest, deadline: Instant) -> Decision;
}
pub enum DecisionRequest {
    Speak { kind: SpeechKind },                       // 普通发言 / 警上发言 / 遗言 / PK / 归票
    SheriffSignup, SheriffWithdraw,
    Vote { candidates: Vec<SeatId>, kind: VoteKind }, // 警长票 / 放逐票
    WolfKill { .. }, WitchAct { victim: Option<SeatId>, has_save: bool, has_poison: bool },
    SeerCheck { .. }, HunterShoot { .. }, BadgeTransfer { .. }, ChooseSpeakOrder { .. },
}
```

| 实现 | 说明 |
|------|------|
| `NpcAgent` | 自己维护 `Memory`（观察记录 + 私人笔记/怀疑度表）和 `Backend`；发言用流式文本，动作用 tool use 做结构化输出 → 规则层校验 → 不合法就重试一次 → 仍不合法则按默认动作处理 |
| `RemoteHumanSeat` | `observe` 推送到该玩家手机的 WS；`decide` 中的非发言请求发到手机，由玩家点选；`Speak` 请求交给主机麦克风（PTT）或手机麦克风（P1） |

---

## 4. 信息隔离设计（核心约束）

**原则：只有引擎持有上帝视角。任何参与者——NPC agent、手机、主机大屏——都只能拿到引擎明确推送给它的信息。**

### 4.1 NPC agent 之间
1. 每个座位是一个独立的 tokio 任务，拥有自己的 `Memory` 和 `Backend` 实例，**不存在共享的对话历史，也没有全局 prompt**。
2. 所有事件必须经过 `Game::observe(event, seat) -> Option<Observation>` 才能发出，这是**唯一的出口**。`Observation` 的字段是私有的，构造函数只在 crate 内部可见，因此外部代码无法绕过过滤。（实现说明：过滤函数是纯函数，放在 ww-core 里，这样可以直接被随机对局测试覆盖；ww-engine 的 VisibilityRouter 只负责按座位分发。）
3. 狼队夜聊用 `Team(Wolf)` 可见性分发给每只狼，每只狼仍然各自维护自己的上下文。
4. `ClaudeCodeCli` 后端：每个座位使用独立的 session 和独立的空临时目录，不开放任何工具，防止 agent 读到 `records/` 目录。
5. 真人语音的 LLM 意图解析（如果需要）只传入转写文本和候选座位号，**不传任何游戏状态**。

### 4.2 手机之间
1. 入座时，服务端为该座位签发一个随机 token；WS 连接必须携带 token，服务端只把该座位的 `Observation` 推送到这条连接。
2. 同一个座位同一时间只允许一个连接；断线后用 token 重连，服务端会补发该座位完整的观察历史。
3. 手机网页**不在本地保存任何状态**（刷新后由服务端补发），避免手机被别人拿到后翻看历史。
4. 夜间所有真人的手机界面**保持一致的节奏**：不管身份是什么，每一步都显示"请等待 / 请操作"界面，并且有轻微震动，防止邻座通过屏幕亮灭或震动推断身份。

### 4.3 主机大屏和音箱
1. 大屏只渲染 `PublicView`，由 `Visibility::Public` 的事件派生；Bevy 侧拿不到 `GameState`。
2. 音箱只播报公开信息和流程台词；私密信息（例如查验结果）**永远不走音箱**。
3. 已死亡或不存在的角色仍然照常播报"XX 请睁眼"，并随机等待 8–15 秒，防止从停顿时长推断身份。

### 4.4 可审计
- `ww-record` 为每个座位写一份 `seat_<n>.obs.jsonl`，也就是"这个座位看到过的全部内容"。
- 测试断言：对任意一局，seat X 的观察日志中不存在对 X 不可见的事件（property test + 固定种子回放，覆盖 Scripted 对局 1000 局）。

---

## 5. 一局流程（引擎视角）

```
Lobby:
  主机大屏显示二维码 → 真人扫码入座（选座位号 + 昵称）→ 房主在主机上设置人数 → 空座自动分配 NPC
  → 开始游戏：洗牌 → 私发身份(Private，发到手机/agent) → 狼人互认(Team)

Night(n):
  TTS「天黑请闭眼」
  WolfKill : 「狼人请睁眼」→ 狼队夜聊（手机上用快捷短语/点选；NPC 用文字）→ 提交刀口 →「狼人请闭眼」
  Witch    : 「女巫请睁眼」→ 私发刀口（若有解药）→ 救/毒/跳过        （女巫已死也照常播报 + 随机等待）
  Seer     : 「预言家请睁眼」→ 查验 → 私发结果                        （同上）
  结算（GodOnly）

Day(n):
  「天亮了」
  if n == 1 (或警徽顺延):
     SheriffSignup(同时决定) → SheriffSpeech(随机起点, 可退水) → SheriffVote(警下投票)
     → 平票 PK → 再平警徽流失 → 公布警长
  公布死讯 → 胜负检查
  首夜死者遗言 / 猎人开枪（若可）/ 警长死亡则移交或撕警徽 → 胜负检查
  警长决定发言顺序（无警长则从死者下家开始，方向随机）
  依次发言（警长最后发言并归票）
  放逐投票（手机上同时投票，警长 1.5 票）→ 公布票型 → 平票 PK → 再投 → 放逐
  遗言 → 猎人开枪（若可）→ 警长死亡则移交警徽 → 胜负检查
  → Night(n+1)

GameOver: 公布全部身份 + 生成复盘
```

- 每个 `decide` 都有 deadline。NPC 超时就使用默认动作；真人超时会先在手机上震动提醒，再按弃权处理。
- 发言权是**串行**的；投票、上警、夜间技能等操作是**并发**的，全部提交或截止后统一揭晓。

---

## 6. 语音与手机交互细节

### 6.1 主机语音

| 场景 | 流程 |
|------|------|
| 法官播报 | 固定台词优先读缓存 → 动态文本（死讯、票型、警长结果）即时合成 → 串行播放队列 |
| NPC 发言 | LLM 流式输出 → 按标点切句 → 逐句本地 TTS → 播放；大屏同步显示字幕和发言人高亮 |
| 真人发言 | 轮到某名真人时：大屏高亮其座位，手机提示"轮到你发言" → 按住主机 PTT 键（空格键或外接按钮）说话，也可以在手机上点"开始/结束"按钮远程控制主机录音 → SenseVoice 识别 → 生成 `Speech` 事件，同时保存 wav |
| 结束发言 | 松开 PTT 或手机上点"过" → 超过时限（默认 90 秒）自动结束 |

### 6.2 手机网页（web/）

| 页面状态 | 内容 |
|----------|------|
| 入座 | 选座位号、填写昵称 → 获得 token（存在 URL 参数中，可以直接重连） |
| 常驻 | 我的身份卡（按住才显示，松开即隐藏）、存活玩家列表、当前阶段、倒计时 |
| 操作 | 根据 `DecisionRequest` 渲染：座位点选 + 确认 / 救·毒·跳过 / 上警·不上 / 退水 / 移交警徽 / 选择发言方向 |
| 狼队夜聊 | 快捷短语（"刀 X 号"、"我悍跳"、"我藏着"）+ 目标点选，队友的选择实时同步 |
| 私密结果 | 查验结果、女巫看到的刀口：只显示在屏幕上，配合震动提示 |

> 真人的夜间操作用手机点选完成；需求中"不用打字"的约束在此满足（全程没有文本输入框）。白天发言用语音。

### 6.3 手机麦克风（P1）
浏览器在局域网 `http://` 地址下不允许调用麦克风（`getUserMedia` 要求安全上下文）。所以 P0 使用主机麦克风；P1 时由主机生成自签名证书，改用 `https://` 访问，手机上需要手动信任一次证书，之后就可以在手机上按住说话，音频上传给主机识别。

---

## 7. 记录与回放（ww-record）

```
records/<game_id>/
├── meta.json              # 板子、人数、种子、座位→身份、真人/NPC 配置、模型、版本
├── events.jsonl           # 全部事件（上帝视角），追加写入
├── transcript.md          # 可读的发言记录：[D2][警长·3号·小明] ……
├── seat_<n>.obs.jsonl     # 每个座位收到的 Observation（隔离审计）
├── seat_<n>.llm.jsonl     # NPC 的原始 LLM 请求/响应（调试用，可关闭）
└── audio/                 # 真人录音、TTS 输出（可配置是否保存）
```

- **回放**：用 `events.jsonl` 重建状态机，Bevy 上可以切换"上帝视角 / 某个座位视角"逐步播放。
- **复盘**：赛后可以（可选）调用 LLM 读取 `events.jsonl` 生成对局点评，这是离线任务，不影响对局中的隔离。

---

## 8. Git 隔离方案

另一个 agent 正在主工作区 `FutureGadgetLabtory/` 开发（当前分支 `dev_X-003_BattleCity`，还有未提交的改动），在同一个工作区里 `checkout` 会切走它正在用的分支。所以：

1. **git worktree**（已创建）：`FutureGadgetLabtory.wt\X-005`，分支 `dev_X-005_Werewolf`，基于 `main`（`ed1877a`）。两个工作区各自有独立的 HEAD、index 和工作目录。
2. **路径约束**：所有改动只能出现在 `X-005_Werewolf/` 下；提交时只用 `git add X-005_Werewolf/`，**禁止在仓库根目录 `git add -A`**。
3. **不改根目录文件**：根目录 README 里的项目清单留到合并 PR 时再添加。
4. **构建隔离**：独立的 Cargo workspace，`target/` 放在项目目录内。
5. **合并**：通过 PR 合入 `main`；worktree 由用户确认后再移除。

---

## 9. 里程碑

| 里程碑 | 内容 | 验收标准 |
|--------|------|----------|
| **M0 规则核心** ✅ | ww-core：基础 5 个角色 + 警长全流程 + 胜负判定 + 板子预设 | ✅ 17 个规则场景；6–12 人 × 1000 局随机对局全部正常结束，双方都有胜局；每局都做信息隔离审计；同一种子可复现 |
| **M1 NPC 自对局** | ww-agent（Anthropic / Gemini / Antigravity CLI 后端）+ ww-engine + ww-cli 文本模式 + ww-record | 9 个 NPC 完整打完一局（含警长），可以 Claude 与 Gemini 混桌；发言记录可读；审计日志通过 |
| **M2 本地语音** | ww-voice：sherpa-onnx 的 TTS/STT、法官台词缓存、NPC 多音色、PTT | 无头模式下能"听"完一局；主机麦克风的发言能正确转写 |
| **M3 主机 + 手机** | ww-net + web/ + ww-app（Bevy 大屏、二维码、公开视图） | 2 名真人用手机 + 7 个 NPC 完整对局，全程不需要打字；断线重连后信息完整 |
| **M4 完善** | 自爆、VAD、回放 UI、复盘、手机麦克风（HTTPS） | — |

---

## 10. 剩余待定（不阻塞 M0/M1）

1. NPC 人设和音色池的具体内容（M2 时从 TTS 模型的说话人列表中挑选试听）。
2. 狼队夜聊中真人的交互方式：快捷短语是否足够？是否需要支持真人狼语音夜聊（这需要手机麦克风，也就是 P1 的 HTTPS）。
3. 主机 PTT 的物理形态：键盘空格键、手机远程按钮，还是外接蓝牙翻页笔。
