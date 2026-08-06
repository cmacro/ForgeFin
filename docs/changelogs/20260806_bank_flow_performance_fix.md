# 20260806_bank_flow_performance_fix

## Fixed

- **银行流水页面卡死** — `fetch_bank_balance_check_rows` 全表扫描时读取 `raw_data` JSON 列并对每行执行 `serde_json::from_str` 解析，数据量大时导致页面假死。改为直接读取 `amount_in`/`amount_out` 整数分列，消除 JSON 解析开销。
- **`BankFlowBalanceBar` 无限循环** — 空数据库时 `Effect::new` 中 `set_selected_batch.set(None)` 反复触发自身，导致页面卡死。改为仅在存在实际批次时才设置值。

## Changed

- **银行流水列表查询** — `list_bank_flows_core` SQL 去掉 `LEFT JOIN import_batches` 和来源字段（`source_file_name`、`source_row_no`、`file_path`），减少 JOIN 开销和数据传输。来源信息仅在详情查询时补充。
- **订单流水列表查询** — 同上，`list_order_flows_core` 去掉 JOIN 和来源字段。
- **列默认可见性** — `source_file_name` 和 `source_row_no` 默认隐藏，来源信息在详情面板展示。

## Docs

- **README** — 新增「数据库文件位置」章节，说明各平台路径。
