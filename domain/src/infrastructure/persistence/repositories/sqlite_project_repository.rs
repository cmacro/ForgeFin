use async_trait::async_trait;
use sqlx::SqlitePool;
use crate::ledger::Project;
use anyhow::{Result, Context};

pub struct SqliteProjectRepository {
    pool: SqlitePool,
}

impl SqliteProjectRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl crate::repositories::ProjectRepository for SqliteProjectRepository {
    async fn create(&self, project: &Project) -> Result<i64> {
        let result = sqlx::query("INSERT INTO projects (code, name) VALUES (?, ?)")
            .bind(&project.code)
            .bind(&project.name)
            .execute(&self.pool)
            .await
            .context("Failed to create project")?;

        Ok(result.last_insert_rowid())
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Project>> {
        let row = sqlx::query_as::<_, Project>(
            "SELECT id, code, name, created_at as \"created_at: DateTime<Utc>\" FROM projects WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find project by id")?;

        Ok(row)
    }

    async fn list(&self) -> Result<Vec<Project>> {
        let rows = sqlx::query_as::<_, Project>(
            "SELECT id, code, name, created_at as \"created_at: DateTime<Utc>\" FROM projects"
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list projects")?;

        Ok(rows)
    }
}
