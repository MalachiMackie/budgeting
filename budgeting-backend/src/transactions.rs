use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    Json,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{payees::get_payee, AppError};

#[derive(OpenApi)]
#[openapi(
    paths(get_transactions, create_transaction),
    components(schemas(Transaction, CreateTransactionRequest))
)]
pub struct TransactionApi;

const API_TAG: &str = "Transactions";

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Transaction {
    id: Uuid,
    payee_id: Uuid,
    date: NaiveDate,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    amount: Decimal,
    bank_account_id: Uuid,
}

struct TransactionModel {
    id: String,
    payee_id: String,
    date: NaiveDate,
    amount: Decimal,
    bank_account_id: String,
}

impl TryFrom<TransactionModel> for Transaction {
    type Error = anyhow::Error;

    fn try_from(value: TransactionModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.parse()?,
            date: value.date,
            payee_id: value.payee_id.parse()?,
            amount: value.amount,
            bank_account_id: value.bank_account_id.parse()?,
        })
    }
}

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

    let transactions = sqlx::query_as!(TransactionModel, "SELECT id, amount, date, payee_id, bank_account_id FROM Transactions WHERE bank_account_id = ?", bank_account_id.as_simple())
        .fetch_all(&db_pool)
        .await?
        .into_iter()
        .map(|transaction| transaction.try_into())
        .collect::<Result<Vec<Transaction>, _>>()?;

    Ok(Json(transactions.into_boxed_slice()))
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    payee_id: Uuid,
    #[schema(value_type = f32)]
    #[serde(with = "rust_decimal::serde::float")]
    amount: Decimal,
    date: NaiveDate,
}

#[utoipa::path(
    post,
    path = "/api/bank-accounts/{bankAccountId}/transactions",
    responses(
        (status = OK, description = "Success", body = Uuid, content_type = "application/json")
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
) -> Result<Json<Uuid>, AppError> {
    if request.payee_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("Payee Id must be set")));
    }

    if bank_account_id.is_nil() {
        return Err(AppError::BadRequest(anyhow!("Bank Account Id must be set")));
    }

    let id = Uuid::new_v4();

    let payee = get_payee(request.payee_id, &db_pool).await?;

    if payee.is_none() {
        return Err(AppError::NotFound(anyhow::anyhow!("Payee not found")));
    }

    sqlx::query!(r"
            INSERT INTO Transactions (id, payee_id, date, amount, bank_account_id)
            VALUE (?, ?, ?, ?, ?)",
             id.as_simple(),
              request.payee_id.as_simple(),
               request.date,
                request.amount,
                bank_account_id.as_simple())
            .execute(&db_pool)
            .await?;

    Ok(Json(id))
}
