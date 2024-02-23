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
struct Client {
     id: i32,
     name: String,
     credit_limit: i32,
     balance: i32,
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

     if body.description.len() > 10 {
          return StatusCode::UNPROCESSABLE_ENTITY.into_response();
     }

     match client {
          Err(_) => {
               StatusCode::NOT_FOUND.into_response()
          }
          Ok(c) => {
               match body.r#type.as_str() {
                    "c" => {
                         let balance = sqlx::query!("select new_balance from alter_balance($1, $2, $3, $4)", c.id, body.value, body.r#type, body.description)
                             .fetch_one(&state.db)
                             .await
                             .expect("handle error idk just unwrap this thing")
                             .new_balance
                             .expect("unwrapping again ?");

                         Json(TransactionResponse {
                              limit: c.credit_limit,
                              balance
                         }).into_response()
                    }
                    "d" => {
                         let balance = sqlx::query!("select * from alter_balance($1, $2, $3, $4)", c.id, body.value, body.r#type, body.description)
                             .fetch_one(&state.db)
                             .await
                             .expect("handle error idk just unwrap this thing");

                         if balance.success == Some(-2) {
                              return StatusCode::UNPROCESSABLE_ENTITY.into_response();
                         }

                         let balance = balance.new_balance.unwrap();

                         Json(TransactionResponse {
                              limit: c.credit_limit,
                              balance
                         }).into_response()
                    }
                    _ => {
                         StatusCode::UNPROCESSABLE_ENTITY.into_response()
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