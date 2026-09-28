# X-005 Werewolf（语音狼人杀）

一台电脑做主机、手机扫码入座、本地语音主持、LLM NPC 补位的基础版狼人杀（预女猎 + 警长）。
完整需求与架构见 [SPEC.md](SPEC.md)。

## 进度

| 里程碑 | 状态 |
|--------|------|
| M0 规则核心 `ww-core` | ✅ 完成 |
| M1 NPC 自对局（Claude / Gemini / Antigravity CLI agent + 文本 CLI + 记录） | ⏳ |
| M2 本地语音（sherpa-onnx） | — |
| M3 主机 Bevy 大屏 + 手机网页 | — |
| M4 完善 | — |

## ww-core

纯规则引擎，不做 IO、不含 async，结果可以用固定种子复现。

- 角色：狼人 / 村民 / 预言家 / 女巫 / 猎人；6–12 人预设板子（默认 9 人 3 狼 + 预女猎 + 3 民）
- 流程：狼刀 → 女巫 → 预言家 → 警长竞选（上警 / 退水 / PK / 警徽流失）→ 公布死讯 → 遗言 / 开枪 / 移交警徽 → 警长定发言顺序 → 发言 → 放逐投票（警长 1.5 票）/ PK
- 驱动：`Game::pending()` 查看每个座位待做的决策 → `Game::submit(seat, decision)` 或 `Game::timeout()` → 返回新事件
- 隔离：每个事件都带可见性标签；座位只能通过 `Game::observe` 拿到 `Observation`（无法直接构造）

```sh
cargo test --release   # 17 个规则场景 + 6–12 人各 1000 局随机对局（含信息隔离审计）
```
