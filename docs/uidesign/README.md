---
title: ForgeFin UI 原型目录
description: docs/uidesign 原型清单、OpenDesign 统一项目 forgefin-uidesign 的预览地址规则，以及新增原型的流程
---

# ForgeFin UI 原型目录

本目录存放 ForgeFin 五大业务页面的高保真原型，以及这些原型共同遵循的**统一设计规范**。

- 设计规范（单一事实源）：[`DESIGN.md`](./DESIGN.md)
- 机器可读 token / 基础组件样式：[`tokens.css`](./tokens.css)
- 预览入口：[`index.html`](./index.html)

## 文件清单

| 文件 | 页面 | 说明 |
|---|---|---|
| `index.html` | 原型总览 | 卡片导航，链接到下面五个页面 |
| `import-center.html` | F1 导入中心 | 文件/目录导入、列映射预览、余额连续性校验、批次历史 |
| `bank-flow.html` | F2 银行流水 | 三色余额校验、原始表只读、行详情抽屉、按批次确认余额 |
| `order-flow.html` | F3 订单流水 | 手续费/实收差异校验、费率复算、归集状态 |
| `store-collection.html` | F9 店面收款 | 线下收款登记、附件、审核、作废留痕、生成汇总 |
| `reconciliation.html` | F4 对账中心 | 三方勾稽、KPI、差异清单、忽略差异、生成汇总草稿 |
| `DESIGN.md` | — | 统一设计规范（色板/字体/间距/组件/状态色/逐页规格） |
| `tokens.css` | — | 设计 token 与原型基础组件样式（与 `src/forgefin-theme/styles/app.css` 对齐） |
| `grid-stats.js` | — | 表格页脚统计：单元格选区（点击/拖拽/Shift/Ctrl+A）实时输出合计、平均、数量、最大、最小 |

## 预览方式

原型托管在 OpenDesign 项目 `forgefin-uidesign` 中，与本地文件保持同名：

```
http://127.0.0.1:7456/api/projects/forgefin-uidesign/raw/{文件名}
```

- 总览：<http://127.0.0.1:7456/api/projects/forgefin-uidesign/raw/index.html>
- 直接双击本地 HTML 也可打开（除 `tokens.css` 用相对路径外无外部依赖，无 CDN、无框架）。

## 设计规范要点

| 维度 | 约定 |
|---|---|
| 色彩 | 只用语义 token（`--color-brand`、`--color-surface`、`--color-danger` …），禁止硬编码 hex 与 Tailwind 原色类 |
| 主题 | 浅色为默认，`html.dark` 覆盖 token；切换后不得出现不可读文本 |
| 数字 | 金额两位小数、千分位、右对齐、等宽（`.text-money`）；负数红色 |
| 布局 | 侧栏 220px + 顶栏 64px + 内容区 + 页脚 40px；表格 sticky 表头、行高 30px（密度优先） |
| 状态 | 绿=正常/通过，橙=待处理/警告，红=不平/失败，蓝=已过账/汇总，灰=草稿/忽略 |
| 安全 | 原始表只读；确认余额、审核、作废、忽略差异必须二次确认并填写原因 |
| 页脚 | 每页底部固定 40px 页脚：左侧 Excel 式选中区域统计（合计/平均/数量/最大/最小 + 清除），右侧记录数与分页；未选中时显示操作提示 |
| 数据 | 全部为脱敏示例数据（账号 `1001****7575`、公司「示例商贸有限公司」） |

## 新增原型流程

1. 先读 `DESIGN.md` 与 `tokens.css`，参照基准页 `bank-flow.html` 的组件形态。
2. 在 OpenDesign 项目 `forgefin-uidesign` 根目录新增自包含 HTML，页面级样式写在 `<style>` 内，公共样式只引用 `tokens.css`。
3. 同步一份到本目录，并更新 `index.html` 入口卡与上表。
4. 保持侧栏分组与 `src/nav.rs` 一致，只改变高亮项；不得自创配色、字体、圆角或导航分组。
