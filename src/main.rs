use tokio::net::TcpListener;
use crate::config::Config;
use crate::state::AppState;

mod state;
mod domains;
mod api;
mod config;
mod db;

#[tokio::main]
async fn main() {
    let config = Config::default();
    let pool = db::create_connection_pool(&config.database_url).await;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    let state = AppState::new(pool);

    let api = api::routes()
        .with_state(state);

    let tcp_listener = TcpListener::bind(&config.address)
        .await
        .unwrap();

    axum::serve(tcp_listener, api).await.unwrap();
}