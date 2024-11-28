use anyhow::anyhow;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use http::StatusCode;
use serde::Deserialize;
use sqlx::MySqlPool;
use utoipa::{IntoParams, OpenApi};
use uuid::Uuid;

use crate::{
    db,
    models::{BankAccount, CreateBankAccountRequest, UpdateBankAccountRequest},
    AppError,
};

#[derive(OpenApi)]
#[openapi(
    paths(get, get_single, create, delete, update),
    components(schemas(BankAccount, CreateBankAccountRequest, UpdateBankAccountRequest))
)]
pub struct Api;

const API_TAG: &str = "BankAccounts";

#[utoipa::path(
    post,
    path = "/api/bank-accounts",
    responses(
        (status = CREATED, description = "Success", body = Uuid, content_type = "application/json")
    ),
    request_body(content_type = "application/json", content = CreateBankAccountRequest),
    tag = API_TAG,
    operation_id = "createBankAccount"
)]
pub async fn create(
    State(db_pool): State<MySqlPool>,
    Json(request): Json<CreateBankAccountRequest>,
) -> Result<(StatusCode, Json<Uuid>), AppError> {
    if request.name.trim().is_empty() {
        return Err(AppError::BadRequest(anyhow!("Name must not be empty")));
    }

    if request.user_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("User Id must be set")));
    }

    let id = Uuid::new_v4();

    db::bank_accounts::create(&db_pool, id, request)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Could not create bank account")))?;

    Ok((StatusCode::CREATED, Json(id)))
}

#[derive(Deserialize, IntoParams)]
pub struct GetBankAccountsQuery {
    pub user_id: Uuid,
}

#[utoipa::path(
    get,
    path = "/api/bank-accounts",
    responses(
        (status = OK, description = "Success", body = Box<[BankAccount]>, content_type = "application/json")
    ),
    params(
        GetBankAccountsQuery,
    ),
    tag = API_TAG,
    operation_id = "getBankAccounts"
)]
pub async fn get(
    Query(query): Query<GetBankAccountsQuery>,
    State(db_pool): State<MySqlPool>,
) -> Result<Json<Box<[BankAccount]>>, AppError> {
    // todo: validate user_id exists
    db::bank_accounts::get(&db_pool, query.user_id)
        .await
        .map(Json)
        .map_err(|e| e.to_app_error(anyhow!("Could not get bank accounts")))
}

#[derive(Deserialize, IntoParams)]
pub struct GetBankAccountQuery {
    user_id: Uuid,
}

#[derive(Deserialize, IntoParams)]
pub struct DeleteBankAccountQuery {
    user_id: Uuid,
}

#[derive(Deserialize, IntoParams)]
pub struct UpdateBankAccountQuery {
    user_id: Uuid,
}

#[utoipa::path(
    get,
    path = "/api/bank-accounts/{accountId}",
    responses(
        (status = OK, description = "Success", body = BankAccount, content_type = "application/json")
    ),
    params(
        ("accountId" = Uuid, Path,),
        GetBankAccountQuery,
    ),
    tag = API_TAG,
    operation_id = "getBankAccount"
)]
pub async fn get_single(
    Query(query): Query<GetBankAccountQuery>,
    State(db_pool): State<MySqlPool>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<BankAccount>, AppError> {
    if account_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!(
            "{account_id} is not a valid bank account id"
        )));
    }
    if query.user_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("user_id must be set")));
    }

    db::bank_accounts::get_single(&db_pool, account_id, query.user_id)
        .await
        .map(Json)
        .map_err(|e| e.to_app_error(anyhow!("Could not get bank_account with id {account_id}")))
}

#[utoipa::path(
    delete,
    path = "/api/bank-accounts/{accountId}",
    responses(
        (status = OK, description = "Success",)
    ),
    params(
        ("accountId" = Uuid, Path,),
        DeleteBankAccountQuery,
    ),
    tag = API_TAG,
    operation_id = "deleteBankAccount"
)]
pub async fn delete(
    State(db_pool): State<MySqlPool>,
    Path(account_id): Path<Uuid>,
    Query(DeleteBankAccountQuery { user_id }): Query<DeleteBankAccountQuery>,
) -> Result<(), AppError> {
    db::bank_accounts::get_single(&db_pool, account_id, user_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to find bank account")))?;

    db::bank_accounts::delete(&db_pool, account_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to delete bank account")))?;

    Ok(())
}

#[utoipa::path(
    put,
    path = "/api/bank-accounts/{accountId}",
    responses(
        (status = OK, description = "Success",)
    ),
    params(
        ("accountId" = Uuid, Path,),
        UpdateBankAccountQuery,
    ),
    tag = API_TAG,
    operation_id = "updateBankAccount"
)]
pub async fn update(
    State(db_pool): State<MySqlPool>,
    Path(account_id): Path<Uuid>,
    Query(UpdateBankAccountQuery { user_id }): Query<UpdateBankAccountQuery>,
    Json(request): Json<UpdateBankAccountRequest>,
) -> Result<(), AppError> {
    _ = db::bank_accounts::get_single(&db_pool, account_id, user_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to get bank account")))?;

    db::bank_accounts::update(&db_pool, account_id, &request.name)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to update bank account")))?;

    Ok(())
}
