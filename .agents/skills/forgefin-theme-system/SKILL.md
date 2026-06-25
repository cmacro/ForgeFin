---
name: forgefin-theme-system
description: Use when creating or modifying themes, colors, design tokens, dark mode, density modes, semantic color mappings, or component styling. Ensures all UI remains theme-compatible and uses semantic design tokens instead of hardcoded colors.
---

# Semantic Token Rules

## Purpose

All visual styling must use semantic design tokens.

Components should describe intent, not color.

Bad:

```html
bg-white
bg-slate-50
text-gray-900
border-gray-200
```

Good:

```html
bg-surface
bg-surface-alt
text-primary
border-main
```

Components must never depend on specific colors.

Themes control colors.

Components consume semantics.

---

## Core Principle

Design Flow:

Business Intent
    ↓
Semantic Token
    ↓
Theme Mapping
    ↓
Actual Color

Example:

Card
    ↓
surface
    ↓
Classic Theme
    ↓
#ffffff

Card
    ↓
surface
    ↓
Dark Theme
    ↓
#1e293b

---

# Surface Tokens

## surface

Primary container background.

Examples:

- Card
- Dialog
- Drawer
- Table

Usage:

```html
bg-surface
```

---

## surface-alt

Secondary workspace background.

Examples:

- Page background
- Nested containers

Usage:

```html
bg-surface-alt
```

---

## surface-hover

Hover state background.

Usage:

```html
hover:bg-surface-hover
```

---

## surface-active

Selected state background.

Usage:

```html
bg-surface-active
```

---

# Text Tokens

## primary

Main readable text.

Usage:

```html
text-primary
```

Examples:

- Titles
- Table data
- Form labels

---

## secondary

Supporting information.

Usage:

```html
text-secondary
```

Examples:

- Descriptions
- Metadata
- Helper text

---

## disabled

Disabled content.

Usage:

```html
text-disabled
```

---

# Border Tokens

## main

Primary border.

Usage:

```html
border-main
```

Examples:

- Cards
- Tables
- Inputs

---

## muted

Low-emphasis separators.

Usage:

```html
border-muted
```

---

# Brand Tokens

## brand

Primary application color.

Usage:

```html
bg-brand
text-brand
border-brand
```

Examples:

- Primary buttons
- Active menu items
- Links

---

## brand-hover

Hover state.

Usage:

```html
hover:bg-brand-hover
```

---

# Status Tokens

## success

Successful state.

Usage:

```html
bg-success
text-success
border-success
```

Examples:

- Posted vouchers
- Completed approvals

---

## warning

Attention required.

Usage:

```html
bg-warning
text-warning
border-warning
```

Examples:

- Pending review
- Near due invoices

---

## danger

Error or critical state.

Usage:

```html
bg-danger
text-danger
border-danger
```

Examples:

- Failed transactions
- Overdue accounts

---

## info

Informational state.

Usage:

```html
bg-info
text-info
border-info
```

---

# Navigation Tokens

## sidebar

Sidebar background.

Usage:

```html
bg-sidebar
```

---

## sidebar-active

Active menu item.

Usage:

```html
bg-sidebar-active
```

---

## sidebar-hover

Hover menu item.

Usage:

```html
hover:bg-sidebar-hover
```

---

# Financial Workflow Tokens

## draft

Draft documents.

Usage:

```html
bg-draft
text-draft
```

---

## pending

Waiting for approval.

Usage:

```html
bg-pending
text-pending
```

---

## approved

Approved documents.

Usage:

```html
bg-approved
text-approved
```

---

## posted

Posted accounting entries.

Usage:

```html
bg-posted
text-posted
```

---

## archived

Archived records.

Usage:

```html
bg-archived
text-archived
```

---

# Theme Compatibility

Every token must exist in every theme.

Required themes:

- Classic
- Dark
- Blue
- High Contrast

Example:

Classic:

```css
--color-surface: #ffffff;
```

Dark:

```css
--color-surface: #1e293b;
```

Blue:

```css
--color-surface: #f8fafc;
```

Component code never changes.

Only theme definitions change.

---

# Forbidden

Never use business colors directly:

```html
bg-white
bg-black
bg-gray-100

text-gray-900
text-slate-700

border-gray-200
border-slate-300
```

Never hardcode theme colors inside components.

All visual styling must use semantic tokens.
