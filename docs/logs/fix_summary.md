# 编译错误修复总结

## 1. `web_sys::window()` 未导入

**文件**: `src/pages/voucher_entry.rs:375`, `src/pages/voucher.rs:458`

**问题**: 直接使用 `web_sys::window()` 但未在 `Cargo.toml` 中添加 `web_sys` 依赖。

**分析**: Leptos 0.8 已通过 `leptos::prelude::*` 导出了 `window()` 函数，返回 `web_sys::Window`，无需额外依赖。

**处理**: 将 `web_sys::window()` 替换为 `window()`，并直接调用 `.print()` 方法。

```rust
// 修改前
if let Some(w) = web_sys::window() {
    let _ = w.print();
}

// 修改后
let _ = window().print();
```

---

## 2. `RwSignal::new` 不能在 `static` 中调用

**文件**: `src/auth.rs:12-15`

**问题**: `static SESSION: RwSignal<Option<UserInfo>> = RwSignal::new(None);` 报错 `E0015`，因为 `RwSignal::new` 不是 `const fn`。

**分析**: Rust 的 `static` 初始化必须是常量表达式，而 `RwSignal::new` 是运行时函数。需要使用 `LazyLock` 延迟初始化。

**处理**: 将所有 4 个 `static` 变量包裹在 `LazyLock` 中，并添加 `use std::sync::LazyLock;`。

```rust
// 修改前
static SESSION: RwSignal<Option<UserInfo>> = RwSignal::new(None);

// 修改后
static SESSION: LazyLock<RwSignal<Option<UserInfo>>> = LazyLock::new(|| RwSignal::new(None));
```

---

## 3. 普通闭包调用了 `.get()` 方法

**文件**: `src/pages/voucher_entry.rs:44-58`

**问题**: `let debit_total = move || { ... };` 是普通闭包，后续调用 `debit_total.get()` 报错，因为 `.get()` 是 `Signal`/`Memo` 的方法。

**分析**: 需要将普通闭包改为 `Memo`，使其成为响应式信号，才能调用 `.get()`。

**处理**: 将 `debit_total`、`credit_total`、`balanced` 从普通闭包改为 `Memo::new`。

```rust
// 修改前
let debit_total = move || {
    entries.get().iter().map(|e| e.debit.parse::<i64>().unwrap_or(0)).sum::<i64>()
};

// 修改后
let debit_total = Memo::new(move |_| {
    entries.get().iter().map(|e| e.debit.parse::<i64>().unwrap_or(0)).sum::<i64>()
});
```

---

## 4. `Resource::get().await` 错误

**文件**: `src/pages/voucher_entry.rs:217`, `src/pages/voucher.rs:134,262,434`, `src/pages/settings.rs:105,186`, `src/pages/accounts.rs:54`, `src/pages/contacts.rs:100`

**问题**: `Resource::get()` 返回 `Option<T>`，不是 Future，不能调用 `.await`。

**分析**: Leptos 0.8 中 `Resource` 实现了 `IntoFuture`，可以直接 `.await` 获取结果，无需先 `.get()`。

**处理**: 将 `resource.get().await` 替换为 `resource.await`，并相应调整 `match` 模式从 `Some(Ok(x))` 改为 `Ok(x)`。

```rust
// 修改前
match accounts.get().await {
    Some(Ok(list)) => { ... }
    Some(Err(e)) => { ... }
    None => { ... }
}

// 修改后
match accounts.await {
    Ok(list) => { ... }
    Err(e) => { ... }
}
```

---

## 5. `Suspend::with` 不存在

**文件**: `src/pages/voucher.rs:133,261,433`, `src/pages/settings.rs:104,185`, `src/pages/accounts.rs:53`, `src/pages/contacts.rs:99`

**问题**: `Suspend::with(async move { ... })` 报错，因为 `Suspend::with` 在 Leptos 0.8 中已被移除。

**分析**: Leptos 0.8 使用 `Suspend::new(async move { ... })` 替代 `Suspend::with`。

