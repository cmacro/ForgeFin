# 项目收支管理对动态原始凭证模型的扩展建议

## 1. 背景

在 `docs/analysis/dynamic-source-data-model.md` 中提出的元数据驱动模型，适合处理：

- 来源类型多样（银行流水、订单流水、费用报销）
- 字段结构灵活（不同企业字段不同）
- 附件类型多样（照片、PDF、合同）

但当业务规模扩大，尤其是出现**项目型企业**（建筑、咨询、广告、会展、研发等）时，仅依赖 `source_records.raw_data` 存储 JSON 动态字段会面临以下问题：

- 项目维度的收支需要独立核算
- 同一项目涉及多个阶段、多笔收入、多笔成本
- 需要按项目出损益表、现金流、成本明细
- 项目与客户/供应商/合同/发票强关联
- 项目内存在预算控制、阶段里程碑、分摊规则

这些需求如果全部塞进通用 `source_records` 表，会导致：

- 查询性能下降（JSON 无法高效聚合）
- 业务语义弱化（项目编号只是字符串）
- 无法做项目级对账、预算控制、阶段分析
- 凭证生成规则过于复杂

因此需要在**不破坏现有账套结构**的前提下，对动态模型进行分层扩展。

---

## 2. 核心原则

1. **账套结构不动**：会计科目表、凭证表、凭证分录表保持原样。
2. **原始凭证层不动**：`source_types`、`source_fields`、`source_records`、`attachments`、`audit_logs` 继续承担通用原始数据归档职责。
3. **新增业务维度层**：在原始记录之上，增加“项目”“合同”“阶段”等业务对象表，用于横向归集。
4. **凭证生成链路不变**：最终仍通过 `source_records` → `vouchers`，只是 `source_records` 可以关联更丰富的业务上下文。
5. **可插拔**：没有项目需求的企业，完全不需要启用这些表和页面。

---

## 3. 扩展后的整体架构

```text
原始文件/附件层
        ↓
source_types / source_fields / source_records（通用动态原始凭证层）
        ↓                    ↓                    ↓
    projects              contracts             invoices
   （项目档案）           （合同档案）           （发票档案）
        ↓                    ↓                    ↓
   project_phases        contract_items
  （项目阶段）            （合同明细）
        ↓                    ↓
   project_entries       contract_entries
  （项目收支记录）        （合同执行记录）
        ↓                    ↓                    ↓
   source_records.voucher_id ─────────────→ vouchers（正式记账凭证）
        ↓
   audit_logs（审计日志）
```

---

## 4. 建议新增表结构

### 4.1 项目档案表 `projects`

记录项目基本信息，按公司隔离。

```sql
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    project_no TEXT NOT NULL,          -- 项目编号
    project_name TEXT NOT NULL,        -- 项目名称
    project_type TEXT,                 -- 项目类型：工程 / 咨询 / 研发 / 其他
    customer_id INTEGER,               -- 关联客户
    manager_id INTEGER,                -- 项目负责人
    start_date TEXT,                   -- 开始日期
    end_date TEXT,                     -- 预计结束日期
    budget_amount DECIMAL(18,4),       -- 项目预算
    status TEXT DEFAULT 'active',      -- active / paused / completed / cancelled
    created_at TEXT,
    updated_at TEXT
);
```

### 4.2 项目阶段表 `project_phases`

用于拆分大型项目的里程碑或阶段，便于阶段核算。

```sql
CREATE TABLE project_phases (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    project_id INTEGER NOT NULL,
    phase_no TEXT NOT NULL,            -- 阶段编号
    phase_name TEXT NOT NULL,          -- 阶段名称
    plan_start TEXT,
    plan_end TEXT,
    actual_start TEXT,
    actual_end TEXT,
    budget_amount DECIMAL(18,4),
    status TEXT DEFAULT 'pending',     -- pending / in_progress / completed
    sort_order INTEGER DEFAULT 0,
    created_at TEXT
);
```

### 4.3 合同档案表 `contracts`

项目型企业通常以合同为主线，项目与合同可能一对一、一对多。

```sql
CREATE TABLE contracts (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    contract_no TEXT NOT NULL,         -- 合同编号
    contract_name TEXT,
    project_id INTEGER,                  -- 关联项目（可为空）
    customer_id INTEGER,
    supplier_id INTEGER,
    sign_date TEXT,
    amount DECIMAL(18,4),              -- 合同金额
    tax_amount DECIMAL(18,4),            -- 税额
    status TEXT DEFAULT 'active',        -- active / completed / terminated
    created_at TEXT
);
```

### 4.4 合同明细表 `contract_items`

拆分合同的收入/成本明细行。

```sql
CREATE TABLE contract_items (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    contract_id INTEGER NOT NULL,
    item_name TEXT NOT NULL,
    item_type TEXT,                    -- income / cost
    amount DECIMAL(18,4),
    plan_date TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT
);
```

### 4.5 项目收支记录表 `project_entries`

把与项目相关的原始记录，按业务语义重新归集到这里。

