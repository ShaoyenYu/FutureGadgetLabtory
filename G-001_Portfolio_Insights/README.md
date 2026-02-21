# G-001 Portfolio Insights

未来ガジェット研究所的第一个实验作品。G-001 Portfolio Insights 是一款用于基金组合分析与展示的应用，聚合净值数据，生成可交互的资产趋势图与关键指标，并提供快捷的组合录入与可视化管理能力。

## 主要功能
- 组合录入与管理：支持基金代码与份额的录入、增删、排序与清空
- 指标计算：Return Ratio、Sharpe Ratio、Max Drawdown 等关键指标展示
- 趋势图可视化：资产趋势曲线、回撤区间标注与回撤天数提示、末尾点数值标注
- 表格交互：列宽拖拽调整、三态排序、字段格式化显示
- 数据导入导出：支持 CSV 导入与导出
- 主题与隐私：深浅色主题切换；一键隐藏 Total Amount、Position、Shares

## 技术栈
- 前端：Vue 3 + Vite + ECharts
- 后端：Python（API 服务）

## 快速启动
1. 启动后端
   - 进入 backend 目录并安装依赖
   - 运行 main.py 启动 API
2. 启动前端
   - 进入 frontend-vue 目录安装依赖
   - 运行 Vite 开发服务器
