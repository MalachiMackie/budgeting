use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    Json,
};
use http::StatusCode;
use sqlx::MySqlPool;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    db::{self, DbError},
    models::{CreateTransactionRequest, Transaction},
    AppError,
};

#[derive(OpenApi)]
#[openapi(
    paths(get_transactions, create_transaction),
    components(schemas(Transaction, CreateTransactionRequest))
)]
pub struct TransactionApi;

const API_TAG: &str = "Transactions";

#[utoipa::path(
    get,
    path = "/api/bank-accounts/{bankAccountId}/transactions",
    responses(
        (status = OK, description = "Success", body = Box<[Transaction]>, content_type = "application/json")
    ),
    params(
        ("bankAccountId" = Uuid, Path,)
    ),
    tag = API_TAG
)]
pub async fn get_transactions(
    State(db_pool): State<MySqlPool>,
    Path(bank_account_id): Path<Uuid>,
) -> Result<Json<Box<[Transaction]>>, AppError> {
    if bank_account_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("Bank account id must be set")));
    }

    db::transactions::get_transactions(&db_pool, bank_account_id)
        .await
        .map(Json)
        .map_err(|e| e.to_app_error(anyhow!("Could not get transactions")))
}

#[utoipa::path(
    post,
    path = "/api/bank-accounts/{bankAccountId}/transactions",
    responses(
        (status = CREATED, description = "Success", body = Uuid, content_type = "application/json")
    ),
    request_body = CreateTransactionRequest,
    params(
        ("bankAccountId" = Uuid, Path,)
    ),
    tag = API_TAG
)]
pub async fn create_transaction(
    State(db_pool): State<MySqlPool>,
    Path(bank_account_id): Path<Uuid>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<Uuid>), AppError> {
    if request.payee_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("Payee Id must be set")));
    }

    if bank_account_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("Bank Account Id must be set")));
    }

    let id = Uuid::new_v4();

    let payee_result = db::payees::get_payee(&db_pool, request.payee_id).await;

    match payee_result {
        Ok(_) => (),
        Err(DbError::NotFound) => {
            return Err(AppError::NotFound(anyhow::anyhow!("Payee not found with id {}", request.payee_id)))
        }
        Err(e) => return Err(e.to_app_error(anyhow!("Could not create transaction"))),
    }

    db::transactions::create_transaction(&db_pool, id, bank_account_id, request)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Could not create transaction")))?;

    Ok((StatusCode::CREATED, Json(id)))
}
