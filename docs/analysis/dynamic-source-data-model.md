# 动态原始凭证数据模型分析

## 1. 背景

不同企业、不同行业的原始凭证数据来源差异很大：

- 健康管理公司A：银行流水、订单流水、微信群备注
- 项目型企业A（建筑、咨询）：项目编号、项目成本、阶段、供应商
- 贸易/电商企业：采购单、销售单、物流费、发票
- 通用企业：收据照片、合同 PDF、银行回单、审批截图

因此，原始凭证存储不能写死表结构，需要一套**可扩展、元数据驱动**的动态模型。

---

## 2. 核心思路：分层 + 元数据驱动

把原始凭证存储拆成五层：

| 层级 | 作用 | 是否固定 |
|------|------|----------|
| 凭证来源层 | 定义企业有哪些数据来源 | 固定表 + 动态配置 |
| 动态字段层 | 定义每个数据源的字段、类型、校验规则 | 动态配置 |
| 原始记录层 | 存储每笔原始数据 | 固定核心字段 + 动态 JSON |
| 附件/文档层 | 保存照片、PDF、合同等 | 固定表 |
| 映射规则层 | 把字段映射到会计科目、生成凭证模板 | 动态配置 |

---

## 3. 建议表结构

### 3.1 数据源定义表 `source_types`

每个企业配置自己的数据来源。

```sql
CREATE TABLE source_types (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    code TEXT NOT NULL,              -- 如 bank_flow, order_flow, project_cost
    name TEXT NOT NULL,              -- 显示名
    category TEXT NOT NULL,          -- bank / order / project / expense / income / other
    is_system INTEGER DEFAULT 0,     -- 1=内置，0=企业自定义
    icon TEXT,
    created_at TEXT
);
```

示例：

| company_id | code | name | category |
|------------|------|------|----------|
| 1 | bank_flow | 银行流水 | bank |
| 1 | order_flow | 订单流水 | order |
| 1 | project_cost | 项目成本 | project |
| 1 | reimbursement | 费用报销 | expense |

### 3.2 动态字段定义表 `source_fields`

每个数据源定义自己的字段。

```sql
CREATE TABLE source_fields (
    id INTEGER PRIMARY KEY,
    source_type_id INTEGER NOT NULL,
    field_code TEXT NOT NULL,        -- 如 amount, counterparty, project_no
    field_name TEXT NOT NULL,        -- 显示名
    data_type TEXT NOT NULL,         -- string / number / date / boolean / enum / file
    is_required INTEGER DEFAULT 0,
    is_amount INTEGER DEFAULT 0,     -- 是否为金额字段，用于汇总
    is_debit INTEGER DEFAULT 0,      -- 1=借方，-1=贷方，0=非金额
    sort_order INTEGER DEFAULT 0,
    options TEXT,                    -- enum 可选值，JSON 数组
    validation_rule TEXT,            -- 简单正则或表达式
    map_to_account INTEGER           -- 是否可映射到会计科目
);
```

示例：项目成本数据源

| source_type_id | field_code | field_name | data_type | is_amount | is_debit |
|----------------|------------|------------|-----------|-----------|----------|
| 3 | project_no | 项目编号 | string | 0 | 0 |
| 3 | project_name | 项目名称 | string | 0 | 0 |
| 3 | cost_amount | 成本金额 | number | 1 | 1 |
| 3 | invoice_no | 发票号 | string | 0 | 0 |
| 3 | vendor | 供应商 | string | 0 | 0 |

### 3.3 原始记录主表 `source_records`

核心字段固定，动态字段用 JSON 保存。

```sql
CREATE TABLE source_records (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    source_type_id INTEGER NOT NULL,
    import_batch_id TEXT,
    source_file_name TEXT,
    source_row_no INTEGER,
    record_no TEXT,                  -- 业务单号/流水号
    record_date TEXT,                -- 业务日期
    amount_total DECIMAL(18,4),      -- 金额合计，自动从动态字段计算
    currency TEXT DEFAULT 'CNY',
    counterpart_info TEXT,           -- 对方信息
    summary TEXT,                    -- 摘要/事由
    raw_data TEXT NOT NULL,          -- 动态字段 JSON
    status TEXT DEFAULT 'pending',   -- pending / matched / reviewed / void
    voucher_id INTEGER,
    created_by INTEGER,
    created_at TEXT,
    reviewed_by INTEGER,
    reviewed_at TEXT
);
```

