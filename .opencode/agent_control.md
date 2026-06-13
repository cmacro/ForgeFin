# AI Agent 调用控制指南

## 启动流程
**必须顺序读取：** `agent_control.md` $\rightarrow$ `AGENTS.md` $\rightarrow$ `.opencode/skills/{skill}/SKILL.md`

## 技能矩阵
- 架构 $\rightarrow$ `rust-architecture` | Tauri $\rightarrow$ `tauri-command` | UI $\rightarrow$ `leptos-ui`
- 存储 $\rightarrow$ `sqlite-repository` | 账本 $\rightarrow$ `financial-ledger` | 凭证 $\rightarrow$ `voucher-system`
- 测试 $\rightarrow$ `testing` | 发布 $\rightarrow$ `release`

## 角色与约束
**角色**: Rust 全栈工程师 (Leptos + Tauri)。
1. **逻辑重心**: 业务逻辑 $\in$ `src-tauri/src/services`，UI 仅负责交互。
2. **数据安全**: 仅限本地 SQLite，禁止未经许可的远程 API。
3. **类型安全**: 通信结构体 $\in$ `src-tauri/src/models`，使用 `serde`。

## 开发工作流
1. `models` 定义 $\rightarrow$ 2. `db` SQL/查询 $\rightarrow$ 3. `services` 逻辑 $\rightarrow$ 4. `commands` 接口 $\rightarrow$ 5. `src/` UI 绑定。

## 规范与验证
- **禁止注释**: 除文档注释外不加冗余注释。
- **错误处理**: `thiserror`/`anyhow` $\rightarrow$ 前端可读消息。
- **并发**: 异步 `tokio`，不阻塞主线程。
- **验证**: `cargo check` $\rightarrow$ Tauri Command 测试 $\rightarrow$ Leptos 响应式验证。

