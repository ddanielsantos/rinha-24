use axum::response::IntoResponse;
use axum::Router;
use axum::routing::{get, post};
use crate::state::AppState;

async fn transactions_handler() -> impl IntoResponse {}

async fn extract_handler() -> impl IntoResponse {}

pub fn client_routes() -> Router<AppState> {
     Router::new()
         .route("/:id/transacoes", post(transactions_handler))
         .route("/:id/extrato", get(extract_handler))
}