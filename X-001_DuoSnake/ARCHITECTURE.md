### 项目模块与跨平台架构文档

该项目是一个基于 Bevy 引擎（v0.14）开发的双人贪吃蛇游戏，支持 **桌面原生（Windows/macOS/Linux）、网页端（WebAssembly/WASM + WebGL/WebGPU）以及移动端（Android / iOS）** 跨平台组织与运行。视觉风格为「糖果软萌像素风」（Candy Kawaii），所有像素美术在启动时由代码程序化烘焙生成，仓库中不含任何二进制素材。

---

#### 目录与模块架构概览
```text
X-001_DuoSnake/
├── Cargo.toml                  # 项目元数据、依赖管理、多端特征与 Profile 优化
├── .cargo/
│   └── config.toml             # 跨目标 Rust 编译器标志（如 WASM getrandom_backend）
├── index.html                  # 根目录 WebAssembly 挂载页面（供 Trunk 零配置启动）
├── Trunk.toml                  # Trunk 打包器主配置
│
├── src/                        # 游戏核心业务逻辑（Core Engine & Systems）
│   ├── lib.rs                  # 核心库入口：create_app() 与 run()（用于多端共享与动态库绑定）
│   ├── main.rs                 # 桌面与 Web 二进制可执行文件启动入口
│   ├── components.rs           # ECS 组件（Components）、全局资源（Resources）与事件（Events）
│   ├── constants.rs            # 游戏常量与调色板（Candy Kawaii palette）
│   ├── pixel_art.rs            # 像素美术：字符画 + 调色板 → 运行时程序化烘焙为 Image 纹理
│   └── systems/                # ECS 业务系统分包
│       ├── mod.rs              # 模块导出文件，整合所有系统模块
│       ├── snake.rs            # 贪吃蛇逻辑（双人移动、进食、生长、碰撞死亡、HP扣减与复活）
│       ├── environment.rs      # 环境逻辑（苹果随机生成、炸弹陷阱几何形状生成与倒计时销毁）
│       ├── ui.rs               # 用户界面（顶部 HUD 玩家卡、心之容器、暂停/设置菜单、按钮交互）
│       └── render.rs           # 渲染布局（棋盘、双层外框、坐标换算、缩放、呼吸动画与安全窗口查询）
│
├── platforms/                  # 跨端独立配置与打包模版（Platform Templates & Manifests）
│   ├── web/                    # 🌐 WebAssembly
│   │   ├── index.html          # Web 容器、响应式样式、加载动画与全屏
│   │   ├── Trunk.toml          # Trunk 独立配置文件
│   │   └── README.md           # 网页端发布指南
│   ├── android/                # 🤖 Android
│   │   ├── AndroidManifest.xml # AndroidManifest 清单文件（NativeActivity 配置）
│   │   └── README.md           # Android NDK 编译与 cargo-apk 指南
│   ├── ios/                    # 🍎 iOS
│   │   ├── Info.plist          # iOS 属性配置文件
│   │   └── README.md           # iOS 编译与 Xcode / cargo-mobile2 指南
│   └── desktop/                # 💻 Desktop
│       └── README.md           # 桌面端原生打包指南
│
├── scripts/                    # 跨平台构建与调试脚本（Automation Scripts）
│   ├── build-web.ps1           # Web 端启动与构建脚本 (PowerShell)
│   ├── build-web.sh            # Web 端启动与构建脚本 (Bash)
│   ├── build-desktop.ps1       # 桌面端构建与运行脚本 (PowerShell)
│   ├── build-desktop.sh        # 桌面端构建与运行脚本 (Bash)
│   └── build-android.ps1       # Android 构建与调试脚本 (PowerShell)
│
├── dist/                       # Web 打包静态生成目录（包含 js 胶水代码与 wasm 二进制，已加 gitignore）
├── ARCHITECTURE.md             # 架构技术规范文档
└── README.md                   # 项目使用与快速上手说明
```

---

#### 跨平台设计规范

1. **库与可执行文件解耦（`lib.rs` + `main.rs`）**
   - 将 Bevy 应用构建逻辑封装于 [`src/lib.rs`](file:///C:/Users/Admin/Documents/PycharmProjects/FutureGadgetLabtory/X-001_DuoSnake/src/lib.rs) 的 `create_app()` 与 `run()` 中。
   - `Cargo.toml` 配置 `crate-type = ["rlib", "cdylib"]`，确保桌面与 Web 端直接调用二进制入口，而 Android (`cargo-apk` / `ndk-glue`) 与 iOS 能够顺利链接动态/静态库。

2. **Web (WASM) 适配规范**
   - **Canvas 与视口自适应**：
     - `canvas: Some("#bevy".into())`：绑定至 Web 页面中已有的 `<canvas id="bevy">` 画布。
     - `fit_canvas_to_parent: true`：使画布自动适应父容器尺寸，支持浏览器窗口 resize 与移动端/全屏切换。
     - `prevent_default_event_handling: true`：捕获方向键与 WASD 输入，防止按键导致浏览器页面滚动。
   - **随机数与 WebAssembly 特性**：
     - 在 `.cargo/config.toml` 中配置 `rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]`。
     - 在 `Cargo.toml` 的 `target.'cfg(target_arch = "wasm32")'.dependencies` 中启用 `getrandom` 与 `uuid` 的 `js` / `wasm_js` 特性。

3. **安全的 ECS 窗口查询 (`systems/render.rs`)**
   - 在 Web 端画布挂载或窗口失焦切换期间，改用 `windows.get_single()` 安全模式，避免因单窗口假设 panic，保证游戏生命周期的健壮性。

4. **自动化脚本与平台分离**
   - 所有的构建逻辑通过 `scripts/` 下的跨平台脚本统一分发，配置信息归集于 `platforms/` 对应子目录，具备清晰的可维护性与扩展性。
