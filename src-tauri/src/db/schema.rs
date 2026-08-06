use rusqlite::Connection;

use super::system;

/// 初始化系统库 schema,并在无用户时创建默认 admin 用户。
///
/// 系统库 `forgefin_system.db` 存放跨公司共享的注册信息:
/// - users: 用户(可跨公司)
/// - companies: 公司/账套注册
/// - user_company_permissions: 用户在某公司的权限
pub fn init_system(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS users (
            id              TEXT PRIMARY KEY,
            username        TEXT NOT NULL UNIQUE,
            display_name    TEXT NOT NULL,
            password_hash   TEXT NOT NULL,
            department      TEXT,
            is_admin        INTEGER NOT NULL DEFAULT 0,
            is_active       INTEGER NOT NULL DEFAULT 1,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS companies (
            id              TEXT PRIMARY KEY,
            name            TEXT NOT NULL,
            tax_id          TEXT,
            legal_person    TEXT,
            address         TEXT,
            phone           TEXT,
            currency        TEXT NOT NULL DEFAULT 'CNY',
            is_active       INTEGER NOT NULL DEFAULT 1,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS user_company_permissions (
            id              TEXT PRIMARY KEY,
            user_id         TEXT NOT NULL,
            company_id      TEXT NOT NULL,
            role            TEXT NOT NULL DEFAULT 'accountant',
            can_audit       INTEGER NOT NULL DEFAULT 0,
            can_post         INTEGER NOT NULL DEFAULT 0,
            can_manage      INTEGER NOT NULL DEFAULT 0,
            can_backup      INTEGER NOT NULL DEFAULT 0,
            created_at      TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
            UNIQUE (user_id, company_id)
        );

        CREATE INDEX IF NOT EXISTS idx_ucp_user
            ON user_company_permissions(user_id);
        CREATE INDEX IF NOT EXISTS idx_ucp_company
            ON user_company_permissions(company_id);
        ",
    )
    .map_err(|e| format!("系统库建表失败: {e}"))?;

    // 无用户时自动创建默认 admin 用户(开发测试用)
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))
        .map_err(|e| format!("查询用户数失败: {e}"))?;
    if count == 0 {
        system::create_user(conn, "admin", "管理员", "admin", None, true)?;
    }

    Ok(())
}

