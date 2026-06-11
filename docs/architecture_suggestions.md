# 项目架构建议 (源自 ChatGPT 共享内容)

## 1. 核心技术栈
- **语言**: Rust
- **前端 UI**: Leptos
- **客户端壳**: Tauri 2.x
- **本地数据库**: SQLite
- **本地 AI**: Qwen3-Coder / Qwen2.5-Coder (本地部署)
- **业务领域**: 财务系统 $\rightarrow$ 未来扩展 WhatsApp、邮件、Agent

## 2. 架构模式
**建议采用: DDD Lite + Clean Architecture**
- **目的**: 避免后期重构，确保系统在从 MVP 演进到中大型桌面财务系统时的稳定性。
- **适用场景**: 非常适合基于 Skill 的 AI 开发工作流（如 OpenCode, Claude Code 等）。

## 3. 推荐项目结构
```text
.
├── .harness/
│   ├── AGENTS.md               # Agent 行为定义与工作流
│   └── skills/                  # 领域专项技能集
│       ├── rust-architecture/    # 架构约束与分层规范
│       ├── tauri-command/       # 接口定义规范
│       ├── leptos-ui/           # UI 实现标准
│       ├── sqlite-repository/   # 数据持久化模式
│       ├── llm-provider/        # AI 推理与 Prompt 规范
│       ├── financial-ledger/     # 财务账本逻辑
│       ├── voucher-system/      # 凭证管理系统
│       ├── testing/              # 测试验证标准
│       └── release/              # 发布构建流程
```

## 4. 未来扩展路径
当需要引入外部集成（如 WhatsApp、邮件）时，仅需在以下两个维度进行水平扩展，无需调整现有核心架构：
- **新增领域 (Domain)**: `domain/whatsapp`, `domain/email`, `domain/agent`
- **新增技能 (Skill)**: `skills/whatsapp-gateway`, `skills/email-smtp`, `skills/agent-workflow`
