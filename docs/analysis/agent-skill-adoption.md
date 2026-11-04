# Agent Skill 采纳评估（mattpocock/skills）

> 日期：2026-11-04 · 结论：只采纳 6 个「即可用」+ 4 个「需适配」，其余不进主链。
> 本文件只记录决策，不改动 `src/`、`Cargo.toml`、`Makefile`。覆盖边界的理由见
> `docs/adr/0001-automated-coverage-boundary.md`，术语见 `CONTEXT.md`（协作语言一节）。

## 1. 上游现状

`github.com/mattpocock/skills`（main）按「谁能调用」分两轴，分四个区：

- `skills/engineering`（代码日常）：`ask-matt`、`grill-with-docs`、`triage`、
  `improve-codebase-architecture`、`setup-matt-pocock-skills`、`to-spec`、
  `to-tickets`、`implement`、`implement-spec`、`wayfinder`、`retro`；
  模型级：`prototype`、`diagnosing-bugs`、`research`、`tdd`、`domain-modeling`、
  `codebase-design`、`code-review`、`pr`、`wizard`
- `skills/productivity`：`grill-me`、`handoff`、`teach`、`to-questionnaire`、
  `wait-what`；模型级：`grilling`、`writing-for-agents`
- `skills/in-progress`（不成熟）：`chief-of-staff`、`claude-handoff`、`loop-me`、
  `software-factory`、`setup-ts-deep-modules`、`writing-*`
- `skills/deprecated`、`skills/misc`：`git-guardrails-claude-code`、
  `setup-pre-commit`、`scaffold-exercises`、`migrate-to-shoehorn`

## 2. 本仓库现状

`/home/dev/fin/ForgeFin/.agents/skills/` 共 19 个：

- 来自该库 8 个：`code-review`、`codebase-design`、`diagnosing-bugs`、
  `domain-modeling`、`grill-with-docs`、`grilling`、`improve-codebase-architecture`、`research`
- ForgeFin 自研 5 个：`forgefin-page-generator`、`forgefin-ui-principles`、
  `forgefin-theme-system`、`forgefin-layout-patterns`、`forgefin-financial-workflows`
- 其他来源 6 个：`leptos-0.8`、`rust-expert`、`tauri-2-architecture`、
  `tailwind-design-system`、`find-skills`、`karpathy-guidelines`

已知问题（本次未处理，仅记录）：

1. **未纳入 lock**：`skills-lock.json` 只跟踪 `tailwind-design-system` 与
   `tauri-v2`，上述 8 个来自该库的 skill 是手工拷贝，无版本锚点。
2. **版本漂移**：本地 `domain-modeling` / `grill-with-docs` 是旧版，写
   `CONTEXT.md`；上游已改为 `GLOSSARY.md`。**本仓库裁决保留 `CONTEXT.md` 命名**，
   因此不追上游改名，只按需吸收内容更新。
3. `TODO.md` 引用的 `docs/业务模块分析.md` 不存在（失效引用）。
4. `CONTEXT.md` 旧有条目带「对应表 / 结构体 / 页面路径」，与「词表不承载实现细节」
   的原则冲突（旧格式遗留，未在本轮重写）。

## 3. 文档链与 skill 归属

缺的不是 skill，而是链路：②④⑤ 三段有文档无执行体，③ 段有任务无规格。

| 段 | 唯一权威载体 | 执行 skill | 档位 | 今天的实际状态 |
|---|---|---|---|---|
| ① 基础业务文档 | `docs/analysis/*.md`、`docs/开发计划.md`、`docs/uidesign/DESIGN.md` | `grill-with-docs`（只提问对账，不改写） | 即可用 | 已有 5 份分析文档 |
| ② 领域语言 | `CONTEXT.md`（保留现名） | `domain-modeling` | 即可用 | 词表覆盖约 2%，代码已到多币种/数据汇总 |
| ④ 验收条件 | `docs/spec/<feature>.md` → 验收条件一节 | `to-spec` | 需适配（无 issue tracker，落本地 md） | 功能级验收**不存在**；`docs/plan/phase1_acceptance.md` 标注为「示例」且被代码片段污染 |
| ③ 开发 | `TODO.md` 复选框 = 票 + 阻塞边 | `to-tickets` → `implement` | 需适配 | 32 个复选框，无规格来源、无阻塞边、单 `main` 无分支隔离 |
| ⑤ 覆盖测试 | `docs/测试方案.md`（策略 + 用例清单，与验收条件对号） | `tdd` | **先行票落地前按「不可用」处理**；之后仅限 `src-tauri` | 方案 164 行已写、零落地：全仓 1 个 `#[cfg(test)]`、无 `[dev-dependencies]`、无 `Makefile: test`、无 CI |
| 收尾 | `docs/changelogs/YYYYMMDD_*.md` | `code-review` 双轴 + `cargo fmt` / `cargo check` | 即可用 | changelog 仅 3 份，明显落后于已交付功能数 |