/// 初始化公司库 schema。
///
/// 每个公司库 `forgefin_company_{id}.db` 存放该公司全部业务数据:
/// - accounts: 会计科目(树形,parent_id 自引用)
/// - contacts: 客户/供应商
/// - vouchers: 凭证主表
/// - voucher_entries: 凭证分录
/// - voucher_audit_logs: 审核日志
pub fn init_company(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS accounts (
            id              TEXT PRIMARY KEY,
            code            TEXT NOT NULL,
            name            TEXT NOT NULL,
            parent_id       TEXT,
            account_type    TEXT NOT NULL,
            balance_direction TEXT NOT NULL DEFAULT 'auto',
            is_leaf         INTEGER NOT NULL DEFAULT 1,
            is_active       INTEGER NOT NULL DEFAULT 1,
            description     TEXT,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            FOREIGN KEY (parent_id) REFERENCES accounts(id) ON DELETE RESTRICT,
            UNIQUE (code)
        );

        CREATE INDEX IF NOT EXISTS idx_accounts_parent
            ON accounts(parent_id);

        CREATE TABLE IF NOT EXISTS contacts (
            id              TEXT PRIMARY KEY,
            code            TEXT NOT NULL,
            name            TEXT NOT NULL,
            contact_type    TEXT NOT NULL,
            tax_id          TEXT,
            bank_account    TEXT,
            bank_name       TEXT,
            address         TEXT,
            phone           TEXT,
            email           TEXT,
            remark          TEXT,
            is_active       INTEGER NOT NULL DEFAULT 1,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            UNIQUE (code)
        );

        CREATE INDEX IF NOT EXISTS idx_contacts_type
            ON contacts(contact_type);

        CREATE TABLE IF NOT EXISTS vouchers (
            id              TEXT PRIMARY KEY,
            voucher_no      TEXT NOT NULL,
            voucher_date    TEXT NOT NULL,
            voucher_type    TEXT NOT NULL,
            summary         TEXT NOT NULL,
            attachments     INTEGER NOT NULL DEFAULT 0,
            status          TEXT NOT NULL DEFAULT 'draft',
            debit_total     TEXT NOT NULL DEFAULT '0',
            credit_total    TEXT NOT NULL DEFAULT '0',
            operator_id     TEXT,
            operator_name   TEXT,
            auditor_id      TEXT,
            auditor_name    TEXT,
            audited_at      TEXT,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            UNIQUE (voucher_no)
        );

        CREATE INDEX IF NOT EXISTS idx_vouchers_date
            ON vouchers(voucher_date);
        CREATE INDEX IF NOT EXISTS idx_vouchers_status
            ON vouchers(status);
        CREATE INDEX IF NOT EXISTS idx_vouchers_type
            ON vouchers(voucher_type);

        CREATE TABLE IF NOT EXISTS voucher_entries (
            id              TEXT PRIMARY KEY,
            voucher_id      TEXT NOT NULL,
            line_no         INTEGER NOT NULL,
            account_id      TEXT NOT NULL,
            account_code    TEXT NOT NULL,
            account_name    TEXT NOT NULL,
            summary         TEXT,
            debit           TEXT NOT NULL DEFAULT '0',
            credit          TEXT NOT NULL DEFAULT '0',
            contact_id      TEXT,
            contact_name    TEXT,
            created_at      TEXT NOT NULL,
            FOREIGN KEY (voucher_id) REFERENCES vouchers(id) ON DELETE CASCADE,
            FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
            UNIQUE (voucher_id, line_no)
        );

        CREATE INDEX IF NOT EXISTS idx_entries_voucher
            ON voucher_entries(voucher_id);
        CREATE INDEX IF NOT EXISTS idx_entries_account
            ON voucher_entries(account_id);

        CREATE TABLE IF NOT EXISTS voucher_audit_logs (
            id              TEXT PRIMARY KEY,
            voucher_id      TEXT NOT NULL,
            action          TEXT NOT NULL,
            operator_id     TEXT,
            operator_name   TEXT,
            comment         TEXT,
            created_at      TEXT NOT NULL,
            FOREIGN KEY (voucher_id) REFERENCES vouchers(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_audit_voucher
            ON voucher_audit_logs(voucher_id);

        -- 原始凭证相关表
        CREATE TABLE IF NOT EXISTS source_types (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            code            TEXT NOT NULL UNIQUE,
            name            TEXT NOT NULL,
            category        TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS import_batches (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path       TEXT NOT NULL,
            file_name       TEXT NOT NULL,
            file_hash       TEXT NOT NULL,
            source_type     TEXT NOT NULL,
            row_count       INTEGER NOT NULL DEFAULT 0,
            imported_at     TEXT NOT NULL,
            created_by      TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_import_batches_hash
            ON import_batches(file_hash);
        CREATE INDEX IF NOT EXISTS idx_import_batches_name
            ON import_batches(file_name);

        CREATE TABLE IF NOT EXISTS source_records (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            source_type_id  INTEGER NOT NULL,
            import_batch_id INTEGER NOT NULL,
            source_file_name TEXT NOT NULL,
            source_row_no   INTEGER NOT NULL,
            record_no       TEXT,
            record_date     TEXT,
            amount_total    TEXT,
            balance         TEXT,
            currency        TEXT DEFAULT 'CNY',
            counterpart_info TEXT,
            summary         TEXT,
            raw_data        TEXT NOT NULL,
            row_hash        TEXT,
            status          TEXT NOT NULL DEFAULT 'pending',
            created_at      TEXT NOT NULL,
            balance_confirmed_at TEXT,
            FOREIGN KEY (source_type_id) REFERENCES source_types(id)
        );

        CREATE INDEX IF NOT EXISTS idx_source_records_batch
            ON source_records(import_batch_id);
        CREATE INDEX IF NOT EXISTS idx_source_records_type
            ON source_records(source_type_id);
        CREATE INDEX IF NOT EXISTS idx_source_records_date
            ON source_records(record_date);
        CREATE INDEX IF NOT EXISTS idx_source_records_row_hash
            ON source_records(source_type_id, row_hash);

        -- 行级去重:订单/POS 来源以业务单号(record_no)为主键指纹。
        -- 排除银行流水的占位符 `000000000`(分析文档 §4.1:凭证号不可用),
        -- 否则真实 POS 清算行会被大量误判为重复。
        CREATE UNIQUE INDEX IF NOT EXISTS uq_source_records_by_record_no
            ON source_records(source_type_id, record_no)
            WHERE record_no IS NOT NULL
              AND record_no != ''
              AND record_no != '000000000';

        -- 行级去重兜底:同一导入批次、同一文件、同一行号绝不重复入库。
        -- 跨批次允许同名同 row_no 的记录(便于多次重新导入同一文件)。
        CREATE UNIQUE INDEX IF NOT EXISTS uq_source_records_batch_file_row
            ON source_records(import_batch_id, source_file_name, source_row_no);

        -- 原始凭证对账汇总(按日期)
        CREATE TABLE IF NOT EXISTS transaction_summaries (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            summary_date    TEXT NOT NULL,
            source_type     TEXT NOT NULL,
            bank_amount     TEXT NOT NULL DEFAULT '0',
            order_amount    TEXT NOT NULL DEFAULT '0',
            diff_amount     TEXT NOT NULL DEFAULT '0',
            review_status   TEXT NOT NULL DEFAULT 'pending',
            voucher_id      TEXT,
            matched_bank_ids TEXT,
            matched_order_ids TEXT,
            comment         TEXT,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_transaction_summaries_date
            ON transaction_summaries(summary_date);
        CREATE INDEX IF NOT EXISTS idx_transaction_summaries_status
            ON transaction_summaries(review_status);

        -- 审计日志(原始凭证/对账/凭证生成)
        CREATE TABLE IF NOT EXISTS audit_logs (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            entity_type     TEXT NOT NULL,
            entity_id       TEXT,
            action          TEXT NOT NULL,
            old_values      TEXT,
            new_values      TEXT,
            operator_id     TEXT,
            operator_name   TEXT,
            comment         TEXT,
            created_at      TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_audit_logs_entity
            ON audit_logs(entity_type, entity_id);
        CREATE INDEX IF NOT EXISTS idx_audit_logs_action
            ON audit_logs(action);
        CREATE INDEX IF NOT EXISTS idx_audit_logs_created
            ON audit_logs(created_at);

        -- 附件
        CREATE TABLE IF NOT EXISTS attachments (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            entity_type     TEXT NOT NULL,
            entity_id       TEXT NOT NULL,
            file_name       TEXT NOT NULL,
            file_path       TEXT,
            file_size       INTEGER DEFAULT 0,
            mime_type       TEXT,
            created_by      TEXT,
            created_at      TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_attachments_entity
            ON attachments(entity_type, entity_id);

        -- 银行流水独立表(不再写入 source_records)
        -- import_batch_id 关联导入批次,方便追溯来源文件。
        -- 金额字段以分为单位(INTEGER),避免浮点精度问题。
        -- currency 为整数枚举:0=CNY,1=USD,2=EUR,3=HKD,4=JPY,5=GBP
        CREATE TABLE IF NOT EXISTS bank_flows (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            import_batch_id INTEGER NOT NULL,
            source_file_name TEXT NOT NULL,
            source_row_no   INTEGER NOT NULL,
            record_no       TEXT,
            record_date     TEXT,
            amount_in       INTEGER NOT NULL DEFAULT 0,
            amount_out      INTEGER NOT NULL DEFAULT 0,
            amount_total    INTEGER NOT NULL DEFAULT 0,
            balance         INTEGER,
            currency        INTEGER NOT NULL DEFAULT 0,
            counterpart_info TEXT,
            summary         TEXT,
            raw_data        TEXT NOT NULL,
            row_hash        TEXT,
            status          TEXT NOT NULL DEFAULT 'pending',
            created_at      TEXT NOT NULL,
            balance_confirmed_at TEXT,
            FOREIGN KEY (import_batch_id) REFERENCES import_batches(id)
        );

        CREATE INDEX IF NOT EXISTS idx_bank_flows_batch
            ON bank_flows(import_batch_id);
        CREATE INDEX IF NOT EXISTS idx_bank_flows_date
            ON bank_flows(record_date);
        CREATE INDEX IF NOT EXISTS idx_bank_flows_row_hash
            ON bank_flows(row_hash);

        -- 订单流水独立表(不再写入 source_records)
        -- import_batch_id 关联导入批次,方便追溯来源文件。
        -- 金额字段以分为单位(INTEGER)。
        -- currency 为整数枚举:0=CNY,1=USD,2=EUR,3=HKD,4=JPY,5=GBP
        CREATE TABLE IF NOT EXISTS order_flows (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            import_batch_id INTEGER NOT NULL,
            source_file_name TEXT NOT NULL,
            source_row_no   INTEGER NOT NULL,
            record_no       TEXT,
            record_date     TEXT,
            amount_total    INTEGER NOT NULL DEFAULT 0,
            currency        INTEGER NOT NULL DEFAULT 0,
            counterpart_info TEXT,
            summary         TEXT,
            raw_data        TEXT NOT NULL,
            row_hash        TEXT,
            status          TEXT NOT NULL DEFAULT 'pending',
            created_at      TEXT NOT NULL,
            FOREIGN KEY (import_batch_id) REFERENCES import_batches(id)
        );

        CREATE INDEX IF NOT EXISTS idx_order_flows_batch
            ON order_flows(import_batch_id);
        CREATE INDEX IF NOT EXISTS idx_order_flows_date
            ON order_flows(record_date);
        CREATE INDEX IF NOT EXISTS idx_order_flows_row_hash
            ON order_flows(row_hash);

        -- 数据汇总独立表(人工整理的凭证草稿,对应 数据汇总.tsv)
        -- 金额字段以元为单位(TEXT),保留原始精度。
        -- payment_method: debit_card / credit_card / wechat / alipay
        -- source_info: JSON,记录生成来源追溯信息
        CREATE TABLE IF NOT EXISTS data_summaries (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            summary_date    TEXT NOT NULL,
            receipt_no      TEXT,
            category        TEXT NOT NULL,
            project         TEXT NOT NULL,
            reason          TEXT,
            payment_method  TEXT,
            payment_amount  TEXT DEFAULT '0',
            fee             TEXT DEFAULT '0',
            actual_income   TEXT DEFAULT '0',
            expense         TEXT DEFAULT '0',
            balance         TEXT,
            remarks         TEXT,
            source_info     TEXT,
            voucher_id      TEXT,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_data_summaries_date
            ON data_summaries(summary_date);
        CREATE INDEX IF NOT EXISTS idx_data_summaries_category
            ON data_summaries(category);

        -- 企业通用手续费率表
        -- payment_method: debit_card / credit_card / wechat / alipay
        -- rate: 费率(百分比,如 0.5 表示 0.5%)
        CREATE TABLE IF NOT EXISTS fee_rates (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            payment_method  TEXT NOT NULL,
            rate            TEXT NOT NULL,
            description     TEXT,
            is_active       INTEGER NOT NULL DEFAULT 1,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            UNIQUE (payment_method)
        );

        -- 导入错误明细
        CREATE TABLE IF NOT EXISTS import_errors (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            import_batch_id INTEGER NOT NULL,
            source_row_no   INTEGER,
            field_name      TEXT,
            field_value     TEXT,
            error_message   TEXT NOT NULL,
            created_at      TEXT NOT NULL,
            FOREIGN KEY (import_batch_id) REFERENCES import_batches(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_import_errors_batch
            ON import_errors(import_batch_id);

        -- UI 列显示偏好(账套 × 来源类型)
        -- 每条记录表示某来源类型(银行流水/订单流水/POS流水/数据汇总)下,
        -- 某个列的可见性。column_key 稳定标识列名,visible=0/1。
        -- 单一账套 + 来源类型下,各 column_key 唯一(UNIQUE)。
        CREATE TABLE IF NOT EXISTS ui_column_prefs (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            source_type     TEXT NOT NULL,
            column_key      TEXT NOT NULL,
            visible         INTEGER NOT NULL DEFAULT 1,
            updated_at      TEXT NOT NULL,
            UNIQUE (source_type, column_key)
        );

        CREATE INDEX IF NOT EXISTS idx_ui_column_prefs_source
            ON ui_column_prefs(source_type);
        ",
    )
    .map_err(|e| format!("公司库建表失败: {e}"))?;

    // 轻量迁移:为旧库补齐 source_records.row_hash 列
    let has_row_hash: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('source_records') WHERE name = 'row_hash'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);
    if !has_row_hash {
        conn.execute_batch("ALTER TABLE source_records ADD COLUMN row_hash TEXT;")
            .map_err(|e| format!("为 source_records 添加 row_hash 列失败: {e}"))?;
    }

    // 轻量迁移:为旧库补齐 source_records.balance 列(银行流水余额)
    let has_balance: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('source_records') WHERE name = 'balance'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);
    if !has_balance {
        conn.execute_batch("ALTER TABLE source_records ADD COLUMN balance TEXT;")
            .map_err(|e| format!("为 source_records 添加 balance 列失败: {e}"))?;
    }

    // 轻量迁移:为旧库补齐 source_records.balance_confirmed_at 列(银行流水余额连续性人工确认)
    let has_balance_confirmed_at: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('source_records') WHERE name = 'balance_confirmed_at'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);
    if !has_balance_confirmed_at {
        conn.execute_batch("ALTER TABLE source_records ADD COLUMN balance_confirmed_at TEXT;")
            .map_err(|e| format!("为 source_records 添加 balance_confirmed_at 列失败: {e}"))?;
    }

    // 初始化默认原始凭证来源类型
    init_source_types(conn)?;

    // 初始化默认手续费率
    init_fee_rates(conn)?;

    Ok(())
}

fn init_source_types(conn: &Connection) -> Result<(), String> {
    let types = [
        ("bank_flow", "银行流水", "bank"),
        ("order_flow", "订单流水", "order"),
        ("pos_flow", "POS流水", "pos"),
        ("summary_flow", "数据汇总", "summary"),
    ];
    for (code, name, category) in types {
        conn.execute(
            "INSERT OR IGNORE INTO source_types (code, name, category) VALUES (?1, ?2, ?3)",
            rusqlite::params![code, name, category],
        )
        .map_err(|e| format!("初始化来源类型失败: {e}"))?;
    }
    Ok(())
}

fn init_fee_rates(conn: &Connection) -> Result<(), String> {
    let rates = [
        ("debit_card", "0.5", "借记卡"),
        ("credit_card", "0.6", "信用卡(贷记卡)"),
        ("wechat", "0.25", "微信支付"),
        ("alipay", "0.25", "支付宝"),
    ];
    for (method, rate, desc) in rates {
        conn.execute(
            "INSERT OR IGNORE INTO fee_rates (payment_method, rate, description, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, 1, datetime('now'), datetime('now'))",
            rusqlite::params![method, rate, desc],
        )
        .map_err(|e| format!("初始化手续费率失败: {e}"))?;
    }
    Ok(())
}
