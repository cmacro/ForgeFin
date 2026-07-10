# ForgeFin 编译错误修复指南（Agent 使用）

## 快速分类索引

| 错误类别 | 错误编号 | 快速定位 |
|---------|---------|---------|
| Leptos API 变更 | #5, #6, #11, #22 | `Suspend::with` → `Suspend::new`，`For` 组件语法 |
| Resource 使用错误 | #4, #8, #19, #28 | `.get().await` → `.await`，类型推断 |
| 所有权/借用 | #14, #20, #26, #27, #30 | 闭包中移动后再次使用 |
| 类型不匹配 | #12, #15, #18, #21 | `Option`/`String`/`Result` 比较 |
| 线程安全 | #16, #24, #29 | `Send`/`Sync` 约束缺失 |
| 信号/闭包 | #3, #13, #25 | 普通闭包 vs `Memo`/`Signal` |
| 事件回调 | #16, #24, #31 | `impl Fn + Clone + Send + Sync` → `Callback` |
| 其他 | #1, #2, #7, #9, #10, #17, #23 | 杂项 |

---

## 1. `web_sys::window()` 未导入

**文件**: `src/pages/voucher_entry.rs`, `src/pages/voucher.rs`

**错误**: `cannot find function web_sys::window`

**根因**: Leptos 0.8 已通过 `leptos::prelude::*` 导出 `window()`，无需 `web_sys` 依赖。

**修复**:
```rust
// 修改前
if let Some(w) = web_sys::window() { let _ = w.print(); }

// 修改后
let _ = window().print();
```

---

## 2. `RwSignal::new` 不能在 `static` 中调用

**文件**: `src/auth.rs`

**错误**: `E0015` — `RwSignal::new` is not const

**根因**: Rust `static` 初始化必须是常量表达式，`RwSignal::new` 是运行时函数。

**修复**: 使用 `LazyLock` 延迟初始化。
```rust
use std::sync::LazyLock;

// 修改前
static SESSION: RwSignal<Option<UserInfo>> = RwSignal::new(None);

// 修改后
static SESSION: LazyLock<RwSignal<Option<UserInfo>>> = LazyLock::new(|| RwSignal::new(None));
```

---

## 3. 普通闭包调用了 `.get()` 方法

**文件**: `src/pages/voucher_entry.rs`

**错误**: `method get` exists for type `...` but not for closure

**根因**: 普通闭包没有 `.get()` 方法，只有 `Signal`/`Memo` 才有。

**修复**: 将闭包改为 `Memo::new`。
```rust
// 修改前
let debit_total = move || { entries.get().iter().map(...).sum() };

// 修改后
let debit_total = Memo::new(move |_| { entries.get().iter().map(...).sum() });
```

---

## 4. `Resource::get().await` 错误

**文件**: 多处（`voucher.rs`, `settings.rs`, `accounts.rs`, `contacts.rs`, `voucher_entry.rs`）

**错误**: `Resource::get()` 返回 `Option<T>`，不是 Future

**根因**: Leptos 0.8 中 `Resource` 实现了 `IntoFuture`，可直接 `.await`。

**修复**:
```rust
// 修改前
match resource.get().await {
    Some(Ok(x)) => { ... }
    Some(Err(e)) => { ... }
    None => { ... }
}

// 修改后
match resource.await {
    Ok(x) => { ... }
    Err(e) => { ... }
}
```

---

## 5. `Suspend::with` 不存在

**文件**: 多处（`voucher.rs`, `settings.rs`, `accounts.rs`, `contacts.rs`）

**错误**: `Suspend::with` 不存在

**根因**: Leptos 0.8 移除了 `Suspend::with`，改用 `Suspend::new`。

**修复**:
```rust
// 修改前
{move || Suspend::with(async move { ... })}

// 修改后
{move || Suspend::new(async move { ... })}
```

---

## 6. `For` 组件 `each` 属性解析错误

**文件**: `src/pages/voucher_entry.rs`

**错误**: `no method named r#move`

**根因**: `view!` 宏在处理 `For` 的 `each` 属性时，复杂闭包体导致 `move` 被解析为方法名。

