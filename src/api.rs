use axum::Router;

use crate::domains::client;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().nest("/clientes", client::client_routes())
}
