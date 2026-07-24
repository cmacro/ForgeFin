# Phase 1 验收文档（示例）

## 1. 目标概述
本阶段实现 **原始凭证导入 → 自动/人工对账 → 审核 → 凭证生成** 的完整闭环。所有业务均在 **公司/账套隔离** 的 SQLite 文件中运行，使用 `rusqlite` 原生 SQL，前端 Leptos 0.8 + Tauri 2 + Tailwind v4 实现 UI。

---

## 2. 关键功能列表
| 功能 | 前端页面 | Tauri 命令 | 关键表 |
|------|----------|-----------|--------|
| 原始文件导入 | `src/pages/raw_data.rs` | `importRawFile` | `raw_bank_statements`, `raw_order_statements`, `attachments` |
| 自动对账 | `src/pages/reconciliation.rs` | `reconcile` | `transaction_summaries` |
| 差异人工审核 | `src/pages/reconciliation.rs`（审核弹框） | `reviewSummary` | `transaction_summaries`, `audit_logs` |
| 凭证生成 | 内部调用（`generateVoucher`） | `generateVoucher` | `vouchers`, `voucher_entries` |
| 审计日志查询 | `src/pages/audit_log.rs` | `listAuditLogs` | `audit_logs` |

---

## 3. 验收标准（Definition of Done）
| 编号 | 标准 | 检查方式 |
|------|------|-----------|
| **1** | `cargo fmt && cargo check` 均通过 | CI 本地运行 `cargo fmt && cargo check` |
| **2** | 数据库迁移成功，所有新表存在 | 启动应用后 `SELECT name FROM sqlite_master WHERE type='table'` 包含 `raw_bank_statements`、`raw_order_statements`、`transaction_summaries`、`attachments`、`audit_logs` |
| **3** | 导入示例文件后 `raw_bank_statements` 行数 > 0，`attachments` 记录产生 | 前端上传 `bank_sample.tsv` → 数据库查询行数 | 
| **4** | 同日银行与订单金额相等时系统自动匹配，无差异记录 | `SELECT * FROM transaction_summaries WHERE review_status='pending' AND date='2026-07-14'` 返回空 |
| **5** | 人工产生差异后可在 UI 中 “通过” → 生成凭证，凭证分录关联原始记录 | 点击审核通过 → 查询 `vouchers`、`voucher_entries.source_raw_id` 是否指向对应 `raw_*` 表 |
| **6** | 每一次导入、审核、凭证生成都有完整审计日志 | `SELECT * FROM audit_logs WHERE entity_type='raw_bank'` 等返回对应记录 |
| **7** | 前端页面在 `localhost` 可正常访问且无 JS 错误 | 浏览器控制台检查 | 
| **8** | UI 符合 ForgeFin 设计规范（颜色 token、信息密度、响应式） | 目视检查 + Tailwind 编译通过 |
| **9** | 文档完整 – 在 `docs/plan/phase1_acceptance.md` 记录表结构、接口、业务流程、验收标准 | 代码库中存在该文件 |

---

## 4. 测试场景（脱敏示例数据）
### 4.1 导入原始银行流水
- **文件**: `bank_sample.tsv`（10 行）
- **期望**: `raw_bank_statements` 插入 10 条记录，`attachments` 中新增 1 条，`import_batch_id = "BANK_20260715_001"`

### 4.2 导入原始订单流水
- **文件**: `order_sample.tsv`（8 行）
- **期望**: `raw_order_statements` 插入 8 条记录，`import_batch_id = "ORDER_20260715_001"`

### 4.3 自动对账（日期 2026‑07‑14）
- **银行合计**: 15 970.00（`in_amount` 总和）
- **订单实收合计**: 15 970.00（`net_amount` 总和）
- **期望**: `transaction_summaries` 不产生差异记录，`matched_bank_tx_id` 填充对应银行记录 ID。

### 4.4 差异人工审核
- **情形**: 当日订单实收 15 599.90，银行入账 15 639.00 → 差额 39.10
- **系统行为**: 创建 `transaction_summaries`（`review_status='pending'`）
- **审核通过后**: 系统调用 `generateVoucher`，生成 1 条 `vouchers` + 2 条 `voucher_entries`（借/贷），`voucher_entries.source_raw_id` 分别指向对应 `raw_order_statements` 与 `raw_bank_statements`，`audit_logs` 记录 `action='generate_voucher'`。

### 4.5 审计日志验证
- **检查点**: 每一次 `importRawFile`、`reviewSummary`、`generateVoucher` 必产生 `audit_logs` 条目，字段 `old_values`/`new_values` 为 JSON（脱敏示例使用 `"..."` 占位），`operator_id` 为当前登录用户 ID（示例 `1`）。

---

## 5. 交付物清单
1. **数据库迁移脚本** `src/migrations/V001__raw_tables.sql`（已提交）
2. **后端 Tauri 命令** `src-tauri/src/commands/*`（5 项）
3. **前端页面 & 组件**（`raw_data.rs`, `reconciliation.rs`, `audit_log.rs` + 4 组件）
4. **验收文档** 本文件 `docs/plan/phase1_acceptance.md`
5. **测试用例** `docs/测试方案.md` 中已加入本阶段 5 条用例（指向本文件）

