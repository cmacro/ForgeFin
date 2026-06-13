use async_trait::async_trait;
use crate::ledger::{Project, Voucher, VoucherItem};
use anyhow::Result;

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create(&self, project: &Project) -> Result<i64>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Project>>;
    async fn list(&self) -> Result<Vec<Project>>;
}

#[async_trait]
pub trait VoucherRepository: Send + Sync {
    async fn create_voucher(&self, voucher: &Voucher, items: &[VoucherItem]) -> Result<i64>;
    async fn find_by_project(&self, project_id: i64) -> Result<Vec<Voucher>>;
}