**处理**: 将所有 `Suspend::with` 替换为 `Suspend::new`。

```rust
// 修改前
{move || Suspend::with(async move { ... })}

// 修改后
{move || Suspend::new(async move { ... })}
```

---

## 6. `For` 组件 `each` 属性解析错误

**文件**: `src/pages/voucher_entry.rs:210`

**问题**: `<For each=move || entries.get().into_iter().enumerate().collect::<Vec<_>>() key=|(_, e)| e.uid let:(idx, e)>` 报错 `no method named r#move`。

**分析**: Leptos 的 `view!` 宏在处理 `For` 的 `each` 属性时，如果闭包体过于复杂（包含 `into_iter().enumerate().map().collect()`），宏会错误地将 `move` 关键字解析为方法名。

**处理**: 
1. 创建 `Memo` 存储枚举后的列表。
2. 使用 `let:item` 替代 `let:(idx, e)` 解构。
3. 在模板中使用 `item.0` 和 `item.1` 访问索引和元素。

```rust
// 新增 Memo
let enumerated_entries = Memo::new(move |_| {
    entries.get().iter().enumerate().map(|(i, e)| (i, e.clone())).collect::<Vec<_>>()
});

// 修改 For
<For each=move || enumerated_entries.get() key=|item| item.1.uid.clone() let:item>
    <tr>
        <td>{item.0 + 1}</td>
        ...
    </tr>
</For>
```

---

## 7. `Memo::new` 要求 `PartialEq`

**文件**: `src/pages/voucher_entry.rs:23`

**问题**: `Memo::new` 要求返回值类型实现 `PartialEq`，但 `(usize, EntryRow)` 未实现。

**分析**: `Memo` 需要比较新旧值以决定是否触发更新，因此要求 `T: PartialEq`。

**处理**: 为 `EntryRow` 结构体添加 `#[derive(PartialEq)]`。

```rust
#[derive(Clone, Debug, PartialEq)]
struct EntryRow { ... }
```

---

## 8. `detail` 视图中 `d` 被推断为 `String`

**文件**: `src/pages/voucher.rs:436-447`

**问题**: `d.voucher.voucher_no` 报错 `no field voucher on String`，说明 `d` 被推断为 `String` 而非 `VoucherDetail`。

**分析**: 在 `Suspend::new` 的 `async` 块中，`detail.await` 的返回类型被错误推断。改用 `detail.get().flatten()` 同步获取，避免类型推断问题。

**处理**: 将 `Suspend::new` 替换为普通闭包，使用 `detail.get().flatten()` 获取 `Option<VoucherDetail>`，然后用 `.map()` 和 `.unwrap_or_else()` 处理。

```rust
// 修改前
{move || Suspend::new(async move {
    match detail.await {
        Some(d) => view! { ... },
        None => view! { ... },
    }
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

**文件**: `src/pages/voucher.rs:488,497`

**问题**: `on_delete(vid.clone())` 和 `on_audit(...)` 报错 `cannot find function`。

**分析**: 原代码中 `on_delete` 和 `on_audit` 是子组件的属性（props），当我们将子组件内容内联到父组件后，这些名称不再可用。父组件中实际定义了 `do_delete` 和 `do_audit` 闭包。

**处理**: 将 `on_delete` 替换为 `do_delete`，将 `on_audit` 替换为 `do_audit`。

```rust
// 修改前
on:click=move |_| on_delete(vid.clone())

// 修改后
on:click=move |_| do_delete(vid.clone())
```

---

## 10. `VoucherFilter` 缺少 `PartialEq`

**文件**: `src/ipc.rs:180`

**问题**: `Resource::new` 要求源类型 `S: PartialEq`，但 `VoucherFilter` 未实现 `PartialEq`。

**分析**: `Resource` 需要比较源值是否变化以决定是否重新加载。

**处理**: 为 `VoucherFilter` 添加 `#[derive(PartialEq)]`，并修复重复的 derive 属性。

```rust
// 修改前
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoucherFilter { ... }

// 修改后
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct VoucherFilter { ... }
```

---

