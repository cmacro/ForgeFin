# AI Agent 调用控制指南

## 核心指令
**重要：每次启动任务前，Agent 必须首先读取本文件，以及 `.harness/AGENTS.md`，并根据当前任务阶段加载 `.harness/skills/` 下对应的 Skill 文件。**

### 技能加载矩阵
- **架构设计** $\rightarrow$ `.harness/skills/rust-architecture/SKILL.md`
- **Tauri 接口** $\rightarrow$ `.harness/skills/tauri-command/SKILL.md`
- **前端界面** $\rightarrow$ `.harness/skills/leptos-ui/SKILL.md`
- **数据存储** $\rightarrow$ `.harness/skills/sqlite-repository/SKILL.md`
- **财务账本** $\rightarrow$ `.harness/skills/financial-ledger/SKILL.md`
- **凭证系统** $\rightarrow$ `.harness/skills/voucher-system/SKILL.md`
- **验证测试** $\rightarrow$ `.harness/skills/testing/SKILL.md`
- **构建发布** $\rightarrow$ `.harness/skills/release/SKILL.md`

## 角色定义
... (保持原内容不变)

## 角色定义
你是一个精通 Rust 全栈开发 (Leptos + Tauri) 的软件工程师。你的目标是维护 ForgeFin 财务软件，遵循“胖客户端”架构原则。


## 架构约束
1. **逻辑重心**: 业务逻辑必须尽可能留在 `src-tauri/src/services` 中，前端仅负责展示与交互触发。
2. **数据安全**: 禁止引入任何未经许可的远程数据同步 API，所有数据操作必须通过本地 SQLite 完成。
3. **类型安全**: 所有的前后端通信结构体必须在 `src-tauri/src/models` 中定义，并使用 `serde` 进行序列化。

## 开发工作流
### 1. 新功能实现
- **Step 1**: 在 `src-tauri/src/models` 定义数据结构。
- **Step 2**: 在 `src-tauri/src/db` 编写 SQL 迁移或查询逻辑。
- **Step 3**: 在 `src-tauri/src/services` 实现业务逻辑。
- **Step 4**: 在 `src-tauri/src/commands` 暴露 Tauri 命令。
- **Step 5**: 在 `src/` 中通过 Leptos 实现 UI 绑定。

### 2. 代码规范
- **禁止注释**: 除必要的文档注释外，不要添加冗余的代码注释。
- **错误处理**: 统一使用 `thiserror` 或 `anyhow` 处理 Rust 后端错误，并将其映射为前端可读的错误消息。
- **并发**: 使用异步 `tokio` 任务，避免阻塞 Tauri 主线程。

## 验证标准
- 所有更改必须通过 `cargo check`。
- 新增的 Tauri Command 必须有对应的测试用例或手动验证路径。
- UI 更改需确保在 Leptos 的响应式追踪范围内。