**修复**: 创建 `Memo` 存储枚举列表，使用 `let:item`。
```rust
// 新增 Memo
let enumerated_entries = Memo::new(move |_| {
    entries.get().iter().enumerate().map(|(i, e)| (i, e.clone())).collect::<Vec<_>>()
});

// 修改 For
<For each=move || enumerated_entries.get() key=|item| item.1.uid.clone() let:item>
    <td>{item.0 + 1}</td>
</For>
```

---

## 7. `Memo::new` 要求 `PartialEq`

**文件**: `src/pages/voucher_entry.rs`

**错误**: `PartialEq` is not implemented

**根因**: `Memo` 需要比较新旧值，要求 `T: PartialEq`。

**修复**: 为结构体添加 `#[derive(PartialEq)]`。
```rust
#[derive(Clone, Debug, PartialEq)]
struct EntryRow { ... }
```

---

## 8. `detail` 视图中 `d` 被推断为 `String`

**文件**: `src/pages/voucher.rs`

**错误**: `no field voucher on String`

**根因**: `Suspend::new` 的 `async` 块中 `detail.await` 类型被错误推断。

**修复**: 改用 `detail.get().flatten()` 同步获取。
```rust
// 修改前
{move || Suspend::new(async move {
    match detail.await { Some(d) => view! { ... }, None => view! { ... } }
})}

// 修改后
{move || {
    detail.get().flatten().map(|d| {
        view! { ... }
    }).unwrap_or_else(|| {
        view! { <div class="empty-state">...</div> }
    })
}}
```

---

## 9. `on_delete` / `on_audit` 未定义

**文件**: `src/pages/voucher.rs`

**错误**: `cannot find function on_delete`

**根因**: 子组件 props 名称 `on_delete`/`on_audit` 在父组件中不可用，父组件定义的是 `do_delete`/`do_audit`。

**修复**: 替换为本地闭包名。
```rust
// 修改前
on:click=move |_| on_delete(vid.clone())

// 修改后
on:click=move |_| do_delete(vid.clone())
```

---

## 10. `VoucherFilter` 缺少 `PartialEq`

**文件**: `src/ipc.rs`

**错误**: `Resource::new` 要求 `S: PartialEq`

**根因**: `Resource` 需要比较源值是否变化。

**修复**: 添加 `PartialEq` derive。
```rust
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct VoucherFilter { ... }
```

---

## 11. `For` 组件 `let:(idx, row)` 解构冲突

**文件**: `src/pages/voucher.rs`, `src/pages/contacts.rs`

**错误**: `closure is expected to take a single 2-tuple as argument, but it takes 2 distinct arguments`

**根因**: Leptos 0.8 的 `For` 组件不支持元组解构。

**修复**: 使用 `let:item` 手动访问。
```rust
// 修改前
<For each=move || rows.clone() key=|r| r.id.clone() let:(idx, row)>

// 修改后
<For each=move || rows.clone().into_iter().enumerate().collect::<Vec<_>>() key=|item| item.1.id.clone() let:item>
    <td>{item.0 + 1}</td>
</For>
```

---

## 12. `Modal` 组件 `size` 属性类型不匹配

**文件**: `src/pages/settings.rs`, `src/pages/contacts.rs`

**错误**: `expected Option<&str>, found &str`

**根因**: `size` 属性定义为 `Option<&str>`。

**修复**: 包装为 `Some(...)`。
```rust
// 修改前
<Modal open=open title="账套" size="lg" on_close=close_rc>

// 修改后
<Modal open=open title="账套" size=Some("lg") on_close=close_rc>
```

---

## 13. `has_company` 布尔值调用 `.get()`

**文件**: `src/app.rs`

**错误**: `the method get exists for type bool`

**根因**: `bool` 不是信号，不能调用 `.get()`。

**修复**: 移除 `.get()`。
```rust
// 修改前
if !has_company.get() { ... }

// 修改后
if !has_company { ... }
```

---

## 14. `res.companies` 所有权移动后再次访问

**文件**: `src/auth.rs`

**错误**: `borrow of moved value: res.companies`

**根因**: `AVAILABLE.set(res.companies)` 移动了所有权。

**修复**: 先克隆。
```rust
// 修改前
AVAILABLE.set(res.companies);
if let Some(first) = res.companies.first() { ... }

// 修改后
let companies = res.companies.clone();
AVAILABLE.set(res.companies);
if let Some(first) = companies.first() { ... }
```

---

## 15. `company_switcher` 中 `String` 与 `Option<String>` 比较

