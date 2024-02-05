use sqlx::pool::PoolOptions;
use sqlx::postgres::PgPool;
use crate::config::get_config;

pub async fn create_connection_pool() -> PgPool {
    let database_url = get_config().database_url;

    PoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await.expect("Failed to connect to the database")
}