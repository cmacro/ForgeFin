use async_trait::async_trait;
use sqlx::SqlitePool;
use crate::ledger::{Voucher, VoucherItem, VoucherStatus};
use anyhow::{Result, Context};

pub struct SqliteVoucherRepository {
    pool: SqlitePool,
}

impl SqliteVoucherRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl crate::repositories::VoucherRepository for SqliteVoucherRepository {
    async fn create_voucher(&self, voucher: &Voucher, items: &[VoucherItem]) -> Result<i64> {
        let mut tx = self.pool.begin().await.context("Failed to start transaction")?;

        let voucher_id = sqlx::query("INSERT INTO vouchers (voucher_date, description, project_id, status) VALUES (?, ?, ?, ?)")
            .bind(voucher.voucher_date.to_string())
            .bind(&voucher.description)
            .bind(voucher.project_id)
            .bind(match voucher.status {
                VoucherStatus::Unposted => "Unposted",
                VoucherStatus::Posted => "Posted",
                VoucherStatus::Locked => "Locked",
            })
            .execute(&mut *tx)
            .await
            .context("Failed to insert voucher")?
            .last_insert_rowid();

        for item in items {
            sqlx::query("INSERT INTO voucher_items (voucher_id, account_id, debit, credit, description) VALUES (?, ?, ?, ?, ?)")
                .bind(voucher_id)
                .bind(item.account_id)
                .bind(item.debit)
                .bind(item.credit)
                .bind(&item.description)
                .execute(&mut *tx)
                .await
                .context("Failed to insert voucher item")?;
        }

        tx.commit().await.context("Failed to commit transaction")?;
        Ok(voucher_id)
    }

    async fn find_by_project(&self, project_id: i64) -> Result<Vec<Voucher>> {
        let vouchers = sqlx::query_as::<_, (i64, String, Option<String>, Option<i64>, String)>(
            "SELECT id, voucher_date, description, project_id, status FROM vouchers WHERE project_id = ?"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch vouchers by project")?;

        let mut result = Vec::new();
        for v in vouchers {
            let items = sqlx::query_as::<_, VoucherItem>(
                "SELECT id, voucher_id, account_id, debit, credit, description FROM voucher_items WHERE voucher_id = ?"
            )
            .bind(v.0)
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch voucher items")?;

            result.push(Voucher {
                id: v.0,
                voucher_date: chrono::NaiveDate::parse_from_str(&v.1, "%Y-%m-%d").unwrap_or_default(),
                description: v.2,
                project_id: v.3,
                status: match v.4.as_str() {
                    "Posted" => VoucherStatus::Posted,
                    "Locked" => VoucherStatus::Locked,
                    _ => VoucherStatus::Unposted,
                },
                items,
            });
        }

        Ok(result)
    }
}
