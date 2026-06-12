# Project Rules

Stack:

- Rust stable
- Tauri 2.x
- Leptos
- SQLite
- tokio

Architecture:

UI
 -> Command
 -> Domain Service
 -> Repository
 -> SQLite

Rules:

1. Never access database from command.
2. Never access database from UI.
3. Repository pattern only.
4. All async code uses tokio.
5. Avoid unwrap().
6. Prefer Result<T>.
7. Use anyhow in application layer.
8. Use thiserror in domain layer.
9. Every feature requires tests.

Supported:

- Ollama
- llama.cpp
- MLX


每次执行开发任务都需要在.harness/logs中保留日志

