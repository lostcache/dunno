use crate::db::surreal::schema::TABLES;
use crate::db::surreal::DB;

impl DB {
    /// Deletes all records from all tables.
    pub async fn purge_database(&self) -> anyhow::Result<()> {
        for table in TABLES {
            let sql = format!("DELETE {}", table);
            self.client.query(&sql).await?;
        }
        Ok(())
    }
}
