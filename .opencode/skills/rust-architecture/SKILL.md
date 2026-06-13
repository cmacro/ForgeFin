---
name: rust-architecture
description: Rust architecture generation
---

# Layer Rules

app
domain
infra
shared

# Domain

Contains:

Entity
Repository Trait
Domain Service

# Infra

Contains:

Database
Filesystem
Cache
LLM

# Rules

No business logic in infra.
No database access in app.
Use dependency injection.

