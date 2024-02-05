use tokio::net::TcpListener;
use crate::state::AppState;

mod state;
mod domains;
mod api;
mod config;
mod db;

#[tokio::main]
async fn main() {
    let pool = db::create_connection_pool().await;

    sqlx::migrate!().run(&pool).await.expect("Failed to migrate database");
    let state = AppState::new(pool);

    let api = api::routes()
        .with_state(state);

    let tcp_listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(tcp_listener, api).await.unwrap();
}