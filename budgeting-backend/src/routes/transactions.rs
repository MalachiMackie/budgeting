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
    db::{self, Error},
    models::{CreateTransactionRequest, Transaction, UpdateTransactionRequest},
    AppError,
};

#[derive(OpenApi)]
#[openapi(
    paths(get, create, update, delete),
    components(schemas(Transaction, CreateTransactionRequest, UpdateTransactionRequest))
)]
pub struct Api;

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
pub async fn get(
    State(db_pool): State<MySqlPool>,
    Path(bank_account_id): Path<Uuid>,
) -> Result<Json<Box<[Transaction]>>, AppError> {
    if bank_account_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("Bank account id must be set")));
    }

    db::transactions::get(&db_pool, bank_account_id)
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
pub async fn create(
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

    let payee_result = db::payees::get_single(&db_pool, request.payee_id).await;

    match payee_result {
        Ok(_) => (),
        Err(Error::NotFound) => {
            return Err(AppError::NotFound(anyhow::anyhow!(
                "Payee not found with id {}",
                request.payee_id
            )))
        }
        Err(e) => return Err(e.to_app_error(anyhow!("Could not create transaction"))),
    }

    db::transactions::create(&db_pool, id, bank_account_id, request)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Could not create transaction")))?;

    Ok((StatusCode::CREATED, Json(id)))
}

#[utoipa::path(
    put,
    path = "/api/transactions/{transactionId}",
    responses(
        (status = OK, description = "Success",)
    ),
    request_body = UpdateTransactionRequest,
    params(
        ("bankAccountId" = Uuid, Path,),
        ("transactionId" = Uuid, Path,)
    ),
    tag = API_TAG)]
pub async fn update(
    State(db_pool): State<MySqlPool>,
    Path(transaction_id): Path<Uuid>,
    Json(request): Json<UpdateTransactionRequest>,
) -> Result<(), AppError> {
    let transaction = db::transactions::get_single(&db_pool, transaction_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to update transaction")))?;

    db::transactions::update(
        &db_pool,
        Transaction::new(
            transaction.id,
            request.payee_id,
            request.date,
            request.amount,
            transaction.bank_account_id,
            request.budget_id,
        ),
    )
    .await
    .map_err(|e| e.to_app_error(anyhow!("Failed to update transaction")))?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "/api/transactions/{transactionId}",
    responses(
        (status = OK, description = "Success",)
    ),
    params(
        ("transactionId" = Uuid, Path,)
    ),
    tag = API_TAG
)]
pub async fn delete(
    State(db_pool): State<MySqlPool>,
    Path(transaction_id): Path<Uuid>,
) -> Result<(), AppError> {
    db::transactions::delete(&db_pool, transaction_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to delete transaction")))?;

    Ok(())
}