裁决：`docs/plan/phase*_acceptance.md` 降级为「阶段验收汇总」，功能验收一律下沉
`docs/spec/`；不新建第二个验收标准来源。

## 4. 分档清单

### 即可用（零/低依赖）

| Skill | 在 ForgeFin 的职责 |
|---|---|
| `grill-with-docs` | ① → ②：把业务文档里的模糊词逼成术语 |
| `domain-modeling` | 维护 `CONTEXT.md`，新功能开工前先对齐词表 |
| `code-review` | 双轴复核（Standards / Spec）+ 提交前收尾 |
| `research` | 高信任源调研并落 cited md（ISO 4217、准则映射、第三方接口） |
| `writing-for-agents` | 编写/修改 `.agents/skills/forgefin-*` 与 `AGENTS.md` 的唯一规范 |
| `wait-what` | 表述没落地时让 agent 用词表重讲，纯对话零成本 |

### 需适配（依赖先行票或本地载体）

| Skill | 适配内容 |
|---|---|
| `to-spec` | 输出改落 `docs/spec/<feature>.md`，不写 tracker |
| `to-tickets` | 输出改落 `TODO.md` 复选框并标注阻塞边 |
| `implement` | 入口挂 `cargo check`，出口挂 `code-review`；禁止并发多实现者（单 `main`） |
| `tdd` | 仅 `src-tauri`；需先有 `[dev-dependencies]` 与 `Makefile: test` |

### 按需启用（不进主链）

| Skill | 触发条件 |
|---|---|
| `prototype` | UI 大改、需要并排比较多个变体时 |
| `retro` | 每 1–2 周一次环境复盘（会首先建议补 CI 与 `test` 目标） |
| `handoff` | 单次会话超出上下文，需要交接 |

## 5. 暂不用

| Skill | 硬理由 |
|---|---|
| `setup-matt-pocock-skills` | 它配 issue tracker / 标签 / 文档布局；本仓库裁决 `TODO.md` + `CONTEXT.md` + `docs/spec/`，无 tracker 可配 |
| `triage` | 依赖 tracker 状态机与标签；`TODO.md` 无标签体系 |
| `wayfinder` | 以 tracker 决策票规划超长工作；已由 `docs/plan/phase0-5` 承担 |
| `implement-spec` | 多实现者子代理 + 单集成分支；本仓库只有 `main` |
| `pr` | 无 PR 流程（单 `main` 直推） |
| `wizard` | 面向基础设施 provisioning / 凭据引导；本地桌面 app 无此环节 |
| `ask-matt` | 与本地 `find-skills` 职责重叠 |
| `grill-me` | 与本地 `grilling` 完全重复 |
| `teach`、`to-questionnaire` | 面向教学与外部专家异步问卷；需求由项目负责人裁决 |
| `in-progress/*`、`deprecated/*`、`misc/*` | 上游自标为不成熟 / 已弃用 / 与本项目无关 |

若将来引入 GitHub Issues 或 PR 流程，`triage`、`to-spec`、`to-tickets`、
`implement-spec`、`pr` 的档位需要整体重估。

## 6. 首批先行票（尚未执行）

1. `src-tauri/Cargo.toml` 增加 `[dev-dependencies]`：`rstest`、`tempfile`
2. `Makefile` 增加 `test: cargo test -p forgefin`
3. 以现有 `src-tauri/src/commands/raw.rs` 的 `#[cfg(test)]` 为种子，抽出 `db::test_db()`（内存库）
4. 之后才允许对单个功能走 `to-spec → to-tickets → tdd → code-review`

建议落地顺序：②词表（随功能增量补）→ 先行票 → ④规格 → ③票 → ⑤测试。
