# ForgeFin 文档索引

本文档汇总 `docs/` 目录下所有文档，方便快速查找。索引按主题分类，每个条目包含文件路径和一句话摘要。

---

## 一、分析与设计文档

| 文件 | 摘要 |
|------|------|
| [docs/业务数据源分析.md](业务数据源分析.md) | 梳理银行流水、订单流水、人工汇总三类数据来源的字段、核对关系与质量问题 |
| [docs/业务模块分析.md](业务模块分析.md) | 梳理 ForgeFin 业务模块划分与功能边界 |
| [docs/analysis/source-data-preservation.md](analysis/source-data-preservation.md) | 针对当前企业的原始凭证保存、审核与溯源方案（固定表结构） |
| [docs/analysis/dynamic-source-data-model.md](analysis/dynamic-source-data-model.md) | 面向多企业、多行业的动态可扩展原始凭证元数据模型 |
| [docs/analysis/project-accounting-extension.md](analysis/project-accounting-extension.md) | 项目型企业扩展方案：在动态模型之上增加项目/合同/阶段/发票维度，不破坏现有账套 |
| [docs/stylelayout.md](stylelayout.md) | UI 风格与布局相关说明 |

---

## 二、开发计划

| 文件 | 摘要 |
|------|------|
| [docs/开发计划.md](开发计划.md) | 总体开发计划与里程碑 |
| [docs/plan/phase0_plan.md](plan/phase0_plan.md) | Phase 0：基础设施（Pre‑MVP），含数据库、Tauri 命令、认证、公司管理、备份恢复 |
| [docs/plan/phase1_plan.md](plan/phase1_plan.md) | Phase 1：凭证核心（MVP 核心），含科目、客户/供应商、凭证录入/查询/审核/打印 |
| [docs/plan/phase2_plan.md](plan/phase2_plan.md) | Phase 2 计划 |
| [docs/plan/phase3_plan.md](plan/phase3_plan.md) | Phase 3 计划 |
| [docs/plan/phase4_plan.md](plan/phase4_plan.md) | Phase 4 计划 |
| [docs/plan/phase5_plan.md](plan/phase5_plan.md) | Phase 5 计划 |

---

## 三、专项方案

| 文件 | 摘要 |
|------|------|
| [docs/companies-selection-plan.md](companies-selection-plan.md) | 公司/账套选择页设计方案 |
| [docs/company-selection-page-plan.md](company-selection-page-plan.md) | 公司选择页详细设计 |
| [docs/测试方案.md](测试方案.md) | 项目测试策略与方案 |

---

## 四、开发日志与问题排查

| 文件 | 摘要 |
|------|------|
| [docs/logs/2026-07-06-Phase0-Phase1-开发日报.md](logs/2026-07-06-Phase0-Phase1-开发日报.md) | Phase0 / Phase1 开发日报 |
| [docs/logs/summary.md](logs/summary.md) | 开发总结 |
| [docs/logs/fix_summary.md](logs/fix_summary.md) | 问题修复汇总 |
| [docs/logs/agent_fix_guide.md](logs/agent_fix_guide.md) | Agent 修复指南 |
| [docs/logs/WSL开发环境问题排查.md](logs/WSL开发环境问题排查.md) | WSL 开发环境问题排查记录 |
| [docs/logs/结构提示词样式.md](logs/结构提示词样式.md) | 结构提示词样式参考 |

---

## 五、工具与资源

| 文件 | 摘要 |
|------|------|
| [docs/tools/README.md](tools/README.md) | 工具目录说明 |

---

## 六、样式资源

| 文件 | 摘要 |
|------|------|
| [docs/styletheme.css](styletheme.css) | 主题样式 CSS |
| [docs/logo_v1.png](logo_v1.png) | Logo 版本 1 |
| [docs/logo_v2.jpg](logo_v2.jpg) | Logo 版本 2 |
| [docs/style.png](style.png) | 风格参考图 |

---

## 使用方式

1. 按上方主题分类快速定位所需文档。
2. 点击对应相对路径即可打开。
3. 新增文档后请同步更新本索引。
