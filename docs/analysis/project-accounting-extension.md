# 项目收支管理对动态原始凭证模型的扩展建议

## 1. 背景

在 `docs/analysis/dynamic-source-data-model.md` 中提出的元数据驱动模型，适合处理：

- 来源类型多样（银行流水、订单流水、费用报销）
- 字段结构灵活（不同企业字段不同）
- 附件类型多样（照片、PDF、合同）

但当业务规模扩大，尤其是出现**项目型企业A**（建筑、咨询、广告、会展、研发等）时，仅依赖 `source_records.raw_data` 存储 JSON 动态字段会面临以下问题：

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
2. **原始凭证层不动**：`source_types`、`source_fields`、`source_records`、`attachments`、`audit_logs` 继续承担通用原始凭证归档职责。
3. **新增业务维度层**：在原始记录之上，增加"项目""合同""阶段"等业务对象表，用于横向归集。
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
    project_no TEXT NOT NULL,
    project_name TEXT NOT NULL,
    project_type TEXT,
    customer_id INTEGER,
    manager_id INTEGER,
    start_date TEXT,
    end_date TEXT,
    budget_amount DECIMAL(18,4),
    status TEXT DEFAULT 'active',
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
    phase_no TEXT NOT NULL,
    phase_name TEXT NOT NULL,
    plan_start TEXT,
    plan_end TEXT,
    actual_start TEXT,
    actual_end TEXT,
    budget_amount DECIMAL(18,4),
    status TEXT DEFAULT 'pending',
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
    contract_no TEXT NOT NULL,
    contract_name TEXT,
    project_id INTEGER,
    customer_id INTEGER,
    supplier_id INTEGER,
    sign_date TEXT,
    amount DECIMAL(18,4),
    tax_amount DECIMAL(18,4),
    status TEXT DEFAULT 'active',
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
    item_type TEXT,
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
    phase_id INTEGER,
    contract_id INTEGER,
    entry_type TEXT NOT NULL,
    entry_date TEXT,
    amount DECIMAL(18,4),
    currency TEXT DEFAULT 'CNY',
    counterpart_id INTEGER,
    counterpart_type TEXT,
    invoice_id INTEGER,
    summary TEXT,
    source_record_id INTEGER,
    voucher_id INTEGER,
    status TEXT DEFAULT 'pending',
    fiscal_year INTEGER,
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
    invoice_type TEXT,
    project_id INTEGER,
    contract_id INTEGER,
    counterpart_id INTEGER,
    amount DECIMAL(18,4),
    tax_amount DECIMAL(18,4),
    issue_date TEXT,
    status TEXT DEFAULT 'unused',
    source_record_id INTEGER,
    voucher_id INTEGER,
    created_at TEXT
);
```

### 4.7 质保金表 `project_retentions`

独立管理项目质保金，支持跨年跟踪。

```sql
CREATE TABLE project_retentions (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    project_id INTEGER NOT NULL,
    contract_id INTEGER,
    invoice_id INTEGER,
    amount DECIMAL(18,4),
    release_date TEXT,
    status TEXT DEFAULT 'held',
    retained_at TEXT,
    released_at TEXT,
    voucher_id INTEGER,
    remark TEXT,
    created_at TEXT
);
```

---

## 5. 与现有动态模型的衔接方式

### 5.1 不替换，只增强

- `source_records` 继续作为"原始凭证归档池"，保存每一笔导入或手工录入的原始数据。
- `project_entries` 作为"业务维度归集表"，把属于项目的原始记录再做一次业务级汇总。
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
    ref_type TEXT
);
```

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
    feature_code TEXT NOT NULL,
    enabled INTEGER DEFAULT 0,
    created_at TEXT
);
```

---

## 9. 长周期项目处理策略

项目型企业常见项目周期1年以上，甚至3-5年。需要从架构层面解决跨财年持续跟踪问题。

### 9.1 核心挑战

| 挑战 | 说明 |
|------|------|
| 跨财年收入确认 | 不同财年需分别确认收入/成本 |
| 预算占用 | 项目预算需跨年预留 |
| 质保金账龄 | 质保金可能2年后才到期收回 |
| 阶段里程碑 | 阶段可能跨越多个财年 |
| 历史数据查询 | 项目历史收支需长期保存可查 |

### 9.2 项目生命周期状态机

| 状态 | 说明 | 关键动作 |
|------|------|---------|
| `draft` | 立项待审批 | 仅项目信息录入，不参与财务核算 |
| `active` | 执行中 | 正常收支录入，实时更新项目账 |
| `completed` | 完工待结算 | 停止新增收支，等待结算价确定 |
| `settled` | 已结算 | 结算价确定，质保金独立核算 |
| `warranty` | 质保期 | 质保金留置，到期后触发收款提醒 |
| `archived` | 已归档 | 仅供查询，不再更新 |

### 9.3 跨财年收支归集

`project_entries` 的 `fiscal_year` 字段支持按财年独立核算：

```sql
ALTER TABLE project_entries ADD COLUMN fiscal_year INTEGER;

SELECT
    fiscal_year,
    SUM(CASE WHEN entry_type = 'income' THEN amount ELSE 0 END) as income_amount,
    SUM(CASE WHEN entry_type = 'cost' THEN amount ELSE 0 END) as cost_amount
FROM project_entries
WHERE project_id = ?
GROUP BY fiscal_year;
```

### 9.4 质保金跨期管理

独立质保金表支持2年+账龄跟踪：

```sql
CREATE TABLE project_retentions (
    id INTEGER PRIMARY KEY,
    company_id INTEGER NOT NULL,
    project_id INTEGER NOT NULL,
    contract_id INTEGER,
    invoice_id INTEGER,
    amount DECIMAL(18,4),
    release_date TEXT,
    status TEXT DEFAULT 'held',
    voucher_id INTEGER,
    created_at TEXT
);
```

### 9.5 期间结转处理

每财年结束，需对进行中项目做结转：

```sql
INSERT INTO project_yearly_summary (
    company_id, project_id, fiscal_year,
    opening_balance, contract_amount,
    income_ytd, cost_ytd, closing_balance, created_at
)
SELECT
    company_id, id, 2023,
    (SELECT closing_balance FROM project_yearly_summary WHERE project_id = projects.id AND fiscal_year = 2022),
    budget_total, 0, 0, 0, datetime('now')
FROM projects WHERE status IN ('active', 'completed');
```

---

## 10. 结论

> 当动态原始凭证模型遇到项目型企业的复杂核算需求时，不应把项目语义硬塞进通用 JSON 字段，而应在保留 `source_records` 通用归档能力的基础上，新增 `projects`、`project_phases`、`contracts`、`project_entries`、`invoices`、`voucher_project_refs` 等业务维度表；原始记录负责"从哪里来"，业务维度表负责"属于哪个项目/合同/阶段"，凭证负责"记到哪一科目"；三者通过外键衔接，既满足项目级核算，又不破坏现有账套结构。
>
> 长周期项目需额外处理：状态机管理项目生命周期、fiscal_year 字段支持跨财年归集、独立质保金表管理2年+账龄、财年结转保证期间报表连续性。
