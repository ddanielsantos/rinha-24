use axum::Router;

use crate::domains::cliente;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/clientes", cliente::client_routes())
}