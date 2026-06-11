---
name: release
description: Release workflow
---

cargo test

cargo clippy

cargo build --release

cargo tauri build

Semantic Versioning
