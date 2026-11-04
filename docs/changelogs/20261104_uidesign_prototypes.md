# UI 设计规范与五页高保真原型

日期：2026-11-04

## 变更内容

- 新增统一设计规范 `docs/uidesign/DESIGN.md`（v1.0）：语义色彩 token、字体与数字规范、间距圆角、页面骨架（含页脚）、组件规范、状态色映射、五大页面逐页规格、硬性业务约束与检查清单。
- 新增机器可读 token / 基础组件样式 `docs/uidesign/tokens.css`，与 `src/forgefin-theme/styles/app.css` 对齐，支持 `html.dark` 深色主题。
- 新增表格页脚统计脚本 `docs/uidesign/grid-stats.js`：单元格点击 / 拖拽框选 / `Shift` 扩选 / `Ctrl(⌘)+A` 全选 / `Esc` 清除，页脚实时输出选中区域各数值列的合计、平均、数量、最大、最小；标识列（单号、账号、电话、批次号）只统计非空数量。
- 新增五个页面的高保真原型（OpenDesign 项目 `forgefin-uidesign`，同步到 `docs/uidesign/`）：
  - `import-center.html` F1 导入中心
  - `bank-flow.html` F2 银行流水
  - `order-flow.html` F3 订单流水
  - `store-collection.html` F9 店面收款
  - `reconciliation.html` F4 对账中心
  - `index.html` 原型总览（卡片导航）

## 设计要点

- 页面骨架统一为：侧栏 220px（深色，分组与 `src/nav.rs` 一致）+ 顶栏 64px + 内容区 + **页脚 40px**。
- 表格采用密度优先：行高 30px（表头 32px，紧凑 28px），正文 12.5px，内边距 0×10px，一屏可多显示约 1/3 行数。
- 页脚为 Excel 式区域统计 + 记录数 / 分页，禁止再出现第二条底部信息条。
- 金额两位小数、千分位、右对齐等宽；负数红色。
- 原始流水只读；确认余额、审核、作废、忽略差异必须二次确认并填写原因。
- 示例数据全部脱敏（`1001****7575`、`138****1234`、示例商贸有限公司）。

## 预览

```
http://127.0.0.1:7456/api/projects/forgefin-uidesign/raw/index.html
```

## 后续

原型仅覆盖 UI 层；落地时按 `DESIGN.md` §5.3 的实现映射写入 `src/pages/`，可复用组件放 `src/components/`，样式全部使用语义 token。