---
## 7. 原始数据记录完整界面实现计划

> 本章节用于指导前端/后端开发人员完成 **原始凭证导入、对账、审核、审计日志** 的完整页面实现，页面结构、组件拆分与交互逻辑均遵循现有 `voucher.rs` 页面模式。

### 7.1 页面与组件清单

| 层级 | 文件路径 | 职责 |
|------|----------|------|
| **页面** | `src/pages/raw_data.rs` | 原始数据导入 + 原始记录库（双 Tab） |
| **页面** | `src/pages/reconciliation.rs` | 按日期对账 + 差异列表 + 审核入口 |
| **页面** | `src/pages/audit_log.rs` | 审计日志全局查询与展示 |
| **组件** | `src/components/source/import_uploader.rs` | 文件拖拽/选择上传、进度、格式校验 |
| **组件** | `src/components/source/raw_record_table.rs` | 原始记录表格，含分页与行选择 |
| **组件** | `src/components/source/record_detail.rs` | 原始记录详情面板，展示 `raw_data` JSON、附件、来源 |
| **组件** | `src/components/reconciliation/diff_list.rs` | 差异列表，每行显示差额、来源、操作按钮 |
| **组件** | `src/components/reconciliation/diff_review_modal.rs` | 差异审核弹框，支持“通过/驳回”与备注 |
| **组件** | `src/components/audit/log_table.rs` | 审计日志表格，支持按实体类型/时间过滤 |
| **组件** | `src/components/common/decimal_input.rs` | 金额输入组件（`rust_decimal` + 两位小数格式化） |

### 7.2 导航与路由

在 `src/nav.rs` 新增 `NavKey` 与导航树：

```rust
pub enum NavKey {
    // ... 现有 ...
    RawData,
    Reconciliation,
    AuditLog,
}

// 新增一级菜单或挂在“出纳管理”下（推荐独立一级菜单“原始凭证”）
NavItem {
    key: NavKey::RawData,
    label: "原始数据",
    icon: "upload-cloud",
    route: "/raw-data",
    children: Some(vec![
        NavItem { key: NavKey::RawData, label: "导入中心", icon: "upload", route: "/raw-data/import", children: None },
        NavItem { key: NavKey::Reconciliation, label: "对账中心", icon: "check-circle", route: "/raw-data/reconciliation", children: None },
        NavItem { key: NavKey::AuditLog, label: "审计日志", icon: "scroll-text", route: "/raw-data/audit-log", children: None },
    ]),
}
```

在 `src/app.rs` `MainShell` 的 `match key { ... }` 中增加分支：

```rust
NavKey::RawData => view! { <RawData /> }.into_any(),
NavKey::Reconciliation => view! { <Reconciliation /> }.into_any(),
NavKey::AuditLog => view! { <AuditLog /> }.into_any(),
```

### 7.3 后端接口清单（Tauri 2，camelCase）

| 命令 | 前端调用参数 | 返回 | 说明 |
|------|-------------|------|------|
| `importRawFile` | `{ filePath: String, batchId: String, sourceType: String }` | `{ rows: i32, attachmentId: i64 }` | 解析 TSV/CSV/Excel，写入 `raw_*` 与 `attachments` |
| `listRawRecords` | `{ sourceType?: String, batchId?: String, page: i32, pageSize: i32 }` | `RawRecordPage` | 分页查询原始记录 |
| `getRawRecord` | `{ id: i64 }` | `RawRecordDetail` | 获取单条记录详情（含附件、审计日志） |
| `reconcile` | `{ date: String }` | `ReconcileResult` | 自动勾稽银行/订单/汇总，生成 `transaction_summaries` |
| `listReconciliationItems` | `{ date?: String, status?: String, page: i32 }` | `ReconciliationPage` | 差异任务列表 |
| `reviewSummary` | `{ summaryId: i64, approve: bool, comment?: String }` | `Voucher?` | 审核通过时调用 `generateVoucher` |
| `generateVoucher` | `{ summaryId: i64 }` | `Voucher` | 内部命令，仅由后端/审核触发 |
| `listAuditLogs` | `{ entityType?: String, entityId?: i64, page: i32 }` | `AuditLogPage` | 审计日志查询 |
| `uploadAttachment` | `{ entityType: String, entityId: i64 }` | `Attachment` | 上传图片/PDF/Excel 到附件表 |

### 7.4 状态管理与数据流

参考 `voucher.rs` 的 LocalResource + Signal 模式：

```rust
let (page, set_page) = signal(1i32);
let (filter, set_filter) = signal(RawRecordFilter::default());
let records = LocalResource::new(move || {
    let f = filter.get();
    let p = page.get();
    async move { ipc::list_raw_records(&f, p).await }
});

let (selected_id, set_selected_id) = signal(Option::<i64>::None);
let detail = LocalResource::new(move || {
    let id = selected_id.get();
    async move {
        if let Some(id) = id { ipc::get_raw_record(id).await.ok() } else { None }
    }
});
```

