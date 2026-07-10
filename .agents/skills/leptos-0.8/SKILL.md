---
name: leptos-0.8
description: Leptos 0.8 development guidelines for components, signals, callbacks, async rendering, and view composition.
---

# Leptos 0.8 Skill

## Component Events

Prefer:

```rust
Callback<T>
```

instead of:

```rust
impl Fn()
Arc<dyn Fn()>
```

Example:

```rust
#[component]
fn Modal(
    on_close: Callback<()>
)
```

Call:

```rust
on_close.run(());
```

## Signals

Prefer:

```rust
let (value, set_value) = signal(initial);
```

Avoid deprecated patterns.

## Async UI

Recommended structure:

```
Suspense
  |
  Suspend / Resource
  |
  async operation
```

## View Rules

Avoid ambiguous multi-root views inside complex expressions.

Prefer:

```rust
view! {
    <div>
        <Table />
        <Show>
            ...
        </Show>
    </div>
}
```

especially inside:

- match expressions
- async blocks
- Suspense
- IntoView conversions

## Common Errors

### E0225

Cause:

```rust
dyn Fn() + Clone
```

Solution:

Use `Callback`.

### FnOnce closure

Check moved variables inside:

```rust
move || {}
```

Clone required values before async blocks.
