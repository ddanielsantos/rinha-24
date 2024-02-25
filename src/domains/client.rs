use crate::state::AppState;
use anyhow::bail;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

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
    description: String,
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
    axum::Json(body): axum::Json<TransactionRequest>,
) -> impl IntoResponse {
    if !(1..=5).contains(&id) {
        return StatusCode::NOT_FOUND.into_response();
    }

    if body.description.len() > 10 || body.description.is_empty() {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    }

    match body.r#type.as_str() {
        "c" | "d" => {
            let mut transaction = state.db.begin().await.unwrap();
            let result = alter_balance(&body, &mut transaction, id).await;

            match result {
                Ok((credit_limit, new_balance)) => {
                    transaction.commit().await.unwrap();

                    axum::Json(TransactionResponse {
                        limit: credit_limit,
                        balance: new_balance,
                    })
                    .into_response()
                }
                Err(_) => {
                    transaction.rollback().await.unwrap();
                    StatusCode::UNPROCESSABLE_ENTITY.into_response()
                }
            }
        }
        _ => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
    }
}

async fn alter_balance(
    body: &TransactionRequest,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: i32,
) -> anyhow::Result<(i32, i32)> {
    let cl = sqlx::query!(
        r#"select balance, credit_limit from clients where id = $1"#,
        id
    )
    .fetch_one(&mut **transaction)
    .await?;

    let mut new_balance = cl.balance;

    if body.r#type == "d" {
        new_balance -= body.value;

        if new_balance < -cl.credit_limit {
            bail!("Balance cannot be lower than credit limit");
        }
    } else {
        new_balance += body.value;
    }

    let _ = sqlx::query!(
        "update clients set balance = $1 where id = $2",
        new_balance,
        id
    )
    .execute(&mut **transaction)
    .await?;

    let _ = sqlx::query!("insert into transactions (client_id, value, type, description, created_at) values ($1, $2, $3, $4, now())", id, body.value, body.r#type, body.description)
    .execute(&mut **transaction)
    .await?;

    Ok((cl.credit_limit, new_balance))
}

#[serde_with::serde_as]
#[derive(serde::Serialize)]
struct BalanceExtract {
    total: i32,
    #[serde(rename = "limite")]
    limit: i32,
    #[serde_as(as = "Rfc3339")]
    #[serde(rename = "data_extrato")]
    queryed_at: OffsetDateTime,
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
    created_at: OffsetDateTime,
}

#[derive(serde::Serialize)]
struct ExtractResponse {
    #[serde(rename = "saldo")]
    balance: BalanceExtract,
    #[serde(rename = "ultimas_transacoes")]
    last_transactions: Vec<Transaction>,
}

async fn extract_handler(Path(id): Path<i32>, State(state): State<AppState>) -> impl IntoResponse {
    if !(1..=5).contains(&id) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let mut transaction = state.db.begin().await.unwrap();

    let res = get_extract(id, &mut transaction).await;

    transaction.commit().await.unwrap();

    axum::Json(res).into_response()
}

/// todo: use join instead of this, but im too lazy to fight against sqlx
async fn get_extract(
    id: i32,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> ExtractResponse {
    let balance = sqlx::query_as!(BalanceExtract, r#"select credit_limit as "limit", balance as "total", now() as "queryed_at!" from clients c where c.id = $1"#, id)
        .fetch_one(&mut **transaction)
        .await
        .unwrap();

    let last_transactions = sqlx::query_as!(Transaction, "select value, type, description, created_at from transactions where client_id = $1 order by created_at desc limit 10", id)
         .fetch_all(&mut **transaction)
         .await
         .unwrap();

    ExtractResponse {
        balance,
        last_transactions,
    }
}

pub fn client_routes() -> Router<AppState> {
    Router::new()
        .route("/:id/transacoes", post(transactions_handler))
        .route("/:id/extrato", get(extract_handler))
}