**文件**: `src/components/layout/company_switcher.rs`

**错误**: `can't compare String with Option<String>`

**根因**: `c.id` 是 `String`，`cid` 是 `Option<String>`。

**修复**: 统一类型。
```rust
// 修改前
.find(|c| c.id == cid)

// 修改后
.find(|c| Some(c.id.clone()) == cid)
```

---

## 16. `Modal` 事件闭包 `FnOnce` / `Send` / `Sync` 错误

**文件**: `src/components/layout/modal.rs`

**错误**: `FnOnce` / `Send` / `Sync` 未满足

**根因**: Leptos 要求事件闭包实现 `Fn + Send + Sync + 'static`，`Rc<dyn Fn()>` 不满足。

**修复**: 添加约束，使用 `clone`。
```rust
// 修改前
pub fn Modal(open: ReadSignal<bool>, on_close: impl Fn() + 'static, ...) -> impl IntoView {
    <div class="modal-overlay" on:click=move |_| on_close()>
}

// 修改后
pub fn Modal(open: ReadSignal<bool>, on_close: impl Fn() + Clone + Send + Sync + 'static, ...) -> impl IntoView {
    let close = on_close.clone();
    <div class="modal-overlay" on:click=move |_| close()>
}
```

---

## 17. `ipc.rs` 中 `invoke` 参数类型不匹配

**文件**: `src/ipc.rs`

**错误**: 参数类型不匹配

**根因**: `invoke` 期望 `&String`，但传入了结构体引用。

**修复**: 序列化为 JSON 字符串。
```rust
// 修改前
invoke("update_company_cmd", &[("id", &id), ("input", input)])

// 修改后
invoke("update_company_cmd", &[("id", &id), ("input", &serde_json::to_string(&input).unwrap())])
```

---

## 18. `match` 分支返回类型不兼容

**文件**: `src/pages/settings.rs`, `src/pages/voucher.rs`, `src/pages/accounts.rs`

**错误**: `match` arms have incompatible types

**根因**: `view!` 宏为不同结构生成不同 `View` 类型。

**修复**: 使用 `.into_any()` 统一类型。
```rust
// 修改前
Ok(list) => view! { <table>...</table> },
Err(e) => view! { <div class="login-error">...</div> },

// 修改后
Ok(list) => view! { <table>...</table> }.into_any(),
Err(e) => view! { <div class="login-error">...</div> }.into_any(),
```

---

## 19. `accounts.get().unwrap_or_default()` 返回 `()`

**文件**: `src/pages/accounts.rs`

**错误**: `expected (), found Result`

**根因**: `Resource::get()` 返回 `Option<Result<...>>`，`unwrap_or_default()` 返回 `Result`。

**修复**: 使用 `accounts.await`。
```rust
// 修改前
match accounts.get().unwrap_or_default() { Ok(list) => list, Err(_) => Vec::new() }

// 修改后
accounts.await.unwrap_or_default()
```

---

## 20. `row.id` 在闭包中移动后再次借用

**文件**: `src/pages/voucher.rs`

**错误**: `borrow of moved value: row.id`

**根因**: `move` 闭包捕获了 `row.id` 所有权。

**修复**: 在闭包前克隆。
```rust
// 修改前
let is_active = move || selected.get() == Some(row.id.clone());
let id = row.id.clone();

// 修改后
let id = row.id.clone();
let is_active = move || selected.get() == Some(id.clone());
```

---

## 21. `SummaryStats` 组件 `vouchers` 属性类型不匹配

**文件**: `src/pages/voucher.rs`

**错误**: `expected Resource<(VoucherFilter, i32), ...>, found Resource<Result<VoucherPage, String>>`

**根因**: `Resource` 泛型参数不匹配。

**修复**: 更新 `SummaryStats` 属性类型。
```rust
// 修改前
fn SummaryStats(vouchers: Resource<(VoucherFilter, i32), Result<VoucherPage, String>>)

// 修改后
fn SummaryStats(vouchers: Resource<Result<VoucherPage, String>>)
```

---

## 22. `contacts.rs` 中 `For` 的 `let:(idx, c)` 解构冲突

**文件**: `src/pages/contacts.rs`

**错误**: 同 #11

**根因**: 同 #11

**修复**: 同 #11

---

## 23. `settings.rs` 中 `on_edit` 未定义

