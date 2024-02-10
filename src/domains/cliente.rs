use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::extract::Path;
use axum::routing::{get, post};
use sqlx::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use crate::state::AppState;

#[derive(Debug, serde::Serialize)]
pub struct Client {
     pub id: i32,
     pub name: String,
     pub credit_limit: i32,
     pub balance: i32,
}

#[derive(serde::Deserialize)]
struct TransactionRequest {
     #[serde(rename = "valor")]
     value: i32,
     #[serde(rename = "tipo")]
     r#type: String,
     #[serde(rename = "descricao")]
     description: String
}

#[derive(serde::Serialize)]
struct TransactionResponse {
     #[serde(rename = "limite")]
     limit: i32,
     #[serde(rename = "saldo")]
     balance: i32,
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
               match body.r#type.as_str() {
                    "c" => {
                         let balance = c.balance + body.value;

                         let _ = sqlx::query!("update clients set balance = $1 where id = $2", balance, id)
                             .execute(&state.db)
                             .await;

                         let _ = sqlx::query!("insert into transactions (client_id, value, type, description, created_at) values ($1, $2, $3, $4, now())", c.id, body.value, body.r#type, body.description)
                             .execute(&state.db)
                             .await;

                         Json(TransactionResponse {
                              limit: c.credit_limit,
                              balance
                         }).into_response()
                    }
                    "d" => {
                         let new_balance = c.balance - body.value;

                         if new_balance < (c.credit_limit * -1) {
                              return StatusCode::UNPROCESSABLE_ENTITY.into_response()
                         }

                         let _ = sqlx::query!("update clients set balance = $1 where id = $2", new_balance, id)
                             .execute(&state.db)
                             .await;

                         let _ = sqlx::query!("insert into transactions (client_id, value, type, description, created_at) values ($1, $2, $3, $4, now())", c.id, body.value, body.r#type, body.description)
                             .execute(&state.db)
                             .await;

                         Json(TransactionResponse {
                              limit: c.credit_limit,
                              balance: new_balance
                         }).into_response()
                    }
                    _ => {
                         StatusCode::BAD_REQUEST.into_response()
                    }
               }
          }
     }
}

#[serde_with::serde_as]
#[derive(serde::Serialize)]
struct BalanceExtract {
     total: i32,
     #[serde(rename = "limite")]
     limit: i32,
     #[serde_as(as = "Rfc3339")]
     #[serde(rename = "data_extrato")]
     queryed_at: OffsetDateTime
}

#[serde_with::serde_as]
#[derive(serde::Serialize)]
struct Transaction {
     #[serde(rename = "valor")]
     value: i32,
     #[serde(rename = "tipo")]
     r#type: String,
     #[serde(rename = "descricao")]
     description: String,
     #[serde_as(as = "Rfc3339")]
     #[serde(rename = "realizada_em")]
     created_at: OffsetDateTime
}

#[derive(serde::Serialize)]
struct ExtractResponse {
     #[serde(rename = "saldo")]
     balance: BalanceExtract,
     #[serde(rename = "ultimas_transacoes")]
     last_transactions: Vec<Transaction>
}

async fn extract_handler(
     Path(id): Path<i32>,
     State(state): State<AppState>,
) -> impl IntoResponse {
     let client = sqlx::query_as!(Client, "select * from clients c where c.id = $1", id)
         .fetch_one(&state.db)
         .await;

     match client {
          Err(_) => {
               StatusCode::NOT_FOUND.into_response()
          }
          Ok(c) => {
               let balance = BalanceExtract {
                    queryed_at: OffsetDateTime::now_utc(),
                    total: c.balance,
                    limit: c.credit_limit
               };

               let txs = sqlx::query_as!(Transaction, "select value, type, description, created_at from transactions where client_id = $1 order by created_at desc limit 10", id)
                   .fetch_all(&state.db)
                   .await;

               match txs {
                    Ok(txs) => {
                         let res = ExtractResponse {
                              balance,
                              last_transactions: txs,
                         };

                         Json(res).into_response()
                    }
                    Err(_) => {
                         todo!()
                    }
               }
          }
     }
}

pub fn client_routes() -> Router<AppState> {
     Router::new()
         .route("/:id/transacoes", post(transactions_handler))
         .route("/:id/extrato", get(extract_handler))
}