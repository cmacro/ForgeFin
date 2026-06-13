# ForgeFin 开发计划 (Project Roadmap)

## 1. 核心愿景
构建一个专业的**工程项目财务核算系统**。区别于通用记账软件，本项目深度适配建设工程行业，支持项目独立核算、凭证化记账、成本细分（料/工/费/税）及复杂的资金往来跟踪。

## 2. 架构标准
- **模式**: DDD Lite + Clean Architecture。
- **技术栈**: Rust (Core) $\rightarrow$ SQLite (Persistence) $\rightarrow$ Tauri (Shell) $\rightarrow$ Leptos (UI)。
- **交付标准 (DoD)**:
    - **单元测试**: 核心逻辑覆盖率 $> 80\%$，通过 `cargo test`。
    - **人为验收**: 每个里程碑必须提供可运行的入口（CLI 或 UI），通过 `cargo run` 或 `cargo tauri dev` 验证功能。

## 3. 分阶段开发里程碑

### 阶段 1: 基础设施与数据建模 (Foundation)
- **核心任务**:
    - [ ] 建立 SQLite 数据库迁移机制 (`sqlx` migrations)。
    - [ ] 实现领域模型：`Project` (项目), `Account` (账户), `Category` (成本类目)。
    - [ ] 实现基于 Repository 模式的数据访问层。
- **验证方案**:
    - **测试**: 编写 DB 读写单元测试。
    - **验收**: 运行简单的 CLI 工具，能创建项目并持久化到本地 DB。

### 阶段 2: 凭证核算引擎 (Accounting Engine)
- **核心任务**:
    - [ ] 实现**凭证系统 (Voucher System)**：支持单笔交易的结构化记录。
    - [ ] 实现**成本归集逻辑**：严格区分 $\text{材料费} \rightarrow \text{劳务费} \rightarrow \text{日常费} \rightarrow \text{税金}$。
    - [ ] 实现项目级利润计算：$\text{收入合计} - \text{支出合计} = \text{项目利润}$。
- **验证方案**:
    - **测试**: 针对不同凭证组合的利润计算单元测试。
    - **验收**: 通过 CLI 模拟输入一组凭证，输出正确的项目汇总报表。

### 3. 专项业务模块 (Specialized Modules)
- **核心任务**:
    - [ ] **税务管理**: 实现各项目预缴税费的独立记录与汇总。
    - [ ] **劳务分包**: 实现分包单位合同金额、已付、未付余款跟踪。
    - [ ] **保证金管理**: 实现投标/履约保证金的存取跟踪。
    - [ ] **发票跟踪**: 实现专票/普票状态与付款记录的关联。
- **验证方案**:
    - **测试**: 针对余款计算、税金汇总的边界值测试。
    - **验收**: 通过 CLI 验证特定业务场景（如：分包款支付 $\rightarrow$ 更新余款 $\rightarrow$ 关联发票）。

### 4. 界面集成与 Tauri 桥接 (UI & Integration)
- **核心任务**:
    - [ ] 定义 Tauri Commands 接口层，将后端 Service 暴露给前端。
    - [ ] 开发 Leptos 基础页面：项目概览页、凭证录入页、汇总报表页。
    - [ ] 实现前端状态管理与异步数据刷新。
- **验证方案**:
    - **测试**: 编写 Tauri Command 的集成测试。
    - **验收**: 启动 `cargo tauri dev`，完成一次从“创建项目 $\rightarrow$ 录入凭证 $\rightarrow$ 查看报表”的完整闭环。

### 5. AI 智能增强层 (AI Layer)
- **核心任务**:
    - [ ] 集成本地 LLM (Qwen-Coder) 推理接口。
    - [ ] 实现自然语言 $\rightarrow$ 结构化 JSON 凭证的 Prompt 链。
    - [ ] 实现“AI 辅助审核”功能：检测凭证类目与摘要是否匹配。
- **验证方案**:
    - **测试**: 建立测试集（自然语言描述 $\rightarrow$ 预期 JSON），验证模型准确率。
    - **验收**: 在 UI 中输入“昨天给张三付了劳务费 5000 元”，系统自动生成预览凭证并允许用户确认入账。