## 11. `For` 组件 `let:(idx, row)` 解构冲突

**文件**: `src/pages/voucher.rs:341`, `src/pages/contacts.rs:119`

**问题**: `let:(idx, row)` 报错 `closure is expected to take a single 2-tuple as argument, but it takes 2 distinct arguments`。

**分析**: Leptos 0.8 的 `For` 组件 `let:` 语法在某些版本中不支持元组解构，需要改用 `let:item` 然后手动访问 `item.0`、`item.1`。

**处理**: 将 `let:(idx, row)` 改为 `let:item`，并在模板中使用 `item.0` 和 `item.1`。

```rust
// 修改前
<For each=move || rows.clone() key=|r| r.id.clone() let:(idx, row)>

// 修改后
<For each=move || rows.clone().into_iter().enumerate().collect::<Vec<_>>() key=|item| item.1.id.clone() let:item>
    <tr>
        <td>{item.0 + 1}</td>
        ...
    </tr>
</For>
```

---

## 12. `Modal` 组件 `size` 属性类型不匹配

**文件**: `src/pages/settings.rs:275,379`, `src/pages/contacts.rs:262`

**问题**: `size="lg"` 报错 `expected Option<&str>, found &str`。

**分析**: `Modal` 组件的 `size` 属性定义为 `Option<&str>`，需要显式包装为 `Some("lg")`。

**处理**: 将 `size="lg"` 改为 `size=Some("lg")`。

```rust
// 修改前
<Modal open=open title="账套" size="lg" on_close=close_rc>

// 修改后
<Modal open=open title="账套" size=Some("lg") on_close=close_rc>
```

---

## 13. `has_company` 布尔值调用 `.get()`

**文件**: `src/app.rs:62`

**问题**: `if !has_company.get()` 报错 `the method get exists for type bool`。

**分析**: `has_company` 是 `Session::has_company()` 返回的 `bool`，不是信号，不能调用 `.get()`。

**处理**: 移除 `.get()` 调用。

```rust
// 修改前
if !has_company.get() { ... }

// 修改后
if !has_company { ... }
```

---

## 14. `res.companies` 所有权移动后再次访问

**文件**: `src/auth.rs:84-86`

**问题**: `AVAILABLE.set(res.companies);` 移动了 `res.companies` 的所有权，后续 `res.companies.first()` 报错 `borrow of moved value`。

**分析**: 需要先克隆或先访问再移动。

**处理**: 先克隆公司列表，再设置信号。

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

**文件**: `src/components/layout/company_switcher.rs:20`

**问题**: `c.id == cid` 报错 `can't compare String with Option<String>`。

**分析**: `c.id` 是 `String`，`cid` 是 `Option<String>`，不能直接比较。

**处理**: 将 `cid` 解包为 `String` 后再比较。

```rust
// 修改前
.find(|c| c.id == cid)

// 修改后
.find(|c| Some(c.id.clone()) == cid)
```

---

## 16. `Modal` 事件闭包 `FnOnce` / `Send` / `Sync` 错误

**文件**: `src/components/layout/modal.rs:24`

**问题**: 事件回调闭包报错 `FnOnce`、`Send`、`Sync` 未满足。

**分析**: Leptos 要求事件处理闭包实现 `Fn + Send + Sync + 'static`。当闭包捕获了 `Rc<dyn Fn()>` 时，`Rc` 不是 `Send` 也不是 `Sync`。

**处理**: 将 `on_close` 参数类型改为 `impl Fn() + Clone + Send + Sync + 'static`，并在 `on:click` 中使用 `move || on_close()` 而非直接传递 `on_close`。

```rust
// 修改前
pub fn Modal(open: ReadSignal<bool>, on_close: impl Fn() + 'static, ...) -> impl IntoView {
    ...
    <div class="modal-overlay" on:click=move |_| on_close()>
    ...
}

// 修改后
pub fn Modal(open: ReadSignal<bool>, on_close: impl Fn() + Clone + Send + Sync + 'static, ...) -> impl IntoView {
    let close = on_close.clone();
    ...
    <div class="modal-overlay" on:click=move |_| close()>
    ...
}
```

