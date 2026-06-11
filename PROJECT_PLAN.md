# ForgeFin 项目规划

## 1. 核心愿景
构建一个基于 Rust 的本地优先财务管理软件，采用“胖客户端 + 瘦服务”模式。所有敏感财务数据存储在本地，利用本地轻量级 LLM 提供智能分析与自动化记账。

## 2. 技术栈
- **前端 UI**: Leptos (CSR 模式) - 提供高性能、响应式的 Web-like 体验。
- **客户端壳**: Tauri - 提供原生系统访问、窗口管理与 Rust 后端集成。
- **本地数据库**: SQLite (via sqlx) - 确保数据的 ACID 特性与本地持久化。
- **本地 AI**: Candle (Hugging Face) - 集成 2B 规模模型（如 Qwen2-1.5B / Gemma-2B）。
- **语言**: Rust (全栈) - 保证内存安全与极致性能。

## 3. 目录结构规划
```text
ForgeFin/
├── .harness/                # AI Agent 控制与指令集
├── src/                     # Leptos 前端源代码
│   ├── components/          # 可复用 UI 组件
│   ├── pages/               # 业务页面 (账单, 报表, 设置)
│   ├── app.rs               # 应用主入口与路由
│   └── store/               # 前端状态管理
├── src-tauri/               # Tauri 后端源代码
│   ├── src/
│   │   ├── main.rs          # Tauri 入口
│   │   ├── commands/         # UI 调用接口 (Tauri Commands)
│   │   ├── db/              # SQLite 数据库迁移与访问层
│   │   ├── ai/              # Candle 模型加载与推理逻辑
│   │   ├── services/        # 核心业务逻辑 (财务计算、审计)
│   │   └── models/          # 领域模型定义
│   ├── migrations/          # SQL 迁移脚本
│   └── tauri.conf.json      # Tauri 配置文件
├── assets/                  # 静态资源 (图片, 字体)
└── Cargo.toml               # 全局依赖管理
```

## 4. 核心模块设计
### 4.1 数据层 (Local Storage)
- **模式**: 关系型数据库，支持多账本管理。
- **关键表**: `accounts` (账户), `transactions` (交易记录), `categories` (类目), `budgets` (预算)。

### 4.2 AI 推理层 (Edge AI)
- **任务**: 自然语言记账 (e.g., "昨天买了杯咖啡 25 元") $\rightarrow$ 结构化 JSON $\rightarrow$ 存入 DB。
- **部署**: 模型权重文件存放在用户本地数据目录 (`app_data_dir`)。

### 4.3 通信机制
- **UI $\rightarrow$ Rust**: 通过 `invoke` 调用 Tauri 命令。
- **Rust $\rightarrow$ UI**: 通过 `emit` 发送异步事件通知。