**文件**: `src/pages/settings.rs`

**错误**: `cannot find function on_edit`

**根因**: 变量名应为 `open_edit`。

**修复**: 改名。
```rust
// 修改前
on:click=move |_| on_edit(c.clone())

// 修改后
on:click=move |_| open_edit(c.clone())
```

---

## 24. `accounts.rs` 中 `For` 的 `children` 闭包缺少 `Clone` / `Send` / `Sync`

**文件**: `src/pages/accounts.rs`

**错误**: `Fn` trait bound not satisfied

**根因**: Leptos 0.8 的 `For` 组件要求 `children` 闭包实现 `Fn(T) -> N + Send + Clone + 'static`。

**修复**: 添加约束。
```rust
// 修改前
on_edit: impl Fn(Account) + 'static,
on_delete: impl Fn(String) + 'static,

// 修改后
on_edit: impl Fn(Account) + Clone + Send + Sync + 'static,
on_delete: impl Fn(String) + Clone + Send + Sync + 'static,
```

---

## 25. `accounts.rs` 中 `For` 的 `each` 闭包为 `FnOnce`

**文件**: `src/pages/accounts.rs`

**错误**: `closure is FnOnce`

**根因**: `move` 闭包捕获了 `children` 所有权。

**修复**: 使用 `ReadSignal<Vec<TreeNode>>` 或 `StoredValue` 替代直接传递。

---

## 26. `voucher_entry.rs` 中 `FnOnce` 闭包冲突

**文件**: `src/pages/voucher_entry.rs`

**错误**: `closure implements FnOnce, not FnMut`

**根因**: `item` 被 `move` 闭包捕获后，后续 `item.0`/`item.1` 无法再使用。

**修复**: 在闭包前提取 `idx` 和 `row`。
```rust
// 修改前
{move || { Suspend::new(async move { v[item.0].summary = val; }) }}

// 修改后
{let idx = item.0; let row = item.1.clone();
 move || { Suspend::new(async move { v[idx].summary = val; }) }}
```

---

## 27. `voucher_entry.rs` 中 `val` 移动后再次借用

**文件**: `src/pages/voucher_entry.rs`

**错误**: `borrow of moved value: val`

**根因**: `val` 被移动到 `update` 闭包后无法再访问。

**修复**: 提前计算布尔值。
```rust
// 修改前
set_entries.update(|v| { v[i].debit = val; if !val.is_empty() && ... { ... } });

// 修改后
let is_positive = !val.is_empty() && val.parse::<i64>().unwrap_or(0) > 0;
set_entries.update(|v| { v[i].debit = val; if is_positive { ... } });
```

---

## 28. `voucher.rs` 中 `detail` Resource 类型推断错误

**文件**: `src/pages/voucher.rs`

**错误**: `expected Resource<Option<String>, Option<VoucherDetail>>, found Resource<Option<VoucherDetail>>`

**根因**: 单参数 `Option<T>` 源信号导致 Resource 推断为单参数版本。

**修复**: 使用元组源信号。
```rust
// 修改前
let detail = Resource::new(move || selected_id.get(), move |id| async move { ... });

// 修改后
let detail = Resource::new(move || (selected_id.get(),), move |(id,)| async move { ... });
```

---

## 29. `voucher.rs` 中 `Suspense` 子元素 `FnOnce` 错误

**文件**: `src/pages/voucher.rs`

**错误**: `closure implements FnOnce`

**根因**: 闭包捕获了 `detail`（`Resource`），`Resource::get()` 需要 `Send` 环境。

**修复**: 多层克隆。
```rust
// 修改前
{move || { match detail.get() { ... } }}

// 修改后
{let detail = detail.clone();
 let on_audit = on_audit.clone();
 let on_delete = on_delete.clone();
 move || {
    let detail = detail.clone();
    let on_audit = on_audit.clone();
    let on_delete = on_delete.clone();
    match detail.get() { ... }
}}
```

---

## 30. `voucher.rs` 中 `logs` 闭包生命周期错误

**文件**: `src/pages/voucher.rs`

**错误**: `lifetime may not live long enough`

**根因**: `move` 闭包捕获了 `log` 的引用，但 `log` 生命周期仅限于 `map` 迭代器。