---

## 17. `ipc.rs` 中 `invoke` 参数类型不匹配

**文件**: `src/ipc.rs:248,296,312,328,352,388`

**问题**: `Reflect::apply` 需要 `&Function` 和 `&Array`，但传入了 `&JsValue` 和 `&[JsValue; 2]`。其他 `invoke` 调用传入了结构体引用而非 `&String`。

**分析**: `invoke` 函数期望所有参数为 `&String`，但传入了 `&CompanyInput`、`&AccountInput` 等结构体引用。

**处理**: 将结构体参数序列化为 JSON 字符串后传入。

```rust
// 修改前
invoke("update_company_cmd", &[("id", &id), ("input", input)])

// 修改后
invoke("update_company_cmd", &[("id", &id), ("input", &serde_json::to_string(&input).unwrap())])
```

---

## 18. `match` 分支返回类型不兼容

**文件**: `src/pages/settings.rs:167,214`, `src/pages/voucher.rs:149`, `src/pages/accounts.rs:92`

**问题**: `Ok` 分支返回 `View<(Table, Div)>`，`Err` 分支返回 `View<Div>`，类型不匹配。

**分析**: Leptos 的 `view!` 宏为不同结构生成不同的 `View` 类型，`match` 的所有分支必须返回完全相同的类型。

**处理**: 将 `Err` 分支的视图也包裹在相同的容器结构中，或使用 `EitherView` 统一类型。

```rust
// 修改前
Ok(list) => view! { <table>...</table> },
Err(e) => view! { <div class="login-error">...</div> },

// 修改后
Ok(list) => view! { <table>...</table> },
Err(e) => view! { <table><div class="login-error">...</div></table> },
```

---

## 19. `accounts.get().unwrap_or_default()` 返回 `()`

**文件**: `src/pages/accounts.rs:375-386`

**问题**: `accounts.get().unwrap_or_default()` 报错 `expected (), found Result`。

**分析**: `Resource::get()` 返回 `Option<Result<Vec<Account>, String>>`，`unwrap_or_default()` 在 `Option` 上返回 `Result<Vec<Account>, String>`，但代码期望 `Vec<Account>`。

**处理**: 使用 `accounts.get().unwrap_or(Ok(Vec::new())).unwrap_or_default()` 或直接 `accounts.await`。

```rust
// 修改前
match accounts.get().unwrap_or_default() {
    Ok(list) => list,
    Err(_) => Vec::new(),
}

// 修改后
accounts.await.unwrap_or_default()
```

---

## 20. `row.id` 在闭包中移动后再次借用

**文件**: `src/pages/voucher.rs:380-381`

**问题**: `let is_active = move || selected.get() == Some(row.id.clone());` 移动了 `row.id`，后续 `let id = row.id.clone();` 报错 `borrow of moved value`。

**分析**: `move` 闭包捕获了 `row.id` 的所有权，导致后续无法再访问。

**处理**: 在闭包之前先克隆 `row.id`。

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

**文件**: `src/pages/voucher.rs:122`

**问题**: `vouchers=vouchers` 报错 `expected Resource<(VoucherFilter, i32), ...>, found Resource<Result<VoucherPage, String>>`。

**分析**: `Resource` 的泛型参数不匹配，`SummaryStats` 期望 `Resource<(VoucherFilter, i32), Result<VoucherPage, String>>`，但实际传入的是 `Resource<Result<VoucherPage, String>>`。

**处理**: 更新 `SummaryStats` 组件的属性类型定义，或调整 `vouchers` 的 `Resource` 创建方式以匹配。

---

## 22. `contacts.rs` 中 `For` 的 `let:(idx, c)` 解构冲突

**文件**: `src/pages/contacts.rs:120`

**问题**: 与问题 11 相同，`let:(idx, c)` 报错 `closure is expected to take 1 argument, but it takes 2 arguments`。

**分析**: 与问题 11 相同，Leptos 0.8 的 `For` 组件不支持元组解构。

