---
name: tauri-2-architecture
description: Tauri 2 application architecture guidelines separating UI, commands, services, repositories, and persistence layers.
---

# Tauri 2 Architecture Skill

## Recommended Layers

```
Leptos UI
    |
Tauri Commands
    |
Application Services
    |
Repositories
    |
SQLite / Database
```

## Frontend Rules

Organize:

```
pages/
components/
layouts/
hooks/
```

Components should not directly access database APIs.

## Backend Rules

Organize:

```
commands/
services/
repositories/
models/
dto/
errors/
```

Commands should only:

- validate input
- call services
- convert errors

## Database Rules

Repositories own SQL.

Avoid:

```rust
#[tauri::command]
fn create_x() {
    sql.execute(...)
}
```

Prefer:

```
Command
  |
Service
  |
Repository
  |
Database
```

## Error Handling

Use application error types.

Avoid exposing raw database errors directly to UI.

## Financial Software Rules

For accounting operations:

- use transactions
- never update ledger partially
- keep audit records
- separate business rules from storage