**修复**: 克隆 `log` 并提取变量。
```rust
// 修改前
logs.iter().map(|log| {
    view! { <Show when=move || log.comment.is_some()> ... </Show> }
})

// 修改后
logs.iter().map(|log| {
    let log = log.clone();
    let has_comment = log.comment.is_some();
    let comment = log.comment.clone().unwrap_or_default();
    view! { <Show when=move || has_comment> ... </Show> }
})
```

---

## 31. `E0225` — `Arc<dyn Fn + Clone + Send + Sync>` 不合法，改用 `Callback`

**文件**: `src/components/layout/modal.rs`, `src/pages/voucher.rs`, `src/pages/accounts.rs`, `src/pages/contacts.rs`, `src/pages/settings.rs`, `src/components/form/search_form.rs`

**错误**: `E0225` — `only auto traits can be used as additional traits in a trait object`

**根因**: Rust 不允许在 trait 对象（`dyn Fn()`）上叠加非 auto trait（如 `Clone`）。`Send`/`Sync` 是 auto trait 可以叠加，但 `Clone` 不是。

**修复**: 全部改用 Leptos 0.8 的 `Callback<T>`，它自带 `Clone + Send + Sync + 'static`。

```rust
// ❌ 错误: Arc<dyn Fn + Clone + Send + Sync> 不合法
on_close: std::sync::Arc<dyn Fn() + Clone + Send + Sync + 'static>,

// ❌ 错误: impl Fn + Clone + Send + Sync 在 For 组件 children 中传递会爆炸
on_edit: impl Fn(Account) + Clone + Send + Sync + 'static,

// ✅ 正确: 使用 Callback
on_close: Callback<()>,
on_edit: Callback<Account>,
on_delete: Callback<String>,
on_audit: Callback<(String, Option<String>)>,
on_saved: Callback<()>,
```

**调用方式**:
```rust
// 调用 Callback 使用 .run() 方法
on_close.run(());
on_edit.run(account.clone());
on_delete.run(id.clone());
on_audit.run((id.clone(), comment));
```

**创建 Callback**:
```rust
// 闭包必须接受一个参数（即使不需要也用 |_|）
Callback::new(move |_| refresh())
Callback::new(move |(id, comment): (String, Option<String>)| { ... })

// 默认值
#[prop(default = Callback::new(|_| {}))] on_search: Callback<()>,
```

**关键规则**:
1. `Callback<T>` 的闭包签名必须是 `Fn(T)` — 即使不需要参数也要写 `|_|`
2. 多参数用元组：`Callback<(String, Option<String>)>` → 闭包 `|(a, b): (String, Option<String>)|`
3. 调用用 `.run(input)` 而非 `input()` 或 `.call(input)`
4. `Callback` 自带 `Clone + Send + Sync + 'static`，无需额外约束
5. 不再需要 `Arc`/`Rc` 包裹，不再需要手动 `.clone()` 再调用

**文件**: `src/pages/voucher.rs`

**错误**: `lifetime may not live long enough`

**根因**: `move` 闭包捕获了 `log` 的引用，但 `log` 生命周期仅限于 `map` 迭代器。

**修复**: 克隆 `log` 并提取变量。
```rust
// 修改前
logs.iter().map(|log| {
    view! { <Show when=move || log.comment.is_some()> ... </Show> }
})

// 修改后
logs.iter().map(|log| {
    let log = log.clone();
    let has_comment = log.comment.is_some();
    let comment = log.comment.clone().unwrap_or_default();
    view! { <Show when=move || has_comment> ... </Show> }
})
```

---

## 通用修复原则

1. **`Resource` 使用**: 直接 `.await`，不要 `.get().await`
2. **`For` 组件**: 使用 `let:item` 而非 `let:(a, b)`，复杂 `each` 闭包用 `Memo` 封装
3. **闭包所有权**: 在 `move` 闭包前克隆所有需要多次使用的变量
4. **`match` 分支**: 使用 `.into_any()` 统一 `view!` 返回类型
5. **事件回调**: 全部使用 `Callback<T>`，不要用 `impl Fn + Clone + Send + Sync` 或 `Arc<dyn Fn + Clone + Send + Sync>`
6. **`Suspense` 子元素**: 需要 `IntoView + Send + 'static`
7. **`static` 信号**: 使用 `LazyLock` 包裹
8. **`invoke` 参数**: 结构体需序列化为 JSON 字符串