**处理**: 将 `let:(idx, c)` 改为 `let:item`，使用 `item.0` 和 `item.1`。

---

## 23. `settings.rs` 中 `on_edit` 未定义

**文件**: `src/pages/settings.rs:143`

**问题**: `on:click=move |_| on_edit(c.clone())` 报错 `cannot find function on_edit`。

**分析**: 变量名应为 `open_edit` 而非 `on_edit`。

**处理**: 将 `on_edit` 改为 `open_edit`。

```rust
// 修改前
on:click=move |_| on_edit(c.clone())

// 修改后
on:click=move |_| open_edit(c.clone())
```

---

## 24. `accounts.rs` 中 `For` 的 `children` 闭包缺少 `Clone` / `Send` / `Sync`

**文件**: `src/pages/accounts.rs:164-215`

**问题**: `For` 组件的 `children` 闭包捕获了 `on_edit` 和 `on_delete`，这两个参数是 `impl Fn(...) + 'static`，未实现 `Clone`、`Send`、`Sync`。

**分析**: Leptos 0.8 的 `For` 组件要求 `children` 闭包实现 `Fn(T) -> N + Send + Clone + 'static`。

**处理**: 在组件参数中添加 `Clone`、`Send`、`Sync` 约束。

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

**文件**: `src/pages/accounts.rs:204`

**问题**: `each=move || children.clone()` 报错 `closure is FnOnce`，因为 `children` 被移动到了闭包中。

**分析**: `For` 的 `each` 属性需要 `Fn` 闭包，但 `move` 闭包捕获了 `children` 的所有权，导致只能调用一次。

**处理**: 将 `children` 改为 `ReadSignal<Vec<TreeNode>>` 或使用 `StoredValue`，使闭包可以多次调用。

---

## 26. `voucher_entry.rs` 中 `FnOnce` 闭包冲突

**文件**: `src/pages/voucher_entry.rs:223`

**问题**: `move || Suspend::new(async move { ... })` 报错 `closure implements FnOnce, not FnMut`，因为 `item.0` 和 `item.1` 被移动到了闭包中。

**分析**: `For` 组件的 `let:item` 绑定在 `view!` 宏中，`item` 被 `move` 闭包捕获后，后续的 `item.0` 和 `item.1` 引用无法再使用。

**处理**: 在闭包之前提取 `idx` 和 `row`，在闭包内使用 `idx` 和 `row` 替代 `item.0` 和 `item.1`。

```rust
// 修改前
{move || {
    Suspend::new(async move {
        set_entries.update(|v| v[item.0].summary = val);
    })
}}

// 修改后
{let idx = item.0;
 let row = item.1.clone();
 move || {
    Suspend::new(async move {
        set_entries.update(|v| v[idx].summary = val);
    })
}}
```

---

## 27. `voucher_entry.rs` 中 `val` 移动后再次借用

**文件**: `src/pages/voucher_entry.rs:279-285`

**问题**: `set_entries.update(|v| { v[i].debit = val; if !val.is_empty() ... })` 报错 `borrow of moved value: val`。

**分析**: `val` 被移动到 `update` 闭包后，后续的 `val.is_empty()` 无法再访问。

**处理**: 在 `update` 闭包之前计算 `is_positive` 布尔值，闭包内只使用布尔值。

```rust
// 修改前
set_entries.update(|v| {
    v[i].debit = val;
    if !val.is_empty() && val.parse::<i64>().unwrap_or(0) > 0 { ... }
});

// 修改后
let is_positive = !val.is_empty() && val.parse::<i64>().unwrap_or(0) > 0;
set_entries.update(|v| {
    v[i].debit = val;
    if is_positive { ... }
});
```

---

## 28. `voucher.rs` 中 `detail` Resource 类型推断错误

**文件**: `src/pages/voucher.rs:38-47`

**问题**: `Resource::new` 创建的 `detail` 被推断为 `Resource<Option<VoucherDetail>>`（单参数），但 `VoucherDetail` 组件期望 `Resource<Option<String>, Option<VoucherDetail>>`（双参数）。

