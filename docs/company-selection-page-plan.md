# 企业账套选择界面 — 开发任务

## 1. 背景分析

### 当前流程

```
登录 → Session::login() → 自动选中第一个公司(若有) → MainShell(有账套) / NoCompany(无账套)
```

### 问题

1. `Session::login()` 在 `auth.rs:87-91` 自动选中第一个公司，用户没有选择机会
2. 无账套时 `app.rs:102-110` 的 `NoCompany` 只是一个简单的空状态提示，不是完整的选择界面
3. 用户无法在登录后主动选择/切换账套（虽然有 `CompanySwitcher` 但仅在公司列表 >1 时显示）

### 目标

- 登录后如果没有账套 → 显示账套选择/创建界面
- 登录后如果有账套 → 显示账套选择界面（让用户主动选择，而非自动选中第一个）
- 界面应包含：已有账套列表 + 新建账套入口

---

## 2. 涉及文件

| 文件 | 修改类型 | 说明 |
|------|----------|------|
| `src/pages/company_selection.rs` | **新建** | 账套选择页面组件 |
| `src/pages/mod.rs` | 修改 | 注册新 page 模块 |
| `src/app.rs` | 修改 | 用 CompanySelection 替换 NoCompany |
| `src/auth.rs` | 修改 | 登录后不再自动选中第一个公司 |
| `src/components/layout/company_switcher.rs` | 可选优化 | 确保无公司时隐藏 |

---

## 3. 详细任务

### 3.1 新建 `src/pages/company_selection.rs`

**功能：**
- 用户已登录但未选择账套时显示
- 列出当前用户可访问的所有公司（`Session::available_companies()`）
- 每行显示：公司名称、税号、法人、币种、状态
- 点击「进入账套」按钮 → 调用 `Session::switch_company(id)` → 切换到主界面
- 底部「新建账套」按钮 → 弹出 `CompanyEditModal`（复用 `settings.rs` 中的逻辑，或提取为公共组件）
- 空状态：无公司时显示引导文案 + 新建按钮

**设计参考：**
- 居中卡片布局，类似登录页风格
- 公司列表使用简洁的卡片或表格行
- 每个公司卡片显示关键信息 + 操作按钮

### 3.2 修改 `src/pages/mod.rs`

```rust
pub mod company_selection;
```

### 3.3 修改 `src/app.rs`

- 将 `NoCompany` 组件替换为 `CompanySelection` 页面
- 条件逻辑改为：`!has_company` → 显示 `CompanySelection` 而非 `NoCompany`

### 3.4 修改 `src/auth.rs`

- `login()` 方法中移除自动选中第一个公司的逻辑（删除 `auth.rs:86-91`）
- 登录成功后仅设置 `SESSION` 和 `AVAILABLE`，不设置 `COMPANY_ID`
- 这样 `has_company()` 返回 `false`，自然显示账套选择页

### 3.5 提取公共组件（可选但推荐）

- 将 `settings.rs` 中的 `CompanyEditModal` 提取到 `src/components/layout/company_edit_modal.rs`
- 这样 `company_selection.rs` 和 `settings.rs` 可以复用

---

## 4. 数据流

```
登录成功
  ↓
Session::login() 设置 user + available_companies, 不设 company_id
  ↓
AppRouter 检测: user.is_some() && !has_company()
  ↓
显示 CompanySelection 页面
  ↓
用户点击「进入账套」
  ↓
Session::switch_company(id) → 设置 company_id
  ↓
AppRouter 检测: has_company() == true
  ↓
显示 MainShell (正常业务页面)
```

---

## 5. 实现顺序

1. 提取 `CompanyEditModal` 为公共组件
2. 新建 `src/pages/company_selection.rs`
3. 修改 `src/pages/mod.rs` 注册新模块
4. 修改 `src/auth.rs` 移除自动选公司
5. 修改 `src/app.rs` 替换 NoCompany
6. `cargo fmt && cargo check` 验证

---

## 6. 验收标准

- [ ] 登录后停留在账套选择页，不自动进入任何账套
- [ ] 有可用公司时，列表展示所有公司信息
- [ ] 点击「进入账套」后切换到主界面
- [ ] 无公司时显示空状态引导 + 新建按钮
- [ ] 新建账套后自动出现在列表中
- [ ] `cargo fmt` 通过
- [ ] `cargo check` 通过