交互流程：

1. 上传成功 → `records.refetch()` → 表格刷新；
2. 行点击 → `selected_id.set(Some(id))` → `detail` 自动加载右侧详情；
3. 对账日期选择 → 点击“开始对账” → 调用 `reconcile` → 成功后跳转/刷新 `Reconciliation` 页面的差异列表；
4. 差异行点击“通过” → 调用 `reviewSummary` → 成功后刷新列表并显示生成的凭证字号；
5. 所有后端调用失败统一通过 `error` Signal 在页面顶部展示错误信息。

### 7.5 页面布局规范

每个页面遵循 ForgeFin 统一布局：

```
Tabs（如果需要）
  ↓
SearchForm（筛选条件：日期、来源类型、批次号、状态）
  ↓
SummaryStats / KPI 卡片（总行数、已匹配、待审核、已生成凭证）
  ↓
ActionBar（导入、对账、导出、刷新）
  ↓
page-grid
  ├── data-table 区域（表格 + 分页）
  └── detail-panel 区域（记录详情 / 差异详情 / 审计日志）
```

* `RawData` 页面：左侧显示原始记录列表，右侧 `RecordDetail` 展示 JSON 字段、原始行号、来源文件、附件列表；
* `Reconciliation` 页面：左侧差异列表，右侧 `DiffReviewModal`（弹框）进行审核；
* `AuditLog` 页面：全宽表格，顶部放置过滤表单。

### 7.6 校验与异常处理

| 校验点 | 处理方式 |
|--------|----------|
| 文件类型 | 仅允许 `.tsv`、`.csv`、`.xlsx`；非法类型前端直接拦截 |
| 金额解析 | 后端使用 `rust_decimal::Decimal` 解析，失败行写入 `import_errors` 表并返回错误明细 |
| 日期格式 | 统一校验 `YYYY-MM-DD` 或 `YYYY-MM-DD HH:MM:SS`，失败行标记 |
| 批次号唯一 | `import_batch_id` 不可重复，重复时拒绝导入 |
| 必填字段 | `record_date`、`raw_data` 为空时拒绝插入 |
| 审核前置条件 | `reviewSummary` 仅允许对 `review_status='pending'` 的摘要进行操作 |
| 凭证平衡 | `generateVoucher` 生成前校验借贷相等，否则回滚并返回错误 |

### 7.7 UI 验收标准（新增）

| 编号 | 标准 | 检查方式 |
|------|------|-----------|
| **10** | 新增“原始凭证”导航菜单，包含“导入中心”“对账中心”“审计日志”三个入口 | 左侧导航栏可见且可点击 |
| **11** | 点击导航后页面正常渲染，无 JS 报错，浏览器控制台无红色错误 | 浏览器开发者工具检查 |
| **12** | `RawData` 页面支持 TSV 文件上传，上传完成后表格自动刷新，显示新导入的记录 | 手动上传 `tests/sample_data/health_company/bank_raw.tsv` |
| **13** | 点击原始记录行，右侧详情面板正确显示 `raw_data` JSON、来源文件、行号、附件列表 | 点击表格行验证 |
| **14** | `Reconciliation` 页面选择日期并点击“对账”，差异列表正确展示差额与状态 | 使用示例银行/订单数据验证 |
| **15** | 差异行点击“通过”后，列表状态变为 `approved`，凭证字号出现，审计日志新增记录 | 验证数据库与页面展示 |
| **16** | `AuditLog` 页面可按 `entityType` 过滤，分页正常 | 切换过滤条件验证 |
| **17** | 所有金额输入使用 `DecimalInput` 组件，保留两位小数、千分位显示 | 目视检查 |
| **18** | 所有新增页面通过 `cargo fmt && cargo check` | 本地运行 |

### 7.8 实施步骤建议

1. **第 1 步**：新增 `NavKey` 与导航树、路由分支、`pages/mod.rs` 注册；
2. **第 2 步**：创建基础占位页面 `raw_data.rs`、`reconciliation.rs`、`audit_log.rs`，确保可访问；
3. **第 3 步**：实现 `import_uploader` 组件与 `importRawFile` 后端命令；
4. **第 4 步**：实现 `raw_record_table` + `record_detail` + `listRawRecords`/`getRawRecord`；
5. **第 5 步**：实现 `reconcile` 后端逻辑与 `diff_list` + `diff_review_modal`；
6. **第 6 步**：实现 `generateVoucher` 与凭证分录写入，关联原始记录；
7. **第 7 步**：实现 `audit_log.rs` 页面与 `log_table` 组件；
8. **第 8 步**：联调示例数据（`tests/sample_data/health_company/*`），运行 `cargo fmt && cargo check`。

## 8. 下一步计划

在通过本阶段验收后，进入 **Phase 2**（自动化导入、凭证模板、OCR）并在 `docs/plan/phase2_plan.md` 中更新对应里程碑。

*文档中的所有业务编号、金额、日期均为示例数据，实际运行时请使用真实业务数据。*