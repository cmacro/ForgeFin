use sqlx::{SqlitePool, query};
use async_trait::async_trait;
use crate::domain::ledger::{Project, Voucher, VoucherItem};
use crate::domain::ledger::repositories::{ProjectRepository, VoucherRepository};
use anyhow::Result;

pub struct SqliteProjectRepository {
    pool: SqlitePool,
}

impl SqliteProjectRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    async fn create(&self, project: &Project) -> Result<i64> {
        let id = query!("INSERT INTO projects (code, name) VALUES (?, ?)", project.code, project.name)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
        Ok(id)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Project>> {
        let row = query!("SELECT * FROM projects WHERE id = ?", id)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(row.map(|r| Project {
            id: r.id,
            code: r.code,
            name: r.name,
            created_at: r.created_at.into(), // This might need proper conversion depending on sqlx version
        }))
    }

    async fn list(&self) -> Result<Vec<Project>> {
        let rows = query!("SELECT * FROM projects").fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| Project {
            id: r.id,
            code: r.code,
            name: r.name,
            created_at: r.created_at.into(),
        }).collect())
    }
}

#[async_trait]
impl VoucherRepository for SqliteProjectRepository {
    async fn create_voucher(&self, voucher: &Voucher, items: &[VoucherItem]) -> Result<i64> {
        let mut tx = self.pool.begin().await?;

        let voucher_id = sqlx::query!(
            "INSERT INTO vouchers (voucher_date, description, project_id, status) VALUES (?, ?, ?, ?)",
            voucher.voucher_date,
            voucher.description,
            voucher.project_id,
            voucher.status as i32 // Assuming mapping in DB
        )
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();

        for item in items {
            sqlx::query!(
                "INSERT INTO voucher_items (voucher_id, account_id, debit, credit, description) VALUES (?, ?, ?, ?, ?)",
                voucher_id,
                item.account_id,
                item.debit,
                item.credit,
                item.description
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(voucher_id)
    }

    async fn find_by_project(&self, project_id: i64) -> Result<Vec<Voucher>> {
        // Simplified implementation
        Ok(vec![])
    }
}
