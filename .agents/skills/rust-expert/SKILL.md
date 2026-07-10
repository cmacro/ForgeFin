---
name: rust-expert
description: Rust engineering best practices for ownership, async, traits, errors, and production-quality code. Use when writing, reviewing, or debugging Rust code.
---

# Rust Expert Skill

## Core Rules

- Prefer concrete types and generics over unnecessary trait objects.
- Avoid `Arc<dyn Fn() + Clone>` patterns.
- Use explicit ownership boundaries around async tasks.
- Clone values before `async move` blocks when ownership is required.
- Prefer clear error types over string-based errors.

## Trait Object Rules

Avoid:

```rust
dyn Fn() + Clone
```

because multiple non-auto traits cannot be combined.

Prefer:

- Generic parameters:
  `F: Fn() + Clone`
- Framework callbacks:
  `Callback<T>`
- Wrapper traits when dynamic dispatch is truly required.

## Async Rules

Before:

```rust
spawn_local(async move {
    use(value);
});
```

Check:

- Is `value` moved correctly?
- Does the future need `'static`?
- Are clones created before the async boundary?

## Review Checklist

- cargo check passes
- cargo clippy passes
- ownership is explicit
- no unnecessary clones
- Send/Sync requirements are correct
- errors are handled
