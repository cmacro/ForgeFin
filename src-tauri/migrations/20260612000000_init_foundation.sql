-- Create projects table
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create accounts table (Chart of Accounts)
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    account_type TEXT CHECK (account_type IN ('Asset', 'Liability', 'Equity', 'Revenue', 'Expense')) NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create vouchers table
CREATE TABLE vouchers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    voucher_date DATE NOT NULL,
    description TEXT,
    project_id INTEGER,
    status TEXT CHECK (status IN ('Unposted', 'Posted', 'Locked')) DEFAULT 'Unposted',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- Create voucher items table
CREATE TABLE voucher_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    voucher_id INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    debit DECIMAL(19, 4) NOT NULL DEFAULT 0,
    credit DECIMAL(19, 4) NOT NULL DEFAULT 0,
    description TEXT,
    FOREIGN KEY (voucher_id) REFERENCES vouchers(id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

-- Indexes for performance
CREATE INDEX idx_vouchers_project ON vouchers(project_id);
CREATE INDEX idx_voucher_items_voucher ON voucher_items(voucher_id);
