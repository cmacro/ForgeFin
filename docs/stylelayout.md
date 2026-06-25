# ForgeFin UI Structure Layout

## 1. Top Navigation Header
- **Left**: ForgeFin Logo & Application Name ("财务管理工具")
- **Center**: Breadcrumbs (e.g., `总账 / 凭证管理`)
- **Right**: 
    - Quick Access ("快捷键")
    - Help ("帮助")
    - Messages ("消息")
    - User Profile (Name, Department)

## 2. Left Sidebar (Main Navigation)
- **Home** ("首页")
- **General Ledger** ("总账") - *Expanded*
    - Voucher Management ("凭证管理") - *Active*
        - Voucher Entry ("凭证录入")
        - Voucher Audit ("凭证审核")
        - Voucher Query ("凭证查询")
    - Account Balance ("科目余额")
    - Detailed Ledger ("明细账")
    - General Ledger ("总账")
    - Trial Balance ("试算平衡表")
- **Report Center** ("报表中心")
- **Accounts Receivable** ("应收管理")
- **Accounts Payable** ("应付管理")
- **Fixed Assets** ("固定资产")
- **Cashier Management** ("出纳管理")
- **Budget Management** ("预算管理")
- **Tax Management** ("税务管理")
- **System Settings** ("系统设置")

## 3. Main Content Area (Voucher Management)

### 3.1 Tab Navigation
- Tabs for different views (e.g., `凭证概览`, `凭证管理`)

### 3.2 Filter & Search Section
- **Search Fields**: 
    - Period ("期间")
    - Voucher Number ("凭证号")
    - Voucher Type ("凭证类别")
    - Operator ("制单人")
    - Audit Status ("审核状态")
- **Actions**: Query ("查询"), Reset ("重置")
- **Toggle**: More conditions ("更多条件")

### 3.3 Summary Statistics (KPI Cards)
- Total Vouchers ("凭证总数")
- Audited Vouchers ("已审核凭证")
- Unaudited Vouchers ("未审核凭证")
- Total Debit ("借方金额")
- Total Credit ("贷方金额")
- Balance Difference ("借贷差额")

### 3.4 Data Table Action Bar
- **Primary Action**: Add Voucher ("新增凭证")
- **Batch Actions**: Filter ("筛选"), Audit ("审核"), Un-audit ("反审核")
- **Utilities**: Print ("打印"), Export ("导出")

### 3.5 Data Table
- **Columns**: 
    - Index, Voucher No, Date, Summary, Type, Debit Amount, Credit Amount, Operator, Auditor, Status, Actions.
- **Row Actions**: View ("查看"), Copy ("复制"), More ("更多")
- **Pagination**: Page controls, items per page, total count.

### 3.6 Detail Panel (Split Bottom View)
- **Voucher Header**: Type, Date, No, Attachment count, Edit button.
- **Voucher Entries Table**: 
    - Columns: Index, Account Code, Account Name, Auxiliary Accounting, Debit, Credit.
    - Footer: Total amount.
- **Voucher Footer**: Operator, Auditor, Audit Date, Audit Status, Submit button.
- **Right Side Panel (Log/Attachments)**: 
    - Tabs: Attachments, Audit Log, Operation Log.
    - Log Content: User, Status, Timestamp, Comments.
