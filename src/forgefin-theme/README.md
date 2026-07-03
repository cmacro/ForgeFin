# ForgeFin Theme System

ForgeFin 主题系统基于 **Tailwind CSS v4** 与 **CSS 自定义属性(Design Tokens)** 构建。

## 目录结构

```
src/forgefin-theme/
├── styles/
│   ├── app.css        # 主题 Token 定义(:root / html.dark) + 基础元素样式
│   ├── layout.css     # 应用骨架布局(.app-layout / .app-main / .app-content)
│   ├── sidebar.css    # 侧边导航栏
│   ├── header.css     # 顶部应用栏
│   ├── form.css       # 表单控件(input / textarea / select / form-field)
│   ├── table.css      # 数据表格(.data-table)
│   ├── card.css       # 卡片与统计卡片
│   ├── tag.css        # 标签与凭证状态标签
│   ├── modal.css      # 模态对话框
│   ├── utilities.css  # 通用工具类(text-num / text-money / scrollbar-thin)
│   └── index.css      # 入口文件,汇总所有 @import
│
├── examples/
│   ├── sidebar.html   # 侧边栏布局示例
│   ├── voucher.html   # 凭证管理页示例
│   └── dashboard.html # 仪表盘示例
│
└── README.md
```

## 构建方式

入口文件:`src/forgefin-theme/styles/index.css`

使用 Tailwind CLI 编译:

```sh
npx tailwindcss -i src/forgefin-theme/styles/index.css -o public/styles.css
```

或在 `Makefile` 中更新 `build-css` 目标:

```make
build-css:
	npx tailwindcss -i src/forgefin-theme/styles/index.css -o public/styles.css
```

## 设计原则

1. **语义化 Token**:所有颜色通过语义变量(`--color-surface`、`--color-brand`、
   `--color-posted` 等)定义,组件只消费语义,不直接使用颜色值。
2. **主题兼容**:每个 Token 在 `:root`(Classic / Light)与 `html.dark`(Dark)
   下均有定义,新增主题只需追加变量覆盖。
3. **财务语义**:内置凭证状态 Token(draft / pending / approved / posted /
   archived),与会计工作流一一对应。
4. **数值排版**:提供 `.text-num` / `.text-money` 工具类,统一金额的等宽对齐。

## 语义 Token 索引

| 分类 | Token | 用途 |
|------|-------|------|
| 表面 | `surface` / `surface-alt` / `surface-hover` / `surface-active` | 容器、页面、悬停、选中 |
| 文本 | `primary` / `secondary` / `tertiary` / `placeholder` / `disabled` | 主/次/辅助/占位/禁用 |
| 边框 | `border` / `border-light` / `divider` | 卡片、表格、分隔线 |
| 品牌 | `brand` / `brand-hover` / `brand-active` / `brand-soft` | 主按钮、链接、激活态 |
| 状态 | `success` / `warning` / `danger` / `info` (+ `-soft`) | 成功/警告/危险/信息 |
| 侧栏 | `sidebar` / `sidebar-text` / `sidebar-muted` / `sidebar-hover` / `sidebar-active` | 导航相关 |
| 凭证 | `draft` / `pending` / `approved` / `posted` / `archived` (+ `-soft`) | 凭证状态 |
| 圆角 | `radius-xs` / `radius-sm` / `radius-md` / `radius-lg` / `radius-xl` | 组件圆角 |
| 阴影 | `shadow-xs` / `shadow-sm` / `shadow-md` / `shadow-lg` | 层级阴影 |
| 布局 | `layout-header-height` / `layout-sidebar-width` | 头部高度、侧栏宽度 |
| 动效 | `transition-fast` / `transition-normal` | 过渡时长 |

## 使用示例

```html
<div class="card">
  <div class="card-header">
    <span class="card-title">本月收入</span>
  </div>
  <div class="card-body">
    <span class="text-money">¥ 1,280,500.00</span>
  </div>
</div>

<span class="tag tag-posted">
  <span class="tag-dot"></span>已过账
</span>
```