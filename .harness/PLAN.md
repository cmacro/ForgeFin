# ForgeFin 自动化开发主计划 (Agent-Ready)

本计划专为 AI Agent 自动开发设计。每个阶段采用 **“规划-实施-验证”** 闭环。

---

## 阶段 1：底座建设与核心账务引擎 (Foundation & Core Ledger)
**目标**：建立符合会计准则的底层数据结构。

- [ ] **1.1 数据库架构 (Schema)**: 建立 `projects`, `accounts`, `vouchers`, `voucher_items` 表。
    - *Skill*: `sqlite-repository`
    - *产出*: `src-tauri/migrations/` 迁移脚本。
- [ ] **1.2 领域实体 (Entities)**: 定义 `Voucher` 聚合根与 `Project` 实体。
    - *Skill*: `rust-architecture`, `voucher-system`
    - *产出*: `src-tauri/src/domain/ledger/`
- [ ] **1.3 核心规则 (Logic)**: 实现借贷平衡校验 (`total_debit == total_credit`)。
    - *验证*: `cargo test domain::ledger::tests`
- **Agent 执行指令**: “优先执行数据库迁移，随后建立领域模型并编写借贷平衡的单元测试。”

---

## 阶段 2：项目收支核算模块 (Project Accounting - Module 1)
**目标**：实现业务动作到凭证的自动化转换。

- [ ] **2.1 业务服务层 (Services)**: 实现 `ProjectService` 处理收支录入。
    - *逻辑*: 录入收支时，自动生成关联项目的凭证记录。
    - *产出*: `src-tauri/src/application/services/project_service.rs`
- [ ] **2.2 API 接口 (Commands)**: 暴露 Tauri 命令 `add_project_record`。
    - *Skill*: `tauri-command`
- [ ] **2.3 响应式 UI**: 实现项目列表与录入表单。
    - *Skill*: `leptos-ui`
- **Agent 执行指令**: “在 Service 层实现收支逻辑，确保每一笔收支都对应生成一张平衡凭证。通过 UI 触发命令验证链路。”

---

## 阶段 3：资金流转与发票管理 (Finance & Invoice - Module 7, 8)
**目标**：管理银行账户流水与税务合规。

- [ ] **3.1 账户管理**: 实现 `BankAccounts` 增删改查。
- [ ] **3.2 流水继承逻辑**: 实现基于 `=$P4` 的批量填充逻辑（模拟 Excel 效率）。
- [ ] **3.3 发票跟踪**: 在凭证行增加 `InvoiceStatus` 字段。
- **Agent 执行指令**: “扩展领域模型以支持多银行账户，在凭证系统中引入发票状态字段。”

---

## 阶段 4：合同与资源管理 (Contract & Resources - Module 3, 4, 5)
**目标**：管理外部契约与押金。

- [ ] **4.1 材料/劳务合同**: 建立 `Contracts` 模型，关联凭证系统。
- [ ] **4.2 保证金系统**: 实现保证金的“缴纳-退还”全生命周期状态机。
- **Agent 执行指令**: “开发合同模型，确保合同支付进度能从凭证数据中实时统计。”

---

## 阶段 5：税务、报销与废旧物资 (Tax & Expense - Module 2, 6, 9)
**目标**：细化成本归集。

- [ ] **5.1 自动化税务归集**: 根据凭证摘要自动分类税务类型。
- [ ] **5.2 报销工作流**: 实现多类目报销（差旅、办公等）的快速分录生成。
- **Agent 执行指令**: “利用正则表达式或预定义规则，根据凭证内容自动填充税务科目。”

---

## 阶段 6：系统集成与 AI 自动化 (Integration & AI)
**目标**：整体优化与自然语言交互。

- [ ] **6.1 数据一致性校验**: 编写全局审计脚本，验证所有项目余额与凭证合计数一致。
- [ ] **6.2 AI 记账**: 集成 `llm-provider` 技能，实现自然语言生成凭证 JSON。
- **Agent 执行指令**: “集成本地模型，将用户输入转换为结构化会计凭证。”

---

## 验收与质量守则 (DoD)
1. **禁止越级**: 严禁在底层模型未通过测试时开发 UI。
2. **测试覆盖**: 核心财务逻辑（计算、平衡、汇总）必须有单元测试。
3. **Skill 绑定**: 每次切换阶段，必须重新读取对应的 `SKILL.md`。
