# DuoSnake 🐍🐍

DuoSnake is a competitive, two-player snake game built with the [Bevy](https://bevyengine.org/) engine (v0.14). It introduces survival, combat, and respawn mechanics to the classic snake formula with a sweet "Candy Kawaii" procedural pixel art style.

The codebase is organized with a **cross-platform library architecture** (`lib.rs` + `main.rs`), ready for **Web (WASM)**, **Desktop (Windows/macOS/Linux)**, and future **Mobile (Android/iOS)** deployment.

---

## 🎮 Game Features

- **Two-Player Local Co-op / Versus**: Play with a friend on the same screen (Player 1 on Arrow Keys, Player 2 on WASD).
- **HP & Respawn System**: Players start with configurable HP (default 5 hearts). Dying penalizes your length and triggers a 3-second respawn invulnerability timer instead of ending the game immediately.
- **Dynamic Arena**: Apples spawn dynamically to grow your snake; timed bomb traps spawn in varied geometric shapes.
- **Candy Kawaii Pixel Art**: 100% procedurally baked pixel textures at startup (no binary image files required).
- **Cross-Platform**:
  - 🌐 **Web (WASM)**: WebGL/WebGPU via Trunk and WebAssembly.
  - 💻 **Desktop**: Native Windows (`.exe`), macOS, and Linux binaries.
  - 📱 **Mobile Ready**: `src/lib.rs` (`cdylib` / `rlib`) + manifests structured for Android (`cargo-apk`) & iOS (`cargo-mobile2`).
- **Interactive UI & Settings**: In-game HUD, starting HP configuration, pause menu, and game restart.

---

## 🕹️ Controls

| Action | Player 1 (Mint 🟢) | Player 2 (Sky Blue 🔵) |
|---|---|---|
| **Move Up** | <kbd>▲</kbd> Arrow Up | <kbd>W</kbd> |
| **Move Left** | <kbd>◄</kbd> Arrow Left | <kbd>A</kbd> |
| **Move Down** | <kbd>▼</kbd> Arrow Down | <kbd>S</kbd> |
| **Move Right** | <kbd>►</kbd> Arrow Right | <kbd>D</kbd> |
| **Pause / Settings** | <kbd>ESC</kbd> | <kbd>ESC</kbd> |

---

## 📂 Project Architecture & Structure

```text
X-001_DuoSnake/
├── Cargo.toml                  # Crate config & multiplatform dependencies
├── .cargo/
│   └── config.toml             # Target-specific rustflags (e.g. WASM getrandom_backend)
├── index.html                  # Root Web entrypoint for instant Trunk execution
├── Trunk.toml                  # Trunk bundler configuration
│
├── src/                        # 核心游戏源码 (Core Game Engine)
│   ├── lib.rs                  # 库入口：create_app() 与 run()（供跨端与移动端调用）
│   ├── main.rs                 # 桌面与 Web 二进制可执行文件入口
│   ├── components.rs           # ECS 组件、资源、事件与状态机
│   ├── constants.rs            # 棋盘规格、UI 尺寸与 Candy Kawaii 调色板
│   ├── pixel_art.rs            # 字符像素画定义与运行时 Image 纹理烘焙
│   └── systems/                # ECS 业务系统分包
│       ├── mod.rs              # 系统模块整合导出
│       ├── snake.rs            # 移动、进食、生长、碰撞结算与复活
│       ├── environment.rs      # 苹果生成与炸弹陷阱几何倒计时
│       ├── ui.rs               # 顶部 HUD、心之容器、暂停与设置菜单
│       └── render.rs           # 安全窗口查询、棋盘布局与呼吸缩放
│
├── platforms/                  # 跨端发布配置与模版 (Platform Configurations)
│   ├── web/                    # 🌐 WebAssembly (WASM / WebGL)
│   │   ├── index.html          # Web 容器、响应式样式、加载动画与全屏
│   │   ├── Trunk.toml          # Trunk 独立配置文件
│   │   └── README.md           # 网页端发布指南
│   ├── android/                # 🤖 Android (NativeActivity / cargo-apk)
│   │   ├── AndroidManifest.xml # AndroidManifest 清单文件
│   │   └── README.md           # Android NDK 编译指南
│   ├── ios/                    # 🍎 iOS (Xcode / cargo-mobile2)
│   │   ├── Info.plist          # iOS 属性配置文件
│   │   └── README.md           # iOS 编译与测试指南
│   └── desktop/                # 💻 Desktop (Windows / macOS / Linux)
│       └── README.md           # 桌面端打包指南
│
├── scripts/                    # 跨端构建与启动脚本 (Automation Scripts)
│   ├── build-web.ps1           # Web 端启动与构建脚本 (PowerShell)
│   ├── build-web.sh            # Web 端启动与构建脚本 (Bash)
│   ├── build-desktop.ps1       # 桌面端构建与运行脚本 (PowerShell)
│   ├── build-desktop.sh        # 桌面端构建与运行脚本 (Bash)
│   └── build-android.ps1       # Android 构建与调试脚本 (PowerShell)
│
├── dist/                       # Web 打包生成目录 (Git ignored)
├── ARCHITECTURE.md             # 详细技术架构与模块规范文档
└── README.md                   # 项目总览文档
```

---

## 🚀 Quick Start & Build Guide

### 1. 🌐 Web (WASM / 浏览器端)

#### 安装依赖
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

#### 本地开发运行
```bash
# 直接使用 trunk
trunk serve

# 或者使用 scripts 脚本
.\scripts\build-web.ps1 -Serve
# Linux/macOS: ./scripts/build-web.sh serve
```
在浏览器中打开 **http://127.0.0.1:8080**。

#### 打包生产版本
```bash
trunk build --release
```
打包输出位于 `dist/` 目录，可直接部署到 GitHub Pages、Vercel 等静态托管平台。

---

### 2. 💻 Desktop (桌面原生端)

```bash
# 调试运行
cargo run

# 发布运行
cargo run --release

# 或者使用 scripts 脚本
.\scripts\build-desktop.ps1 -Release
# Linux/macOS: ./scripts/build-desktop.sh run
```

---

### 3. 🤖 Android (移动端预览)

1. 安装目标与工具：
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi
   cargo install cargo-apk
   ```
2. 构建与运行：
   ```bash
   .\scripts\build-android.ps1 -Run
   # 或 cargo apk run
   ```

---

### 4. 🍎 iOS (移动端预览)

1. 安装目标与工具：
   ```bash
   rustup target add aarch64-apple-ios aarch64-apple-ios-sim
   cargo install cargo-mobile2
   ```
2. 构建静态库：
   ```bash
   cargo build --target aarch64-apple-ios --release --lib
   ```
