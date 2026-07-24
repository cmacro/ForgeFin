# 项目领域术语

## 数据源 (SourceType)

- **定义**：系统中可配置的原始凭证来源，每个企业可以拥有多个数据源，例如银行流水、项目成本、采购单等。
- **对应表**：`source_types`
- **对应 Rust 结构体**：`SourceType`
- **页面路径**：`src/pages/source_type_settings.rs`
- **说明**：数据源是静态的配置记录，用于在后续的动态字段、原始记录和映射规则中进行关联。

## 动态字段 (SourceField)

- **定义**：系统中对每个数据源的字段进行的元数据描述，用于在原始记录中存储 JSON 动态字段。
- **对应表**：`source_fields`
- **对应 Rust 结构体**：`SourceField`
- **页面路径**：`src/pages/source_field_settings.rs`
- **说明**：每个字段声明其代码、显示名、数据类型、是否金额、借贷方向等属性。
