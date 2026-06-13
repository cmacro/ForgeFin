use async_trait::async_trait;
use crate::ledger::{Project, Voucher, VoucherItem, VoucherStatus};
use crate::repositories::{ProjectRepository, VoucherRepository};
use anyhow::Result;

pub struct ProjectService {
    project_repo: Box<dyn ProjectRepository>,
    voucher_repo: Box<dyn VoucherRepository>,
}

impl ProjectService {
    pub fn new(project_repo: Box<dyn ProjectRepository>, voucher_repo: Box<dyn VoucherRepository>) -> Self {
        Self { project_repo, voucher_repo }
    }

    pub async fn create_project(&self, code: String, name: String) -> Result<i64> {
        let project = Project {
            id: 0,
            code,
            name,
            created_at: chrono::Utc::now(),
        };
        self.project_repo.create(&project).await
    }

    pub async fn record_transaction(
        &self,
        project_id: i64,
        description: String,
        items: Vec<(i64, f64, f64)>, // (account_id, debit, credit)
    ) -> Result<i64> {
        let voucher = Voucher {
            id: 0,
            voucher_date: chrono::Utc::now().date_naive(),
            description: Some(description),
            project_id: Some(project_id),
            status: VoucherStatus::Unposted,
            items: vec![],
        };

        let voucher_items: Vec<VoucherItem> = items.into_iter().map(|(acc_id, deb, cre)| {
            VoucherItem {
                id: 0,
                voucher_id: 0,
                account_id: acc_id,
                debit: deb,
                credit: cre,
                description: None,
            }
        }).collect();

        let mut check_voucher = voucher.clone();
        check_voucher.items = voucher_items.clone();
        if !check_voucher.is_balanced() {
            return Err(anyhow::anyhow!("Voucher must be balanced"));
        }

        self.voucher_repo.create_voucher(&voucher, &voucher_items).await
    }
}
