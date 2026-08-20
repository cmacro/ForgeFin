# 20260820_currency_iso4217_encoding

## Fixed

- **银行流水/订单流水加载失败** — `bank_flows`/`order_flows` 表的 `currency` 列为整数，但查询代码用 `row.get::<String>` 读取，导致 `Invalid column type Integer` 报错。改为读取整数后经 `currency_code` 转换为字符串代码。

## Changed

- **币种编码改用 ISO 4217 数字编码** — 原顺序枚举（`0=CNY,1=USD,2=EUR,3=HKD,4=JPY,5=GBP`）替换为标准 ISO 4217 数字编码（https://en.wikipedia.org/wiki/ISO_4217）：
  - `156` = CNY（人民币）
  - `840` = USD（美元）
  - `978` = EUR（欧元）
  - `344` = HKD（港币）
  - `392` = JPY（日元）
  - `826` = GBP（英镑）
- **`currency_code` 转换函数** — 新增于 `src-tauri/src/commands/raw.rs`，将整数编码映射为字符串代码，未知编码回退为 `CNY`。
- **`bank_flows`/`order_flows` 表默认值** — `currency` 列默认值由 `0` 改为 `156`（CNY）。
- **导入写入逻辑** — 银行流水/订单流水入库时硬编码的 `0` 改为 `156`。

## Notes

- 已有数据库中的旧数据（`currency=0`）在 `currency_code` 中回退为 `CNY`，无需数据迁移。
