use sqlx::pool::PoolOptions;
use sqlx::postgres::PgPool;

pub async fn create_connection_pool(database_url: &str) -> PgPool {
    PoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
        .expect("Failed to connect to the database")
}