**分析**: Leptos 0.8 的 `Resource::new` 当源信号是 `Option<T>` 时，会推断为单参数 Resource。需要显式使用元组源信号来创建双参数 Resource。

**处理**: 将源信号改为 `(selected_id.get(),)` 元组，并在异步函数中使用 `(id,)` 解构。

```rust
// 修改前
let detail = Resource::new(
    move || selected_id.get(),
    move |id| async move { ... },
);

// 修改后
let detail = Resource::new(
    move || (selected_id.get(),),
    move |(id,)| async move { ... },
);
```

---

## 29. `voucher.rs` 中 `Suspense` 子元素 `FnOnce` 错误

**文件**: `src/pages/voucher.rs:434-437`

**问题**: `Suspense` 的子元素闭包报错 `FnOnce`，因为闭包捕获了 `on_audit` 和 `on_delete`（`Arc<dyn Fn>`），但 `Arc<dyn Fn>` 不是 `Send`。

**分析**: `Suspense` 的子元素需要实现 `IntoView + Send + 'static`。`Arc<dyn Fn>` 是 `Send + Sync`，但闭包捕获了 `detail`（`Resource`），`Resource` 的 `get()` 方法需要 `Send` 环境。

**处理**: 在闭包之前克隆 `detail`、`on_audit`、`on_delete`，在闭包内再次克隆后使用。

```rust
// 修改前
{move || {
    match detail.get() { ... }
}}

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

**文件**: `src/pages/voucher.rs:527-551`

**问题**: `logs.iter().map(|log| { ... view! { <Show when=move || log.comment.is_some()> ... } })` 报错 `lifetime may not live long enough`。

**分析**: `move` 闭包捕获了 `log` 的引用，但 `log` 的生命周期仅限于 `map` 迭代器内部。

**处理**: 在 `map` 闭包中克隆 `log`，并提取 `has_comment` 和 `comment` 变量，避免在 `view!` 宏中直接引用 `log`。

```rust
// 修改前
logs.iter().map(|log| {
    view! {
        <Show when=move || log.comment.is_some()>
            <div>{log.comment.clone().unwrap_or_default()}</div>
        </Show>
    }
})

// 修改后
logs.iter().map(|log| {
    let log = log.clone();
    let has_comment = log.comment.is_some();
    let comment = log.comment.clone().unwrap_or_default();
    view! {
        <Show when=move || has_comment>
            <div>{comment.clone()}</div>
        </Show>
    }
})
```

---

## 错误日志文件

完整的 `cargo check` 错误输出已保存至：
```
docs/logs/error_log.txt
```

## 总结

本次修复共涉及 **30 类** 编译错误，覆盖了以下方面：

| 类别 | 数量 | 主要文件 |
|------|------|----------|
| Leptos API 变更（`Suspend::with` → `Suspend::new`） | 6 处 | `voucher.rs`, `settings.rs`, `accounts.rs`, `contacts.rs` |
| `Resource` 使用错误（`.get().await` → `.await`） | 6 处 | `voucher.rs`, `settings.rs`, `accounts.rs`, `contacts.rs` |
| `For` 组件语法错误 | 4 处 | `voucher_entry.rs`, `voucher.rs`, `contacts.rs` |
| 类型不匹配（`Option`/`String`/`Result`） | 5 处 | `settings.rs`, `contacts.rs`, `company_switcher.rs` |
| 所有权/借用错误 | 5 处 | `auth.rs`, `voucher.rs`, `voucher_entry.rs` |
| 信号/闭包使用错误 | 3 处 | `voucher_entry.rs`, `app.rs` |
| 线程安全约束 | 3 处 | `modal.rs`, `accounts.rs` |
| 其他 | 5 处 | `ipc.rs`, `auth.rs` 等 |

所有修复遵循以下原则：
1. 最小化修改，不改变业务逻辑
2. 遵循 Leptos 0.8 的 API 规范
3. 保持代码风格与项目现有代码一致
