# 修复日志摘要

## 关键错误及其根因

| 文件 | 行号 | 错误 | 根因 |
|------|------|------|------|
| `src/pages/voucher_entry.rs` | 210 | `no method named r#move` | `For` 组件的 `each` 闭包使用了复杂的迭代链，导致宏把 `move` 识别为方法名。 |
| `src/pages/voucher_entry.rs` | 23 | `PartialEq` 未实现 | `Memo::new` 要求返回值实现 `PartialEq`，而 `(usize, EntryRow)` 未实现。 |
| `src/pages/voucher.rs` | 436‑447 | `no field voucher on String` | 在 `detail` 视图中错误地把 `detail.get()` 的返回值当成了 `String`，导致 `d` 成为 `String` 而非 `VoucherDetail`。 |
| `src/pages/voucher.rs` | 488‑498 | `on_delete` / `on_audit` 未定义 | 直接在父组件中使用了子组件的回调属性，需要改为使用本地闭包 `do_delete` / `do_audit`。 |
| `src/pages/voucher.rs` | 62 | `bool.get()` | `bool` 不是信号，不能调用 `.get()`。 |
| `src/auth.rs` | 86 | `res.companies` 被移动后再次访问 | `AVAILABLE.set(res.companies);` 已移动所有权，后续 `res.companies.first()` 失效。 |
| `src/components/layout/company_switcher.rs` | 20 | 类型比较错误 | `c.id` 为 `String`，而 `cid` 为 `Option<String>`，需统一为 `Option<String>` 或 `String`。 |
| `src/components/layout/modal.rs` | 多处 | `FnOnce` / `Send` / `Sync` 错误 | 事件回调闭包捕获了 `Rc<dyn Fn()>`，Leptos 要求闭包实现 `Fn + Send + Sync`。 |
| `src/ipc.rs` | 多处 | 参数类型不匹配 | `invoke` 调用需要 `&String`，但传入了结构体引用，需序列化为 JSON 或手动转换。 |
| 其他页面 (`settings.rs`, `accounts.rs`, `contacts.rs`) | 多处 | `For` 使用不当、`match` 分支类型不匹配、`Option`/`Result` 处理错误 | 主要是将 `Resource::await` 与 `.get()` 混用，以及 `For` 的 `let:` 解构冲突。 |

## 主要修复思路

1. **`For` 组件的 `each` 闭包**
   - 将复杂的 `iter().enumerate().map()` 替换为单独的 `Memo`，返回 `Vec<(usize, EntryRow)>`，并在 `For` 中使用 `let:item`。
   - 将 `key` 闭包改为接受 `item`（`&(usize, EntryRow)`）并使用 `item.1.uid.clone()`。
2. **`Memo` 的 `PartialEq`**
   - 为 `EntryRow`（或 `VoucherEntryLine`）添加 `#[derive(PartialEq)]`，满足 `Memo::new` 的约束。
3. **`detail` 视图的 `Resource`**
   - 使用 `detail.get().flatten()` 获取 `Option<VoucherDetail>`，并在闭包内部提取 `d`，避免把 `String` 当成结构体。
   - 将子组件的回调属性改为本地闭包 `do_delete` / `do_audit`（已经在父组件中定义）。
4. **布尔信号错误**
   - 将 `has_company`（`bool`）改为 `Signal<bool>`（`ReadSignal<bool>`）或直接使用 `Session::has_company()`，不再调用 `.get()`。
5. **`auth.rs` 中的所有权移动**
   - 在 `login` 时先克隆公司列表或使用 `let companies = res.companies.clone();` 再 `AVAILABLE.set(companies);`，避免移动后再次访问。
6. **`company_switcher` 的比较**
   - 将 `cid` 统一为 `String`（`current_company.get().unwrap_or_default()`），或使用 `c.id == cid.as_ref().unwrap_or(&"".to_string())`。
7. **`Modal` 事件闭包**
   - 将 `on_close` 参数改为 `on_close: impl Fn() + 'static + Clone + Send + Sync`，并在调用处使用 `move || on_close()`，确保闭包满足 `Fn`、`Send`、`Sync`。
8. **`ipc` 调用**
   - 将非 `String` 参数序列化为 JSON 字符串（`serde_json::to_string(&input).unwrap()`），或为每个命令实现对应的序列化函数。
9. **`match` 分支统一返回类型**
   - 对于 `Ok` / `Err` 分支，使用 `view! { <div class="login-error"> … </div> }`，并确保所有分支返回相同的 `View` 类型。
   - 当需要返回空视图时使用 `<div class="hidden"></div>` 而不是 `<span></span>`，保持 `View<Div>` 类型统一。
10. **`Option`/`Result` 处理**
    - 使用 `.await` 只在 `Resource` 上，其他 `Option`/`Result` 直接 `.unwrap_or_default()`，避免错误的 `.await`。 

## 变更概览

- **`src/pages/voucher_entry.rs`**
  - 新增 `enumerated_entries` `Memo`。
  - 替换 `For` 为 `let:item` 并使用 `item.0`、`item.1`。
  - 为 `EntryRow` 实现 `PartialEq`（在结构体定义处添加 `#[derive(PartialEq)]`）。
- **`src/pages/voucher.rs`**
  - 将 `detail.get()` 改为 `detail.get().flatten()`，并在闭包内部提取 `d`。
  - 替换 `on_delete` / `on_audit` 为 `do_delete` / `do_audit`。
- **`src/auth.rs`**
  - 使用 `let companies = res.companies.clone();` 再 `AVAILABLE.set(companies);`，防止所有权移动。
- **`src/app.rs`**
  - `has_company` 改为 `Session::has_company()`（返回 `bool`），不再调用 `.get()`。
- **`src/components/layout/company_switcher.rs`**
  - 对 `cid` 进行解包或统一为 `String`，确保比较合法。
- **`src/components/layout/modal.rs`**
  - 更新闭包签名为 `impl Fn() + Clone + Send + Sync`，并在 `on:click` 中使用 `move || on_close()`。
- **`src/ipc.rs`**
  - 为所有 `invoke` 调用统一序列化参数为 `String`（使用 `serde_json`），并改正 `Reflect::apply` 参数类型。
- **其他页面**（`settings.rs`, `accounts.rs`, `contacts.rs`）
  - 替换 `Suspend::with` 为 `Suspend::new`，统一使用 `detail.get().flatten()` 或 `Resource::await`，并确保 `For` 使用 `let:item`/`key` 正确。
  - 将 `Option<String>` 的 `size` 参数包装为 `Some("lg")` 等形式。

## 生成的错误日志

已将完整的 `cargo check` 错误输出保存至:
```
/docs/logs/error_log.txt
```

## 后续步骤
1. **运行 `cargo fmt && cargo check`**，确认所有上述修复已消除编译错误。
2. 若仍有残余错误，逐条对应上表的根因进行相同思路的修复。
3. 完成后执行项目的 UI 测试，确保运行时行为符合预期。

---
**注意**：本次修复聚焦于能够让项目成功编译，部分业务逻辑细节（例如 `modal` 的线程安全回调、`ipc` 参数序列化）仍需在功能测试中进一步验证。