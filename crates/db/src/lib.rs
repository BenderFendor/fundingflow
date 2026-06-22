use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

pub mod queries;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;
    info!("connected to database");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("running database migrations");
    sqlx::migrate!("./migrations").run(pool).await?;
    info!("migrations complete");
    Ok(())
}

pub fn default_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://fundingflow:fundingflow@localhost:5432/fundingflow".to_string()
    })
}
