
 # FutureGadgetLab（未来道具实验室）
 ![未来道具研究所](./cover.png)
 
 受《命运石之门》中“未来道具研究所”启发，本仓库旨在与 AI 协作快速孵化和验证稀奇古怪的点子，探索人机共创的能力边界。这里的每个“G-系列”子项目都对应一件“未来道具”原型：小而快、敢试错、能落地。
 
 ## 愿景与原则
 - 快速实验：以最小可行原型（MVP）验证想法
 - 人机协作：充分利用 AI 协助编码、设计与迭代
 - 组合复用：优先选用成熟栈，模块化、可复用
 - 可运行即价值：每个道具都能在本地跑起来
 
 ## 当前 G 系列清单
 
 - G-001 Portfolio Insights（基金组合洞察）
   - 功能：基金组合录入、指标计算（如 Sharpe、回撤）、趋势图与表格交互、CSV 导入导出、隐私字段一键隐藏
   - 技术栈：前端 Vue 3 + Vite + ECharts；后端 Python FastAPI
   - 快速开始：Windows 可直接运行根目录下的 `start.bat`
   - 文档：[G-001_Portfolio_Insights/README.md](G-001_Portfolio_Insights/README.md)
 
 - G-002 Gate of Babylon（基金数据宝库）
   - 功能：抓取东方财富基金列表与每日净值（单位/累计），落库到 PostgreSQL
   - 技术栈：Python 脚本；PostgreSQL
   - 文档与使用方式：[G-002_GateOfBabylon/README.md](G-002_GateOfBabylon/README.md)
 
 - G-003 Nantong Long Card（南通长牌记忆训练）
   - 功能：以南通长牌牌面为素材的图像记忆/识别训练；后端提供随机牌面与选项、答案校验；前端进行交互展示
   - 技术栈：后端 Python FastAPI；前端 Next.js (App Router)
   - 快速开始：Windows 可运行根目录下的 `start_app.bat`
   - 目录：`G-003_Nantong_Long_Card/`
 
 > 说明：本仓库除 G 系列外，可能还包含用于支撑或工具化的其他序列项目（例如 I-、T- 前缀）。G 系列聚焦“实验型未来道具”，其余序列以各自目录为准。
 
 ## 如何开始
 - 选择一个感兴趣的 G 子项目，阅读其目录下的 README 并按步骤运行
 - 常用环境
   - Node.js（Vite/Next.js 项目建议 18+）
   - Python 3.11+（FastAPI 等）
 - 推荐做法
   - Python 使用 venv 隔离依赖
   - 前端使用包管理器（npm/pnpm/yarn）安装依赖并运行开发服务器
 
 ## 风格与协作约定
 - 以运行效果为先，保持目录清晰与脚手架简洁
 - 子项目内自带最小可用脚本（如 `start*.bat`）与 README
 - 不在仓库存放敏感信息；连接配置等通过本地未纳入版本控制的文件管理
 
 ## 许可证
 - 以各子项目目录下的声明为准（若未声明则默认保留所有权利）
 
 ---
 
 感谢加入未来道具实验室！愿每一次小步快跑，都能把灵感变成可运行的“道具”。 
 
