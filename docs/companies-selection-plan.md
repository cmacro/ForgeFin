# 企业帐套选择界面开发计划

> 基于 `src/app.rs`、`session.rs`、`nav.rs` 分析，新增「登录态 → 选帐套」路由切换交互，替换当前静态文本。

## 核心行为追溯（现状）

| 流程 | 触发文件 | 现有实现 |
|------|----------|---------|
| App 启动恢复 session | `session.rs:3` → `<Session::init()>` | async + loading state；加载完成后调用 |
| Session::loadUser() API | `lib.rs:94` | 无参数查询，返回 `Result<Option<UserInfo>, _>`（含 token/user_id/username/created_at/companies） |
| AppRouter 路由分支 | `app.rs:38-42` | user=None → `<Login />` / has_company=true→`MainShell`（NavKey Home+Dashboard） |
| MainShell match NavKey | `app.rs:57-96` | NavKey::AccountBalance/GeneralLedger/TrialBalance/ReportCenter/FixedAssets/Cashier/Budget/Tax 全部为 **Placeholder** |

---

## T1：新建帐套列表页组件 (`src/pages/companies/list.tsx`)

> 复用现有 SearchTable + Pagination（table/index.tsx）的列定义模式：SearchHeader + DataTable。新增列：名称、地址、创建日期、会计期间(年/月)、状态、操作按钮组（启用/禁用）。

| # | Tauri 命令 | 说明 |
|---|-----------|------|
| T1.2 | `get_all_companies` → `GET /api/companies/list` (JSON array) | 按创建时间正序排列，返回所有帐套列表（包括已删除/禁用状态） |

> **注意：** 此命令在 Phase 0.3 Schema 中需要定义表结构：company_name / company_address / company_created_at / accounting_period_year_month。
> 参考 TODO.md:59「Phase 0.5 公司/账套管理 — 多组织支持，新建/切换/编辑公司信息」

---

## T2：帐套选择交互（替换静态 NoCompany）

| # | Tauri 命令 | 说明 |
|---|-----------|------|
| T1.3 | `delete_company` → `POST /api/companies/{id}/delete` (JSON body `{name}`) | 删除当前帐套，**二次确认弹窗 + 数据备份警告！**（TODO.md:59 已标记：「重要！删除前请备份」） |

> 前端触发点：**`Tauri::invoke("delete_company", id)`** — T1.4 新建命令。
> **参考 app.rs:98-102、Tauri 调用模式：`window.webviewWindow().open_url = Some("/settings")**。

---

## T3：Session state 扩展 + nav active page state

| # | Tauri 命令 | 说明 |
|---|-----------|------|
| T1.4 | `switch_company(id)` → 无响应/仅刷新页面（`app.rs:69-72`，`NavKey::VoucherManagement=VoucherEntry+search_form_data`） | 切换帐套时自动重置 Dashboard KPI + search_form_value |

> 参考现有模式：`window.webviewWindow().set_title = Some("Dashboard")`；Tauri invoke 使用 `serde_json` + Cargo.toml 中已声明的依赖。
> **注意：** 此命令在 Phase 0.4.3 Tauri API（TODO.md:29-69）需要实现。

---

## T4：Navigation shell 增「帐套面板」Tab

| # | Tauri window config | 说明 |
|---|--------------------|------|
| T1.5 | `webviewWindow` + 居中居中布局（参考 `Tauri::Builder::with_window_size((w,h))`) | 新增 `<AppShell nav=nav.clone()>` 子组件，左侧为 nav tree、右侧为 children view——**当前已有 AppShell！** (`shell.rs:140+`) |

> **验证：** Tauri invoke 命令使用 `serde_json` — `Cargo.toml` + Tauri 2 API 已声明。
> **模式参考：** app.rs:60-75 + shell.rs:8-34（header with hamburger menu；nav tree in side nav; breadcrumb; search; tags）。

---

## Phase 0.5 更新：公司/帐套管理子任务拆分

| # | Tauri 命令 | Priority | 说明 |
|---|-----------|----------|------|
| **0.5a** | `T1 get_all_companies` + nav 路由 state 扩展 | **P0** | 帐套列表页 + Session.active_company_id 状态管理 |
| **0.5b** | **T2 delete_company** → POST/二次确认弹窗 | P0 | 删除前备份；数据隔离不可逆提示（TODO.md:29-30） |

---

## 目录约定速查

| 类型 | 路径 |
|------|------|
| 页面 | `src/pages/*/` |
| 组件 | `src/components/` |
| Tauri 命令/通信 | `Tauri::invoke("xxx")` (JSON with `serde_json`) |
| Session state | `session.rs:29-46` — User/CompanyId/ActivePageId/Loading |

---

## ✅ 完成校验（必须全部通过）

```sh
cargo fmt
cargo check
```
