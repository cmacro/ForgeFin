# 20260806_data_summary_independent_table

## Added

- **数据汇总独立表 `data_summaries`** — 从 `source_records` 通用表分离为独立表，字段对应 `数据汇总.tsv`（日期、收据编号、分类、项目、事由、支付方式、支付金额、手续费、实际收入、支出、余额、备注、来源信息、凭证关联）。
- **支付方式枚举** — `payment_method` 列替代原来的 `debit_card_fee`/`credit_card_fee`/`wechat_alipay` 三列，取值 `debit_card` / `credit_card` / `wechat` / `alipay`。
- **企业通用费率表 `fee_rates`** — 按支付方式存储费率，种子数据：借记卡 0.5%、信用卡 0.6%、微信 0.25%、支付宝 0.25%。
- **来源追溯 `source_info`** — JSON 列记录生成来源（`manual` / `order_flow` / `bank_flow` / `import`），手工录入时自动填充操作人姓名。
- **前端 CRUD 页面** — 数据汇总列表（含日期/分类筛选、分页、来源标签列）+ 编辑弹窗（新增/修改）+ 手续费率管理弹窗（在线编辑各支付方式费率）。

## Changed

- **`data_summaries` 表结构** — 移除 `debit_card_fee`、`credit_card_fee`、`wechat_alipay` 三列，新增 `payment_method`、`payment_amount`、`source_info` 三列。
- **`create_data_summary_core`** — 新增 `operator_name` 参数，自动写入 `source_info`。
- **前端 IPC 类型** — `DataSummaryRecord` 新增 `source_info` 字段，`DataSummaryInput` 替换为 `payment_method`/`payment_amount`。

## Docs

- **`health-source-data-analysis.md`** — 新增 §5.7 手续费手工修正与审核规则，说明四舍五入差异、手工优先原则、差异标记与快速定位机制。
