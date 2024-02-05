use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::extract::Path;
use axum::routing::{get, post};
use crate::state::AppState;

#[derive(Debug, serde::Serialize)]
pub struct Client {
     pub id: i32,
     pub nome: String,
     pub limite: i32,
     pub saldo: i32,
}

#[derive(serde::Deserialize)]
struct TransactionRequest {
     valor: i32,
     tipo: String,
     descricao: String
}

#[derive(serde::Serialize)]
struct TransactionResponse {
     limite: i32,
     saldo: i32,
}

async fn transactions_handler(
     Path(id): Path<i32>,
     State(state): State<AppState>,
     Json(body): Json<TransactionRequest>,
) -> impl IntoResponse {
     let client = sqlx::query_as!(Client, "select * from clients c where c.id = $1", id)
         .fetch_one(&state.db)
         .await;

     match client {
          Err(_) => {
               StatusCode::NOT_FOUND.into_response()
          }
          Ok(c) => {
               match body.tipo.as_str() {
                    "c" => {
                         let saldo = c.saldo + body.valor;

                         let _ = sqlx::query!("update clients set saldo = $1 where id = $2", saldo, id)
                             .execute(&state.db)
                             .await;

                         Json(TransactionResponse {
                              limite: c.limite,
                              saldo
                         }).into_response()
                    }
                    "d" => {
                         let new_saldo = c.saldo - body.valor;

                         if new_saldo < (c.limite * -1) {
                              return StatusCode::UNPROCESSABLE_ENTITY.into_response()
                         }

                         let _ = sqlx::query!("update clients set saldo = $1 where id = $2", new_saldo, id)
                             .execute(&state.db)
                             .await;

                         Json(TransactionResponse {
                              limite: c.limite,
                              saldo: new_saldo
                         }).into_response()
                    }
                    _ => {
                         StatusCode::BAD_REQUEST.into_response()
                    }
               }
          }
     }
}

async fn extract_handler() -> impl IntoResponse {}

pub fn client_routes() -> Router<AppState> {
     Router::new()
         .route("/:id/transacoes", post(transactions_handler))
         .route("/:id/extrato", get(extract_handler))
}