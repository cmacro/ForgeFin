# ForgeFin 文档索引

本文档汇总 `docs/` 目录下所有文档，方便快速查找。索引按主题分类，每个条目包含文件路径和一句话摘要。

---

## 一、分析与设计文档

| 文件 | 摘要 |
|------|------|
| [docs/analysis/health-source-data-analysis.md](analysis/health-source-data-analysis.md) | 健康管理公司(样本企业 A)的数据来源、字段可用性、核对关系与凭证生成规则(含银行流水余额业务重要性章节) |
| [docs/analysis/source-data-preservation.md](analysis/source-data-preservation.md) | 针对当前企业的原始凭证保存、审核与溯源方案(固定表结构) |
| [docs/analysis/dynamic-source-data-model.md](analysis/dynamic-source-data-model.md) | 面向多企业、多行业的动态可扩展原始凭证元数据模型 |
| [docs/analysis/project-accounting-extension.md](analysis/project-accounting-extension.md) | 项目型企业扩展方案:在动态模型之上增加项目/合同/阶段/发票维度,不破坏现有账套 |
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

## 七、运维与数据位置

### 7.1 本地数据库目录

ForgeFin 把系统库与各公司库放在操作系统的"应用数据目录"下的 `ForgeFin/` 子目录中,具体位置由 `dirs` crate 根据平台自动决定。源代码定义在 `src-tauri/src/db/mod.rs:20-26`。

| 平台 | 数据根目录 |
|------|------------|
| **macOS** | `~/Library/Application Support/ForgeFin/` |
| Windows | `%APPDATA%/ForgeFin/`(通常 `C:\Users\<user>\AppData\Roaming\ForgeFin\`) |
| Linux | `~/.local/share/ForgeFin/` |

子目录布局:

```
<数据根目录>/
├── forgefin_system.db          ← 系统库:用户、公司、用户-公司权限
├── forgefin_system.db-wal      ← SQLite WAL 日志(成对存在)
├── companies/
│   ├── forgefin_company_{uuid}.db        ← 每个公司/账套一个独立库
│   └── forgefin_company_{uuid}.db-wal
└── backups/                    ← 系统/公司备份目录(参见 `backup.rs`)
```

### 7.2 当前开发机数据库清单

> 本节随机器变化,仅作查找指引。实际内容以本机文件为准。

```bash
# macOS 开发机
ls -la "$HOME/Library/Application Support/ForgeFin/"
ls -la "$HOME/Library/Application Support/ForgeFin/companies/"
```

打开公司库可使用任意 SQLite 客户端(命令行 `sqlite3`、DB Browser for SQLite、DBeaver、Navicat 等)。WAL 模式下客户端需支持 WAL(`sqlite3` CLI 默认支持)。

### 7.3 常用 SQL 排查语句

```sql
-- 系统库:列出所有公司
SELECT id, name, tax_id, is_active, created_at FROM companies ORDER BY created_at;

-- 公司库:统计各来源类型记录数
SELECT st.code AS source_type, COUNT(*) AS n
FROM source_records sr JOIN source_types st ON sr.source_type_id = st.id
GROUP BY st.code;

-- 公司库:查看最近一次导入
SELECT id, file_name, source_type, row_count, imported_at
FROM import_batches ORDER BY imported_at DESC LIMIT 20;

-- 公司库:余额连续性失败行(后端 balance_check_status = mismatch)
SELECT id, source_file_name, source_row_no, record_date, balance
FROM source_records
WHERE balance IS NOT NULL
ORDER BY record_date;

-- 公司库:用户列显示偏好
SELECT * FROM ui_column_prefs;
```

### 7.4 备份与恢复

- **应用内备份**:「账套管理 → 备份」使用 `backup_company_cmd` / `backup_system_cmd`,文件落在 `<数据根目录>/backups/`
- **手工备份**:`sqlite3 forgefin_company_{uuid}.db ".backup '/path/to/backup.db'"`(WAL 模式下必须用 `.backup` 命令,直接 `cp` 可能丢未刷盘的 WAL)
- **备份文件命名**:`<公司名>_<时间戳>.db`

### 7.5 清除/重置

```bash
# 重置整个应用数据(慎用!)
rm -rf "$HOME/Library/Application Support/ForgeFin/"
```

应用下次启动时会自动重建 `data_dir` 并执行 `init_system` / `init_company` 建表。

### 7.6 常见问题

| 现象 | 原因 | 处理 |
|------|------|------|
| 启动报"无法定位应用数据目录" | 系统权限或磁盘满 | 检查 `~/Library/Application Support/` 写权限 |
| 公司库频繁出现 `-wal` 数十 MB | 长事务未提交 | 应用层调用应短小;不要在交互式终端内 keep 连接 |
| 备份后用 `cp` 恢复,导入时报 SQLITE_CORRUPT | WAL 数据未刷盘 | 用 `sqlite3 ... .backup` 代替 `cp` |
| 卸载应用后数据库残留 | macOS 卸载不删 `~/Library/Application Support/` | 手动 `rm -rf` 或用 AppCleaner |

---

## 使用方式

1. 按上方主题分类快速定位所需文档。
2. 点击对应相对路径即可打开。
3. 新增文档后请同步更新本索引。
