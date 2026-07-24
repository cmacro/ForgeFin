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

## 6. 下一步计划
在通过本阶段验收后，进入 **Phase 2**（自动化导入、凭证模板、OCR）并在 `docs/plan/phase2_plan.md` 中更新对应里程碑。

*文档中的所有业务编号、金额、日期均为脱敏示例，实际运行时请使用真实业务数据。*