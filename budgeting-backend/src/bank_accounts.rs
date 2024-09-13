use anyhow::anyhow;
use axum::{
    extract::{Query, State}, Json
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(get_bank_accounts, create_bank_account),
    components(schemas(BankAccount, CreateBankAccountRequest))
)]
pub struct BankAccountApi;

const API_TAG: &str = "BankAccounts";

#[derive(Serialize, ToSchema)]
pub struct BankAccount {
    id: Uuid,
    name: String,
    initial_amount_dollars: i32,
    initial_amount_cents: u8,
    user_id: Uuid,
}

struct BankAccountDbModel {
    id: String,
    name: String,
    initial_amount_dollars: i32,
    initial_amount_cents: i32,
    user_id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateBankAccountRequest {
    name: String,
    initial_amount_dollars: i32,
    initial_amount_cents: u8,
    user_id: Uuid,
}

impl TryFrom<BankAccountDbModel> for BankAccount {
    type Error = anyhow::Error;

    fn try_from(value: BankAccountDbModel) -> Result<Self, Self::Error> {
        let id: Uuid = value.id.parse()?;
        let user_id: Uuid = value.user_id.parse()?;

        Ok(BankAccount {
            id,
            user_id,
            initial_amount_dollars: value.initial_amount_dollars,
            initial_amount_cents: value.initial_amount_cents as u8,
            name: value.name,
        })
    }
}

#[utoipa::path(
    post,
    path = "/api/bank-accounts",
    responses(
        (status = OK, description = "Success", body = Uuid, content_type = "application/json")
    ),
    request_body = CreateBankAccountRequest,
    tag = API_TAG
)]
pub async fn create_bank_account(
    State(db_pool): State<MySqlPool>,
    Json(request): Json<CreateBankAccountRequest>,
) -> Result<Json<Uuid>, AppError> {
    if request.initial_amount_cents > 99 {
        return Err(AppError::BadRequest(anyhow!(
            "Initial Amount Cents must not exceed 99"
        )));
    }

    if request.name.trim().is_empty() {
        return Err(AppError::BadRequest(anyhow!("Name must not be empty")));
    }

    if request.user_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("User Id must be set")));
    }

    let id = Uuid::new_v4();

    sqlx::query!("INSERT INTO BankAccounts (id, name, user_id, initial_amount_dollars, initial_amount_cents) VALUE(?, ?, ?, ?, ?)",
        id.as_simple(), request.name, request.user_id.as_simple(), request.initial_amount_dollars, request.initial_amount_cents
    ).execute(&db_pool).await?;

    Ok(Json(id))
}

#[derive(Deserialize)]
pub struct GetBankAccountsQuery {
    pub user_id: Uuid
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
    let bank_accounts: Vec<BankAccount> = sqlx::query_as!(
        BankAccountDbModel,
         "SELECT id, name, initial_amount_dollars, initial_amount_cents, user_id FROM BankAccounts WHERE user_id = ?",
          query.user_id.as_simple())
          .fetch_all(&db_pool)
          .await?
          .into_iter()
          .map(|bank_account| bank_account.try_into().unwrap())
          .collect();

    Ok(Json(bank_accounts.into_boxed_slice()))
}
