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

## Theme System Rules

All visual styling MUST use semantic design tokens from `src/forgefin-theme/styles/`. Never use raw Tailwind color classes directly.

**Semantic tokens (always prefer these):**

| Intent | Token Class | Examples |
|--------|-------------|----------|
| Text | `text-primary`, `text-secondary`, `text-tertiary`, `text-disabled` | Titles, descriptions, metadata |
| Text (status) | `text-success`, `text-warning`, `text-danger`, `text-info`, `text-brand` | Status indicators, links |
| Background | `bg-surface`, `bg-surface-alt`, `bg-surface-hover`, `bg-brand`, `bg-border`, `bg-main` | Cards, page backgrounds, hover |
| Border | `border-border`, `border-border-light`, `border-surface`, `border-brand` | Card outlines, separators |
| Font size | `text-12`, `text-13` | Small labels, helper text |
| Numbers | `text-num`, `text-money` | Monetary values, tabular figures |

**Layout classes from theme (prefer over raw Tailwind):**
- `.page-grid` — two-column responsive grid (use instead of `grid grid-cols-2 gap-4`)
- `.page-header` — header with title + actions
- `.page-content` — flex column container
- `.card`, `.card-header`, `.card-title`, `.card-body`, `.card-footer` — card containers
- `.data-table`, `.data-table-num`, `.pagination` — table layout
- `.btn`, `.btn-primary`, `.btn-outline`, `.btn-sm` — buttons
- `.form-field`, `.form-label`, `.form-input` — form controls
- `.tag-brand`, `.tag-success`, `.tag-draft`, `.tag-pending`, `.tag-posted` — status tags
- `.empty-state`, `.empty-state-title`, `.empty-state-desc` — empty placeholders

**Forbidden:**
```html
<!-- Never use raw color classes: -->
bg-white bg-slate-50 bg-gray-100 text-gray-900 text-slate-700 border-gray-200

<!-- Always use semantic tokens: -->
bg-surface bg-surface-alt text-primary text-secondary border-border
```

Tailwind layout classes (`flex`, `gap-*`, `p-*`, `m-*`, `items-center`, etc.) are acceptable for spacing/layout since `--spacing: 1px` is configured in the theme. Colors, backgrounds, and borders must always use the semantic token classes above.

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


