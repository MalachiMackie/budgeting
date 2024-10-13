use anyhow::anyhow;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use http::StatusCode;
use serde::Deserialize;
use sqlx::MySqlPool;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    db,
    models::{BankAccount, CreateBankAccountRequest, UpdateBankAccountRequest},
    AppError,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_bank_accounts,
        get_bank_account,
        create_bank_account,
        delete_bank_account,
        update_bank_account
    ),
    components(schemas(BankAccount, CreateBankAccountRequest))
)]
pub struct BankAccountApi;

const API_TAG: &str = "BankAccounts";

#[utoipa::path(
    post,
    path = "/api/bank-accounts",
    responses(
        (status = CREATED, description = "Success", body = Uuid, content_type = "application/json")
    ),
    request_body = CreateBankAccountRequest,
    tag = API_TAG
)]
pub async fn create_bank_account(
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

    db::bank_accounts::create_bank_account(&db_pool, id, request)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Could not create bank account")))?;

    Ok((StatusCode::CREATED, Json(id)))
}

#[derive(Deserialize)]
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
        ("user_id" = Uuid, Query,)
    ),
    tag = API_TAG
)]
pub async fn get_bank_accounts(
    Query(query): Query<GetBankAccountsQuery>,
    State(db_pool): State<MySqlPool>,
) -> Result<Json<Box<[BankAccount]>>, AppError> {
    // todo: validate user_id exists
    db::bank_accounts::get_bank_accounts(&db_pool, query.user_id)
        .await
        .map(Json)
        .map_err(|e| e.to_app_error(anyhow!("Could not get bank accounts")))
}

#[derive(Deserialize)]
pub struct GetBankAccountQuery {
    user_id: Uuid,
}

#[derive(Deserialize)]
pub struct DeleteBankAccountQuery {
    user_id: Uuid,
}

#[derive(Deserialize)]
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
        ("user_id" = Uuid, Query,),
        ("accountId" = Uuid, Path,)
    ),
    tag = API_TAG
)]
pub async fn get_bank_account(
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

    db::bank_accounts::get_bank_account(&db_pool, account_id, query.user_id)
        .await
        .map(Json)
        .map_err(|e| e.to_app_error(anyhow!("Could not get bank_account with id {account_id}")))
}

#[utoipa::path(
    delete,
    path = "/api/bank-accounts/:accountId",
    responses(
        (status = OK, description = "Success",)
    ),
    params(
        ("user_id" = Uuid, Query,),
        ("accountId" = Uuid, Path,)
    ),
    tag = API_TAG
)]
pub async fn delete_bank_account(
    State(db_pool): State<MySqlPool>,
    Path(account_id): Path<Uuid>,
    Query(DeleteBankAccountQuery { user_id }): Query<DeleteBankAccountQuery>,
) -> Result<(), AppError> {
    db::bank_accounts::get_bank_account(&db_pool, account_id, user_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to find bank account")))?;

    db::bank_accounts::delete_bank_account(&db_pool, account_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to delete bank account")))?;

    Ok(())
}

#[utoipa::path(
    put,
    path = "/api/bank-accounts/:accountId",
    responses(
        (status = OK, description = "Success",)
    ),
    params(
        ("user_id" = Uuid, Query,),
        ("accountId" = Uuid, Path,)
    ),
    tag = API_TAG
)]
pub async fn update_bank_account(
    State(db_pool): State<MySqlPool>,
    Path(account_id): Path<Uuid>,
    Query(UpdateBankAccountQuery { user_id }): Query<UpdateBankAccountQuery>,
    Json(request): Json<UpdateBankAccountRequest>,
) -> Result<(), AppError> {
    _ = db::bank_accounts::get_bank_account(&db_pool, account_id, user_id)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to get bank account")))?;

    db::bank_accounts::update_bank_account(&db_pool, account_id, &request.name)
        .await
        .map_err(|e| e.to_app_error(anyhow!("Failed to update bank account")))?;

    Ok(())
}
