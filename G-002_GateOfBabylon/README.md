# G-002_GateOfBabylon

## 项目简介
G-002_GateOfBabylon 是一个基金数据抓取与入库项目，提供基金列表与基金净值（单位净值、累计净值）的抓取与存储能力，数据落库到 PostgreSQL。

## 目录结构
- eastmoney/
  - util/eastmoney.py：东方财富接口抓取逻辑
  - seed_fund_info.py：基金基础信息入库脚本
  - seed_fund_nav_daily.py：基金每日净值入库脚本
  - requirements.txt：依赖列表
- schema/
  - fund_info.sql：基金基础信息表结构
  - fund_nav_daily.sql：基金净值表结构

## 环境依赖
- Python 3.11+
- PostgreSQL

## 环境准备（venv）
```bash
python -m venv .venv
```

Windows PowerShell 激活：
```bash
.venv\Scripts\Activate.ps1
```

依赖安装：
```bash
pip install -r eastmoney/requirements.txt
```

## 配置说明
在 eastmoney/config.yaml 中配置数据库连接信息（文件已加入 gitignore）：
```yaml
host: your_host
port: 5432
user: your_user
password: your_password
database: babylon
timezone: UTC
```

## 使用方式

### 1. 入库基金基础信息
```bash
python eastmoney/seed_fund_info.py --config eastmoney/config.yaml
```

### 2. 入库基金净值（指定日期区间）
```bash
python eastmoney/seed_fund_nav_daily.py --config eastmoney/config.yaml --start-date 2026-02-09 --end-date 2026-02-13
```

### 3. 全量净值抓取（2020 至今）
```bash
python eastmoney/seed_fund_nav_daily.py --config eastmoney/config.yaml --start-date 2020-01-01 --end-date 2026-02-24
```

## 数据表说明
- fund_info：基金代码与名称
- fund_nav_daily：每日单位净值与累计净值（主键：fund_id + nav_date）
