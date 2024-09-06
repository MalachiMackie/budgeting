use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{payees::{get_payee, PayeeId}, AppError};

#[derive(Deserialize, Serialize)]
pub struct Transaction
{
    id: TransactionId,
    payee_id: PayeeId,
    time: DateTime<Utc>,
    amount_dollars: i32,
    amount_cents: u8
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct TransactionId(pub Uuid);

impl TransactionId {
    pub fn new() -> Self
    {
        Self(Uuid::new_v4())
    }
}

struct TransactionModel
{
    id: String,
    payee_id: String,
    time: DateTime<Utc>,
    amount_cents: i32,
    amount_dollars: i32
}

impl TryFrom<TransactionModel> for Transaction {
    type Error = anyhow::Error;

    fn try_from(value: TransactionModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: TransactionId(value.id.parse()?),
            time: value.time,
            payee_id: PayeeId(value.payee_id.parse()?),
            amount_cents: value.amount_cents as u8,
            amount_dollars: value.amount_dollars
        })
    }
}

pub async fn get_transactions(State(db_pool): State<MySqlPool>) -> Result<Json<Box<[Transaction]>>, AppError>
{
    let transactions = sqlx::query_as!(TransactionModel, "SELECT id, amount_dollars, time, amount_cents, payee_id FROM Transactions")
        .fetch_all(&db_pool)
        .await?
        .into_iter()
        .map(|transaction| transaction.try_into())
        .collect::<Result<Vec<Transaction>, _>>()?;

    Ok(Json(transactions.into_boxed_slice()))
}

#[derive(Deserialize)]
pub struct CreateTransactionRequest
{
    payee_id: Uuid,
    amount_dollars: i32,
    amount_cents: u8,
    time: DateTime<Utc>
}

pub async fn create_transaction(
    State(db_pool): State<MySqlPool>,
    Json(request): Json<CreateTransactionRequest>)
    -> Result<Json<Uuid>, AppError> {
        let id = TransactionId::new();

        let payee = get_payee(PayeeId(request.payee_id), &db_pool)
            .await?;

        if payee.is_none() {
            return Err(AppError::NotFound(anyhow::anyhow!("Payee not found")));
        }

        sqlx::query!(r"
            INSERT INTO Transactions (id, payee_id, time, amount_dollars, amount_cents)
            VALUE (?, ?, ?, ?, ?)", id.0.as_simple(), request.payee_id.as_simple(), request.time, request.amount_dollars, request.amount_cents)
            .execute(&db_pool)
            .await?;

        Ok(Json(id.0))
    }
