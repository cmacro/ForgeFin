# 原始凭证保存方案分析

## 1. 背景

根据 `docs/业务数据源分析.md` 的来源分析，当前 ForgeFin 面临的原始凭证数据来源包括：

- 银行流水（`银行流水.tsv`）
- 订单流水（`订单流水.tsv`）
- 数据汇总（人工整理，备注来自微信群）
- 未来可能涉及的收据照片、合同 PDF、发票扫描件等附件

这些原始凭证需要满足：

- **可追溯**：从记账凭证能反查到原始文件和原始行
- **可审核**：人工汇总草稿需经审核后才能生成正式凭证
- **可勾稽**：银行流水、订单流水、数据汇总三方金额需对得上
- **防篡改**：原始导入数据原则上只读

---

## 2. 核心保存思路

把“原始凭证”拆成四层：

```text
原始文件层（附件 + hash）
        ↓
原始数据层（raw_bank_statements / raw_order_statements）
        ↓
汇总草稿层（transaction_summaries，人工整理、待审核）
        ↓
正式凭证层（vouchers）
        ↓
审计日志层（audit_logs）
```

---

## 3. 建议表结构

### 3.1 原始银行流水表 `raw_bank_statements`

| 字段 | 说明 |
|------|------|
| `id` | 自增主键 |
| `import_batch_id` | 导入批次号，如 `BANK_20260715_001` |
| `source_file_name` | 原文件名，如 `银行流水.tsv` |
| `source_row_no` | 原文件行号 |
| `transaction_time` | 交易时间 |
| `counterparty` | 对方单位 |
| `in_amount` | 转入金额 |
| `out_amount` | 转出金额 |
| `balance` | 余额 |
| `summary` | 摘要 |
| `remarks` | 附言 |
| `voucher_id` | 关联生成的记账凭证（可为空） |
| `created_at` | 导入时间 |

### 3.2 原始订单流水表 `raw_order_statements`

| 字段 | 说明 |
|------|------|
| `id` | 自增主键 |
| `import_batch_id` | 导入批次号 |
| `source_file_name` | 原文件名，如 `订单流水.tsv` |
| `source_row_no` | 原文件行号 |
| `icbc_order_no` | 工行订单号（唯一） |
| `third_order_no` | 微信/支付宝流水号 |
| `merchant_no` | 商户编号 |
| `transaction_time` | 交易时间 |
| `order_amount` | 订单金额 |
| `fee_amount` | 手续费 |
| `net_amount` | 商户实收金额 |
| `payment_method` | 支付方式 |
| `debit_credit_flag` | 借贷记标识 |
| `settlement_date` | 结算日期 |
| `voucher_id` | 关联生成的记账凭证 |
| `matched_bank_tx_id` | 匹配到的银行流水 ID |
| `created_at` | 导入时间 |

### 3.3 数据汇总草稿表 `transaction_summaries`

| 字段 | 说明 |
|------|------|
| `id` | 自增主键 |
| `date` | 日期 |
| `receipt_no` | 收据编号 |
| `category` | 一级分类：收入 / 支出 |
| `project` | 项目：营业收入、手续费、产品成本等 |
| `reason` | 事由 |
| `income_amount` | 收入 |
| `expense_amount` | 支出 |
| `balance` | 余额 |
| `remarks` | 备注 |
| `source_type` | 来源类型：`bank` / `order` / `manual` |
| `source_id` | 关联的原始表 ID |
| `voucher_id` | 关联生成的记账凭证 |
| `review_status` | 待审核 / 已审核 / 已驳回 |
| `reviewer_id` | 审核人 |
| `reviewed_at` | 审核时间 |
| `created_at` | 创建时间 |

### 3.4 附件表 `attachments`

| 字段 | 说明 |
|------|------|
| `id` | 自增主键 |
| `entity_type` | 关联对象类型：`raw_bank` / `raw_order` / `summary` / `voucher` |
| `entity_id` | 关联对象 ID |
| `file_name` | 文件名 |
| `file_path` | 存储路径 |
| `file_hash` | MD5/SHA256，防篡改 |
| `uploaded_by` | 上传人 |
| `uploaded_at` | 上传时间 |

### 3.5 审计日志表 `audit_logs`

| 字段 | 说明 |
|------|------|
| `id` | 自增主键 |
| `entity_type` | `summary` / `raw_bank` / `raw_order` / `voucher` |
| `entity_id` | 对象 ID |
| `action` | 创建 / 修改 / 审核 / 生成凭证 / 驳回 |
| `old_values` | 修改前 JSON |
| `new_values` | 修改后 JSON |
| `operator_id` | 操作人 |
| `operated_at` | 操作时间 |
| `remark` | 操作说明 |

---

## 4. 审核与溯源机制

### 4.1 导入即归档

- 导入 TSV 时，原文件整体存入 `attachments`，并计算文件 hash。
- 解析后的每一行写入 `raw_bank_statements` 或 `raw_order_statements`，保留 `source_row_no`，方便定位到原文件具体行。

### 4.2 人工汇总必须关联来源

- 财务人员在 `transaction_summaries` 录入时，尽量从 `raw_*` 表中选择来源。
- 若来源是微信群等非结构化信息，来源类型填 `manual`，并在 `remarks` 中说明，必要时上传微信群截图到 `attachments`。

### 4.3 三方勾稽后再生成凭证

- 每日比对：订单流水 `settlement_date` 的实收合计 vs 银行流水同日的 POS 清算转入。
- 差异进入“待核对任务”，人工确认后再生成凭证。
- 只有 `review_status = 已审核` 的 `transaction_summaries` 才允许生成正式凭证。

### 4.4 凭证反向溯源

- 生成的 `vouchers` 记录保存 `source_summary_id`。
- 凭证分录行保存 `source_raw_id`（指向具体银行/订单流水）。
- 查询凭证时，可一键展开：原始文件 → 原始行 → 汇总草稿 → 审核记录。

### 4.5 审计日志不可删

- 任何对 `transaction_summaries` 的修改都写入 `audit_logs`。
- 原始导入表（`raw_*`）只读，若发现错误，不直接修改原表，而是通过“调整记录”或“异常标记”处理。

---

## 5. 结合 ForgeFin 的落地建议

按项目目录规则：

- 页面放 `src/pages/`：
  - `src/pages/raw_data.rs`：原始数据导入/查看
  - `src/pages/reconciliation.rs`：对账
  - `src/pages/audit_log.rs`：审计日志
- 组件放 `src/components/`：
  - `src/components/import/`：TSV 导入
  - `src/components/reconciliation/`：对账差异列表
- 后端命令遵循 Tauri 2 camelCase：前端 `invoke` 传 `importBatchId`、`sourceFileName` 等。
- 数据层用 `rusqlite` 裸 SQL，不引入 ORM。
- 金额统一用 `rust_decimal`，避免浮点误差。

---

## 6. 结论

> 把银行流水、订单流水作为只读原始档案整体入库；把人工汇总作为可审核的凭证草稿；用“来源文件 + 行号 + 批次 + 附件 hash”保证可追溯；用稽核日志保证可审计；只有勾稽通过并审核通过的草稿才生成正式凭证。