```sql
CREATE TABLE project_entries (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    project_id INTEGER NOT NULL,
    phase_id INTEGER,                    -- 可关联阶段
    contract_id INTEGER,                   -- 可关联合同
    entry_type TEXT NOT NULL,            -- income / cost / expense / refund
    entry_date TEXT,
    amount DECIMAL(18,4),
    currency TEXT DEFAULT 'CNY',
    counterpart_id INTEGER,              -- 客户/供应商 ID
    counterpart_type TEXT,               -- customer / supplier
    invoice_id INTEGER,                  -- 关联发票
    summary TEXT,
    source_record_id INTEGER,            -- 关联原始记录
    voucher_id INTEGER,                  -- 关联凭证
    status TEXT DEFAULT 'pending',       -- pending / reviewed / posted
    created_by INTEGER,
    created_at TEXT,
    reviewed_by INTEGER,
    reviewed_at TEXT
);
```

### 4.6 发票档案表 `invoices`

项目型企业发票管理非常重要，单独建表。

```sql
CREATE TABLE invoices (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    invoice_no TEXT NOT NULL,
    invoice_type TEXT,                 -- input / output
    project_id INTEGER,
    contract_id INTEGER,
    counterpart_id INTEGER,
    amount DECIMAL(18,4),
    tax_amount DECIMAL(18,4),
    issue_date TEXT,
    status TEXT DEFAULT 'unused',      -- unused / used / void
    source_record_id INTEGER,
    voucher_id INTEGER,
    created_at TEXT
);
```

---

## 5. 与现有动态模型的衔接方式

### 5.1 不替换，只增强

- `source_records` 继续作为“原始凭证归档池”，保存每一笔导入或手工录入的原始数据。
- `project_entries` 作为“业务维度归集表”，把属于项目的原始记录再做一次业务级汇总。
- 两者通过 `project_entries.source_record_id` 一对多或一对一关联。

### 5.2 动态字段 + 业务表各司其职

| 层级 | 负责内容 |
|------|----------|
| `source_fields` | 定义不同企业原始数据的字段 |
| `source_records.raw_data` | 保存导入的原始字段值 |
| `project_entries` | 保存项目级业务语义：收入/成本/阶段/合同 |
| `projects` / `contracts` / `invoices` | 保存主数据档案 |

### 5.3 凭证生成链路

凭证仍从 `source_records` 生成，但可以：

- 通过 `source_records.id` 关联 `project_entries`
- 凭证分录通过 `source_raw_id` 指向 `source_records`
- 同时可扩展 `voucher_project_refs` 表，记录凭证涉及的项目

```sql
CREATE TABLE voucher_project_refs (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    voucher_id INTEGER NOT NULL,
    project_id INTEGER,
    phase_id INTEGER,
    contract_id INTEGER,
    amount DECIMAL(18,4),
    ref_type TEXT                      -- income / cost / fee
);
```

这样查项目账时，可以从 `voucher_project_refs` 快速聚合，不需要扫描 JSON。

---

## 6. 典型业务流程示例

### 6.1 项目成本入账

1. 收到供应商发票和付款单，上传附件到 `attachments`。
2. 在 `source_records` 中录入一条 `project_cost` 类型的原始记录，动态字段包含项目编号。
3. 在 `project_entries` 中生成一条 `cost` 记录，关联 `project_id`、`contract_id`、`invoice_id`。
4. 审核通过后，生成凭证：借工程施工/项目成本，贷银行存款。
5. 同时在 `voucher_project_refs` 写入项目归集记录。

### 6.2 项目收入确认

1. 客户付款到账，银行流水导入 `source_records`。
2. 根据 `raw_data` 中的项目编号，关联到 `projects`。
3. 生成 `project_entries` 收入记录。
4. 根据合同 `contracts.amount` 与已收款累计，判断是预收款还是收入确认。
5. 生成凭证，并写入 `voucher_project_refs`。

---

## 7. 前端页面建议

按 ForgeFin 目录规则，新增以下页面和组件：

| 页面 | 路径 | 作用 |
|------|------|------|
| 项目管理 | `src/pages/projects.rs` | 项目档案增删改查 |
| 项目详情 | `src/pages/project_detail.rs` | 项目收支、阶段、损益 |
| 阶段管理 | `src/pages/project_phases.rs` | 项目阶段维护 |
| 合同管理 | `src/pages/contracts.rs` | 合同档案管理 |
| 发票管理 | `src/pages/invoices.rs` | 发票收发管理 |
| 项目凭证归集 | `src/pages/voucher_project_refs.rs` | 凭证与项目关联查询 |

组件：

- `src/components/project/project_selector.rs` — 项目选择器
- `src/components/project/project_summary_card.rs` — 项目收支卡片
- `src/components/project/phase_timeline.rs` — 阶段时间线
- `src/components/project/contract_items_table.rs` — 合同明细表

---

## 8. 对没有项目需求的企业的兼容性

- 新增表可以预先创建，但不强制使用。
- 没有 `project_id` 的 `source_records` 仍然走原有通用流程。
- 页面只在启用项目模块的企业菜单中显示，可通过 `company_features` 配置开关。

```sql
CREATE TABLE company_features (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    feature_code TEXT NOT NULL,        -- project / contract / invoice / multi_phase
    enabled INTEGER DEFAULT 0,
    created_at TEXT
);
```

---

## 9. 结论

> 当动态原始凭证模型遇到项目型企业的复杂核算需求时，不应把项目语义硬塞进通用 JSON 字段，而应在保留 `source_records` 通用归档能力的基础上，新增 `projects`、`project_phases`、`contracts`、`project_entries`、`invoices`、`voucher_project_refs` 等业务维度表；原始记录负责“从哪里来”，业务维度表负责“属于哪个项目/合同/阶段”，凭证负责“记到哪一科目”；三者通过外键衔接，既满足项目级核算，又不破坏现有账套结构。
