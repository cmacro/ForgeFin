---
name: forgefin-ui-principles
description: Use when creating, modifying, or reviewing any user interface for ForgeFin. Applies to pages, layouts, forms, tables, dialogs, dashboards, reports, and navigation.
---

# ForgeFin UI Principles

## Purpose

ForgeFin is a professional financial management application.

Target users:
- Accountants
- Finance Managers
- Auditors
- CFOs

Every UI decision must prioritize:
- Efficiency
- Readability
- Information density
- Workflow productivity

---

# Design Philosophy

## Data First

Users spend most of their time:
- entering data
- reviewing records
- searching transactions
- generating reports

Therefore:
- tables are primary
- forms are primary
- dashboards support workflows

Avoid decorative content.

---

## Consistency Over Creativity

Reuse existing patterns.

Users should immediately understand:
- navigation
- actions
- data locations

without learning a new interface.

---

# Layout Rules

## Application Shell

Preferred layout:

```text
┌──────────────────────────────┐
│ Header                       │
├───────┬──────────────────────┤
│       │                      │
│ Menu  │   Workspace          │
│       │                      │
└───────┴──────────────────────┘
```

## Sidebar

Required for primary navigation.

Width: 240px
Collapsed: 72px

Characteristics:
- icon + label
- collapsible
- multi-level navigation

## Header

Height: 56px

Contains:
- breadcrumb
- search
- notifications
- user actions

## Workspace

Background: slate-50
Purpose:
- display data
- perform work

Never use workspace area for marketing content.

---

# Component Principles

## Tables

Most important component.

Requirements:
- sticky header
- sorting
- filtering
- pagination
- row selection

Preferred row height: 40px

## Forms

Forms should be compact.

Preferred input height: 32px

Prefer:
- grouped fields
- clear labels
- logical sections

Avoid excessively long forms.

## Dialogs

Use dialogs for:
- confirmations
- small edits

Use drawers or pages for:
- complex workflows
- large forms

## Buttons

Recommended sizes:
- Small: 28px
- Default: 32px
- Large: 40px

Primary actions should be obvious.
Avoid oversized buttons.

## Dashboard Principles

Every dashboard widget should answer:
"What action does this help the user take?"

Good examples:
- Cash Balance
- Accounts Receivable
- Accounts Payable
- Monthly Revenue
- Approval Queue
- Financial Alerts

Avoid meaningless charts.

---

# Visual Rules

## Colors

Primary palette:
- Slate
- Zinc
- Neutral
- Blue

Avoid:
- neon colors
- rainbow palettes
- excessive saturation

## Radius

Use:
- rounded-md

Avoid:
- rounded-2xl
- rounded-3xl
- rounded-full
for application UI.

## Shadows

Use:
- shadow-sm

Borders should provide most separation.

## Spacing

Use an 8px spacing system.

Examples:
- 8
- 16
- 24
- 32
- 40
- 48

Avoid random spacing values.

---

# Theme Compatibility

All UI must support future theme switching.

Never hardcode business colors directly into components.

Prefer design tokens for:
- background
- text
- border
- primary color

Future themes:
- Classic
- Dark
- Blue
- High Contrast

Future density modes:
- Compact
- Comfortable
- Large

---

# Forbidden Patterns

Never generate:
- landing pages
- hero sections
- pricing cards
- testimonials
- startup SaaS layouts
- glassmorphism
- heavy gradients
- oversized illustrations
- decorative animations

---

# Accessibility

Always consider:
- keyboard navigation
- focus states
- readable contrast
- predictable interactions

---

# Review Checklist

Before finishing a UI task:
- Optimized for financial work?
- Information easy to scan?
- Tables prioritized?
- Forms prioritized?
- Consistent with existing pages?
- Professional ERP appearance?

If not, revise the design.