`raw_data` 示例：

```json
{
  "project_no": "PJ2026001",
  "project_name": "XX 办公楼装修",
  "cost_amount": 50000.00,
  "invoice_no": "INV123456",
  "vendor": "XX 建材公司",
  "stage": "一期工程"
}
```

### 3.4 附件表 `attachments`

```sql
CREATE TABLE attachments (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    entity_type TEXT NOT NULL,       -- source_record / voucher / summary
    entity_id INTEGER NOT NULL,
    attachment_type TEXT,            -- receipt / invoice / contract / photo / bank_slip / other
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_hash TEXT,
    ocr_text TEXT,                   -- OCR 文本，可搜索
    uploaded_by INTEGER,
    uploaded_at TEXT
);
```

### 3.5 字段映射与凭证模板 `source_mapping_rules`

按企业配置：字段条件 → 借贷科目 → 凭证模板。

```sql
CREATE TABLE source_mapping_rules (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    source_type_id INTEGER NOT NULL,
    field_conditions TEXT,           -- JSON：{ "project_type": "装修" }
    debit_account_id INTEGER,
    credit_account_id INTEGER,
    fee_account_id INTEGER,
    voucher_template TEXT,
    priority INTEGER DEFAULT 0
);
```

### 3.6 审计日志表 `audit_logs`

```sql
CREATE TABLE audit_logs (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id INTEGER NOT NULL,
    action TEXT NOT NULL,            -- import / create / update / review / generate_voucher / void
    operator_id INTEGER,
    old_values TEXT,
    new_values TEXT,
    attachment_ids TEXT,
    remark TEXT,
    operated_at TEXT
);
```

---

## 4. 如何解决“动态类型”问题

| 问题 | 解决方式 |
|------|----------|
| 不同企业数据来源不同 | `source_types` 按企业配置 |
| 不同数据源字段不同 | `source_fields` 动态定义字段 |
| 金额方向不固定 | `is_amount` + `is_debit` 自动识别借/贷 |
| 凭证科目映射不同 | `source_mapping_rules` 按条件映射 |
| 附件类型多样 | `attachment_type` 可扩展 |
| 单据内容可搜索 | 支持 OCR 文本存入 `ocr_text` |

---

## 5. 典型场景示例

### 5.1 健康管理公司A（样本企业A）

- 数据源：`bank_flow`、`order_flow`、`daily_summary`
- 订单流水字段：订单金额、手续费、实收金额、支付方式、结算日期
- 映射规则：POS 收入 → 借银行存款 / 借财务费用 / 贷主营业务收入

### 5.2 项目型企业A（建筑、咨询）

- 数据源：`project_cost`、`project_income`、`bank_flow`
- 项目成本字段：项目编号、项目名称、阶段、成本金额、供应商、发票号
- 映射规则：按项目编号归集成本

### 5.3 贸易/电商企业

- 数据源：`purchase_order`、`sales_order`、`logistics_fee`、`bank_flow`
- 采购单据字段：供应商、商品、数量、单价、税额、发票
- 附件：采购合同、入库单、发票照片

---

## 6. 前端页面与组件建议

按 ForgeFin 目录规则：

| 页面 | 路径 |
|------|------|
| 数据源配置 | `src/pages/source_type_settings.rs` |
| 导入中心 | `src/pages/import_center.rs` |
| 原始记录库 | `src/pages/source_records.rs` |
| 附件管理 | `src/pages/attachments.rs` |
| 映射规则 | `src/pages/mapping_rules.rs` |
| 凭证生成 | `src/pages/voucher_generation.rs` |

组件：

- `src/components/source/source_field_editor.rs`
- `src/components/source/import_uploader.rs`
- `src/components/source/record_detail.rs`
- `src/components/source/review_panel.rs`

---

## 7. 结论

> 用“数据源定义 + 动态字段 + JSON 原始记录 + 附件 + 映射规则 + 审计日志”这套元数据驱动模型，把不同企业、不同项目、不同单据类型的原始凭证统一存储；每种企业的凭证来源和字段都是可配置的，系统只固化“来源层、字段层、记录层、附件层、规则层、日志层”这六层骨架。
