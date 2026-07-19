# ForgeFin Agent Rules

## Project

ForgeFin is a desktop-first financial management application.

**Target Users:**
- Accountants
- Financial Managers
- Auditors
- CFOs

**Technology Stack:**
- Rust Stable
- Tauri 2.x
- Leptos 0.8
- Tailwind CSS v4
- SQLite

ForgeFin is a professional business application.

**It is NOT:**
- a marketing website
- a landing page
- a portfolio website
- a social application

---

## Directory Rules

Pages MUST be stored in:
`src/pages/`

**Examples:**
- `src/pages/dashboard.rs`
- `src/pages/voucher.rs`
- `src/pages/general_ledger.rs`

Reusable UI components MUST be stored in:
`src/components/`

**Examples:**
- `src/components/layout/`
- `src/components/table/`
- `src/components/form/`
- `src/components/charts/`

Do not place reusable components directly inside page files.

---

## Required Skills

The following skills must be applied based on the task type.

| Task Type | Required Skills | Purpose |
|------------|------------|------------|
| New Page Generation | forgefin-page-generator | Generates complete pages following ForgeFin standards |
| UI Modification / Review | forgefin-ui-principles, forgefin-theme-system | Maintain visual consistency, information density, and theme compatibility |
| Layout Design | forgefin-layout-patterns, forgefin-ui-principles | Apply standard ForgeFin page structures and navigation patterns |
| Financial Logic / UI | forgefin-financial-workflows, forgefin-ui-principles | Implement accounting workflows and financial user experiences |
| Theme / Styling | forgefin-theme-system | Manage design tokens, themes, colors, and density modes |

---

## Financial Application Rules

- All monetary values must use precise decimal handling (`rust_decimal` recommended)
- Date handling must be consistent and timezone-aware (`chrono` or `time` crate)
- Financial operations (vouchers, journals, postings) must include proper audit trail considerations
- All amounts should respect configured decimal precision (usually 2 decimals)
- Sensitive actions must include confirmation dialogs or permission checks
- Reports and ledgers must support data export (Excel recommended)
- Numbers, currencies, and percentages must follow consistent formatting across the application

---

## Coding Rules

Follow existing project patterns and Rust/Leptos best practices.

**Prefer:**
- Small, focused, reusable components
- Clear, descriptive naming conventions
- Strong typing and proper error handling
- Server Functions (`#[server]`) for backend logic

**Rust & Leptos Best Practices:**
- Use `Signal` / `Resource` for state management
- Prefer `Result<T, E>` with proper user-facing error messages
- Avoid `unwrap()` / `expect()` in production code
- All forms should use consistent validation patterns
- Financial calculations must use precise decimal types
- Components should be as pure as possible with minimal side effects
- **No ORM** — use `rusqlite` directly with raw SQL. Keep the data layer simple and explicit
- **Tauri 2 command args use camelCase** — `#[tauri::command]` auto-converts Rust snake_case params to camelCase JSON keys. Frontend `invoke` calls must use camelCase keys (e.g. `companyId` not `company_id`)

**Avoid:**
- Large-scale refactoring unless explicitly requested
- Unnecessary new dependencies
- Rewriting unrelated files
- Inline complex business logic in UI components

---

## Completion Criteria

Before considering a task complete:
```sh 
cargo fmt
cargo check
```

Both commands must succeed.

**If `cargo check` fails:**
The task is NOT complete.

---

## Definition of Done

A task is considered complete only when **all** of the following are satisfied:

- Feature is fully implemented and functional
- UI follows ForgeFin design principles and visual standards
- Files are placed in the correct directories
- Code follows the project's naming and architectural patterns
- `cargo fmt` passes
- `cargo check` passes
- No new warnings introduced (where reasonably avoidable)

If any item above fails, the task is **not complete**.

---

## Data Desensitization Rules

When processing raw data files in `docs/raw/`, always apply desensitization rules to protect sensitive information. Desensitization mapping is stored in:

```
docs/raw/脱敏映射表.tsv
```

**IMPORTANT**: This file is for internal development only and must never be committed to public repositories or shared externally.

### Desensitization Rules

| Data Type | Rule | Example |
|-----------|------|---------|
| Person names | Replace with "客户A/B/C" or "员工A/B/C" | 张三 → 客户A |
| Company names | Replace with "XX公司" or "供应商A/B/C" | 上海XX公司 → 供应商A |
| Bank account numbers | Keep first 4 and last 4 digits | 6226220219453718 → 6226****3718 |
| Mobile numbers | Keep first 3 and last 4 digits | 13812345678 → 138****5678 |
| ID numbers | Keep first 6 and last 4 digits | 310101199001011234 → 310101****1234 |
| Addresses | Keep province/city, replace street details | 上海市徐汇区XX路123号 → 上海市XX路*号 |
| Merchant IDs | Replace with "商户编号A/B" | 100172984575 → 商户编号A |

### Enterprise-Specific Rules

**健康管理公司A (yuemy):**
- Replace "悦己健康" / "上海月满悦健康管理有限公司" → "健康管理公司A"
- Replace customer names → "客户A/B/C..."
- Replace employee names → "员工A/B..."
- Replace suppliers → "供应商A/B/C..."

**项目型企业A (zhentai):**
- Replace "正泰" → "项目型企业A"
- Replace location names → "地点A/B/C..." (杭州→地点F, 上海→地点G, etc.)
- Replace project names → Remove company prefix, keep project type

### Workflow

1. Before creating any analysis or documentation from raw data, consult `docs/raw/脱敏映射表.tsv`
2. Apply mapping rules to all personal names, company names, addresses, and account numbers
3. Verify all desensitized content is consistent across the document
4. Never output original sensitive data in analysis files

### File Location

All desensitization mapping files MUST be stored in `docs/raw/` directory, which is excluded from version control (see `.gitignore`) to prevent accidental exposure of sensitive data.
