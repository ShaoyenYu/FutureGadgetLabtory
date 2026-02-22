# 罗技 K380 FN 键切换工具

一个用于切换罗技 K380 键盘「F1–F12 标准模式 / 多媒体键模式」的桌面小工具。每次运行会在两种模式之间切换，并在终端中显示当前与目标状态。

- 语言与框架：Rust 2021
- 关键依赖：[`hidapi`](file:///d:/Projects/FutureGadgets/T-001_K380_fn_toggle/Cargo.toml)
- 入口文件：[main.rs](file:///d:/Projects/FutureGadgets/T-001_K380_fn_toggle/src/main.rs)

## 原理说明
- 通过 HID 协议向 K380 的厂商自定义接口发送特定序列，实现功能键行为的切换。
- 程序会枚举已连接的 HID 设备，仅在以下条件下尝试写入：
  - 设备 Vendor ID 为 `0x046d`（Logitech）且 Product ID 为 `0xb342`，或设备名包含 `K380`
  - 接口 `usage_page` 属于厂商自定义页（`>= 0xFF00`）或为 `0`（部分蓝牙驱动的表现）
- 发送成功后，会将目标状态写入系统临时目录中的 `k380_fn_state.txt`，以便下次运行进行反向切换。

## 环境要求
- 已安装 Rust 工具链（推荐使用 rustup）
- Windows 10/11（蓝牙或接收器连接的 K380 键盘）
  - 注：在少数环境下，向 HID 设备写入可能需要更高权限；若提示权限被拒，可尝试用管理员终端运行。
- 其他平台（Linux/macOS）理论上也可用，但未在本项目中验证。

## 构建与运行

```powershell
# 克隆或进入项目后
cargo build --release

# 运行可执行文件
.\target\release\k380_fn_toggle.exe
```

首次运行会显示当前检测到的模式，并尝试切换到另一模式；再次运行则会反向切换。

## 使用说明
- 运行程序，终端会输出：
  - 当前状态：`[ON] F1-F12 优先` 或 `[OFF] 多媒体键优先`
  - 正在切换到的目标状态说明
  - 成功或失败的提示
- 程序末尾会等待按下 Enter 键退出，方便查看输出。
- 状态记录文件位置：系统临时目录中的 `k380_fn_state.txt`（自动创建/更新）

## 常见问题
- 找不到键盘
  - 确认 K380 已连接并处于活跃输入通道（蓝牙或接收器）
  - 若同时连接多台罗技设备，程序只会对符合厂商自定义接口的通道进行写入
- 权限被拒
  - 在部分 Windows 配置下，向 HID 设备写入可能需要管理员权限
  - 尝试在管理员 PowerShell 中运行
- 切换无效
  - 少量驱动栈或固件版本可能屏蔽厂商接口；建议重新配对或更新驱动后再试

## 项目结构
- [Cargo.toml](file:///d:/Projects/FutureGadgets/T-001_K380_fn_toggle/Cargo.toml)：项目元数据与依赖
- [src/main.rs](file:///d:/Projects/FutureGadgets/T-001_K380_fn_toggle/src/main.rs)：程序入口与核心逻辑
  - HID 设备枚举与过滤
  - 向厂商接口写入序列以切换功能键模式
  - 临时文件记录当前状态，实现“下一次运行反向切换”

## 开发提示
- 如需在 Linux/macOS 上构建，参考 hidapi 在对应平台上的依赖配置；Linux 可能需要 `libhidapi`/`hidraw` 支持。
- 若要改为“显式设置模式”而非“交替切换”，可根据需要直接选择 `K380_SEQ_FKEYS_ON` 或 `K380_SEQ_FKEYS_OFF` 并移除状态文件逻辑。

## 许可
若未另行声明，默认为 MIT 或与仓库一致的开源许可。请在使用与分发时遵循相关条款。

