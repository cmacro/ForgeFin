pub mod ledger;
pub mod repositories;

#[cfg(test)]
mod tests {
    use super::ledger::*;
    
    #[test]
    fn test_voucher_is_balanced() {
        let items = vec![
            VoucherItem {
                id: 1,
                voucher_id: 1,
                account_id: 101,
                debit: 1000.0,
                credit: 0.0,
                description: Some("Cash".to_string()),
            },
            VoucherItem {
                id: 2,
                voucher_id: 1,
                account_id: 401,
                debit: 0.0,
                credit: 1000.0,
                description: Some("Revenue".to_string()),
            },
        ];

        let voucher = Voucher {
            id: 1,
            voucher_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 12).unwrap(),
            description: Some("Test balanced".to_string()),
            project_id: None,
            status: VoucherStatus::Unposted,
            items,
        };

        assert!(voucher.is_balanced());
    }

    #[test]
    fn test_voucher_not_balanced() {
        let items = vec![
            VoucherItem {
                id: 1,
                voucher_id: 1,
                account_id: 101,
                debit: 1000.0,
                credit: 0.0,
                description: Some("Cash".to_string()),
            },
            VoucherItem {
                id: 2,
                voucher_id: 1,
                account_id: 401,
                debit: 0.0,
                credit: 500.0,
                description: Some("Revenue".to_string()),
            },
        ];

        let voucher = Voucher {
            id: 1,
            voucher_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 12).unwrap(),
            description: Some("Test not balanced".to_string()),
            project_id: None,
            status: VoucherStatus::Unposted,
            items,
        };

        assert!(!voucher.is_balanced());
    }
}
