# ForgeFin Agent Operating Rules

These rules apply throughout the entire project lifecycle.

All agents must follow these rules before generating code, modifying files, or creating new features.

---

# Project Overview

ForgeFin is a desktop-first financial management system.

Technology Stack:

* Rust Stable
* Tauri 2.x
* Leptos 0.8
* Tailwind CSS v4
* SQLite

Target Users:

* Accountants
* Financial Managers
* Auditors
* CFOs

The application is a professional financial tool, not a marketing website.

---

# Core Principles

## Consistency Over Creativity

Follow existing project patterns.

Do not introduce new architectures, libraries, or coding styles unless explicitly requested.

Consistency is more important than personal preference.

---

## Incremental Development

Prefer small, focused changes.

Avoid large-scale rewrites.

Avoid touching unrelated files.

Each task should produce a working result.

---

## Simplicity First

Follow:

* KISS
* DRY
* YAGNI

Do not build future features.

Implement only what is required.

---

# Directory Rules

## Pages

All pages MUST be stored under:

```text
src/pages/
```

Examples:

```text
src/pages/dashboard.rs
src/pages/voucher.rs
src/pages/general_ledger.rs
```

Never place page implementations inside:

```text
src/components/
```

---

## Components

Reusable UI components MUST be stored under:

```text
src/components/
```

Examples:

```text
src/components/table/
src/components/form/
src/components/layout/
src/components/charts/
```

Components should be reusable and independent.

---

## Feature Organization

Prefer feature-oriented organization.

Example:

```text
src/pages/voucher.rs

src/components/voucher/
    voucher_table.rs
    voucher_form.rs
    voucher_toolbar.rs
```

Avoid creating large monolithic files.

---

# UI Design Rules

Use the ForgeFin Enterprise Design System.

Design References:

* SAP Fiori
* Oracle Financials
* Kingdee Cloud
* Ant Design Pro

---

## Required

* Desktop-first
* Information dense
* Professional appearance
* Sidebar navigation
* Top toolbar
* Data-first layout
* Compact forms
* Dense tables

---

## Forbidden

Do NOT generate:

* Landing pages
* Marketing pages
* Hero sections
* Pricing cards
* Testimonials
* Glassmorphism
* Gradient backgrounds
* Startup SaaS layouts

---

## Visual Rules

Colors:

* Neutral
* Slate
* Zinc

Radius:

```text
rounded-md
```

Only.

Shadows:

```text
shadow-sm
```

Only.

Spacing:

```text
8px grid system
```

---

# Leptos Rules

Prefer:

```rust
#[component]
```

small focused components.

Avoid components exceeding 300 lines whenever practical.

Split large UI into smaller components.

---

# Rust Rules

Prefer:

* Strong typing
* Explicit structures
* Clear naming

Avoid:

* unwrap()
* expect()

unless failure is impossible.

Handle errors gracefully.

---

# Dependency Rules

Before introducing a new dependency:

Ask:

1. Can existing code solve this?
2. Can std solve this?
3. Is the dependency maintained?

Do not add dependencies unnecessarily.

---

# Completion Criteria

A task is NOT complete until:

```bash
cargo check
```

passes successfully.

Mandatory validation:

```bash
cargo fmt
cargo check
```

Both must succeed.

---

# Definition of Done

A task is complete only when:

* Feature implemented
* Code compiles
* No obvious warnings
* UI follows ForgeFin design rules
* Files placed in correct directories
* Existing functionality remains intact

If any item fails, the task is not complete.

---

# Agent Behavior

Before coding:

1. Understand existing patterns.
2. Follow existing structure.
3. Minimize changes.

While coding:

1. Create focused components.
2. Keep code readable.
3. Avoid unnecessary abstraction.

After coding:

1. Run formatting.
2. Run cargo check.
3. Verify directory rules.
4. Verify design rules.

Only then consider the task complete.
